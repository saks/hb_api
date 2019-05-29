use actix::prelude::*;
use futures03::compat::Future01CompatExt as _;
use redis::Value;

use super::{Error, RedisActor};

pub async fn send(addr: super::Addr, msg: Command) -> Result<Value, Error> {
    let fut = addr.send(msg);
    let res = Box::new(fut).compat().await;

    match res {
        Ok(Ok(value)) => Ok(value),
        Err(e) => Err(Error::ActixMailbox(e)),
        Ok(Err(e)) => Err(Error::Redis(e)),
    }
}

pub struct Cmd {
    inner: redis::Cmd,
}

impl Cmd {
    pub fn new(command: &str) -> Self {
        let mut inner = redis::Cmd::new();
        inner.arg(command);

        Cmd { inner }
    }

    pub fn arg<T: redis::ToRedisArgs>(self, arg: T) -> Self {
        let mut inner = self.inner;
        inner.arg(arg);

        Cmd { inner }
    }

    pub fn send<T: redis::FromRedisValue>(
        self,
        addr: super::Addr,
    ) -> impl std::future::Future<Output = Result<T, Error>>
    where
        T: Send + 'static,
    {
        use futures03::future::FutureExt as _;

        let command = Command(self.inner);

        send(addr, command).map(|result| {
            result.and_then(|value| redis::from_redis_value(&value).map_err(Into::into))
        })
    }
}

pub fn cmd(command: &str) -> Cmd {
    Cmd::new(command)
}

pub struct Command(pub redis::Cmd);

impl Message for Command {
    type Result = Result<Value, redis::RedisError>;
}

impl Handler<Command> for RedisActor {
    type Result = ResponseFuture<Value, redis::RedisError>;

    fn handle(&mut self, cmd: Command, _: &mut Self::Context) -> Self::Result {
        match self.conn() {
            Some(conn) => {
                // println!("executing command");

                let conn = (**conn).clone();
                let fut = cmd.0.query_async::<_, Value>(conn).map(|(_conn, res)| res);

                Box::new(fut)
            }
            None => panic!("No redis connection in RedisActor!"),
        }
    }
}
