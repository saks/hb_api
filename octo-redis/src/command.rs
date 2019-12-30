use actix::prelude::*;
use futures03::compat::Future01CompatExt as _;
// use futures03::compat::Future03CompatExt;
use log::{error, info};
use redis::Value;

use super::{Error, RedisActor};

pub async fn send(addr: super::Addr, msg: CmdMessage) -> Result<Value, Error> {
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

    pub fn inner(&self) -> redis::Cmd {
        self.inner.clone()
    }

    pub fn send<T: redis::FromRedisValue>(
        self,
        addr: super::Addr,
    ) -> impl std::future::Future<Output = Result<T, Error>>
    where
        T: Send + 'static,
    {
        use futures03::future::FutureExt as _;

        let command = CmdMessage(self.inner);

        send(addr, command).map(|result| {
            result.and_then(|value| redis::from_redis_value(&value).map_err(Into::into))
        })
    }
}

pub fn cmd(command: &str) -> Cmd {
    Cmd::new(command)
}

pub struct CmdMessage(pub redis::Cmd);

impl Message for CmdMessage {
    type Result = Result<Value, redis::RedisError>;
}

impl Handler<CmdMessage> for RedisActor {
    type Result = ResponseFuture<Value, redis::RedisError>;

    fn handle(&mut self, cmd: CmdMessage, ctx: &mut Self::Context) -> Self::Result {
        match self.conn() {
            Some(conn) => {
                // println!("executing command");

                let conn = (**conn).clone();
                let fut = cmd.0.query_async::<_, Value>(conn);

                let fut = async {
                    let x = Box::new(fut).compat().await;

                    match x {
                        _ => Ok(Value::Nil),
                    }
                };
                //     .map(|(_conn, res)| {
                //         //
                //         info!("got response from redis");
                //
                //         res
                //     })
                //     .map_err(|err| {
                //         error!("ERR FROM REDIS");
                //         // ctx.stop();
                //         // self.handle(RestartMsg, ctx);
                //         err
                //     });
                //
                // let res = async {
                //     let x = Box::new(fut).compat().await;
                //     redis::Value::Nil;
                // };
                // // Box::new(fut)
                let res = futures03::compat::Compat::new(fut);
                Box::new(res)
            }
            None => panic!("No redis connection in RedisActor!"),
        }
    }
}

struct RestartMsg;

impl Message for RestartMsg {
    type Result = Result<(), redis::RedisError>;
}

impl Handler<RestartMsg> for RedisActor {
    type Result = ResponseFuture<(), redis::RedisError>;

    fn handle(&mut self, cmd: RestartMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();

        Box::new(futures::future::ok(()))
    }
}
