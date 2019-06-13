use actix::prelude::*;
use futures03::compat::Future01CompatExt as _;
use redis::Value;

use super::{Error, RedisActor};

pub async fn send(addr: super::Addr, msg: PipelineMessage) -> Result<Value, Error> {
    let fut = addr.send(msg);
    let res = Box::new(fut).compat().await;

    match res {
        Ok(Ok(value)) => Ok(value),
        Err(e) => Err(Error::ActixMailbox(e)),
        Ok(Err(e)) => Err(Error::Redis(e)),
    }
}

pub struct Pipeline {
    inner: redis::Pipeline,
}

impl Pipeline {
    pub fn new() -> Self {
        let inner = redis::Pipeline::new();

        Self { inner }
    }

    pub fn add_command(&mut self, cmd: &crate::Cmd) {
        self.inner.add_command(cmd.inner());
    }

    pub fn send<T: redis::FromRedisValue>(
        self,
        addr: super::Addr,
    ) -> impl std::future::Future<Output = Result<T, Error>>
    where
        T: Send + 'static,
    {
        use futures03::future::FutureExt as _;

        let command = PipelineMessage(self.inner);

        send(addr, command).map(|result| {
            result.and_then(|value| redis::from_redis_value(&value).map_err(Into::into))
        })
    }
}

pub struct PipelineMessage(redis::Pipeline);

impl Message for PipelineMessage {
    type Result = Result<Value, redis::RedisError>;
}

impl Handler<PipelineMessage> for RedisActor {
    type Result = ResponseFuture<Value, redis::RedisError>;

    fn handle(&mut self, msg: PipelineMessage, _: &mut Self::Context) -> Self::Result {
        match self.conn() {
            Some(conn) => {
                // println!("executing command");

                let conn = (**conn).clone();
                let fut = msg.0.query_async::<_, Value>(conn).map(|(_conn, res)| res);

                Box::new(fut)
            }
            None => panic!("No redis connection in RedisActor!"),
        }
    }
}
