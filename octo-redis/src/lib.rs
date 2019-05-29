#![feature(async_await)]

use actix::prelude::*;
use failure::Fail;
// use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use futures03::compat::Future01CompatExt as _;
use redis::r#async::SharedConnection;
use redis::Cmd;
use std::sync::Arc;

pub use redis::Value;

pub struct RedisActor {
    addr: String,
    conn: Option<Arc<SharedConnection>>,
}

pub type Db = Addr<RedisActor>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Actix mailbox error {}", _0)]
    ActixMailbox(#[cause] actix::MailboxError),
    #[fail(display = "Redis error {}", _0)]
    Redis(#[cause] redis::RedisError),
    #[fail(display = "Redis error {:?}", _0)]
    UnexpecetdRedisResponse(Value),
}

impl actix_http::ResponseError for Error {}

use futures::Future;
pub async fn send2(db: Db, msg: Command) -> Result<Value, Error> {
    let fut = db.send(msg);
    let res = Box::new(fut).compat().await;

    match res {
        Ok(Ok(value)) => Ok(value),
        Err(e) => Err(Error::ActixMailbox(e)),
        Ok(Err(e)) => Err(Error::Redis(e)),
    }
}
pub async fn send(db: &Db, msg: Command) -> Result<Value, Error> {
    let fut = db.send(msg);
    let res = Box::new(fut).compat().await;

    match res {
        Ok(Ok(value)) => Ok(value),
        Err(e) => Err(Error::ActixMailbox(e)),
        Ok(Err(e)) => Err(Error::Redis(e)),
    }
}

impl RedisActor {
    /// Start new `Supervisor` with `RedisActor`.
    pub fn start<S: Into<String>>(addr: S) -> Db {
        let addr = addr.into();

        Supervisor::start(|_| RedisActor { addr, conn: None })
    }
}

impl Supervised for RedisActor {
    fn restarting(&mut self, _: &mut Self::Context) {
        // TODO
        dbg!("restarting...");
    }
}

impl Actor for RedisActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let client = redis::Client::open(self.addr.as_ref()).expect("Failed to parse redis url");

        client
            .get_shared_async_connection()
            .into_actor(self)
            .map(|conn, act, _ctx| {
                println!("Connected to redis!");
                act.conn = Some(Arc::new(conn.clone()));
            })
            .map_err(|err, _act, ctx| {
                println!("Failed to connect to redis!: {:?}", err);
                let timeout = std::time::Duration::new(1, 0);
                ctx.run_later(timeout, |_, ctx| ctx.stop());
            })
            .wait(ctx);
    }
}

pub struct Cmd2 {
    inner: Cmd,
}

impl Cmd2 {
    pub fn new(command: &str) -> Self {
        let mut inner = redis::Cmd::new();
        inner.arg(command);

        Self { inner }
    }

    pub fn arg<T: redis::ToRedisArgs>(self, arg: T) -> Self {
        let mut inner = self.inner;
        inner.arg(arg);

        Self { inner }
    }

    pub fn send(self, addr: Db) -> impl std::future::Future<Output = Result<Value, Error>> {
        let cmd = Command(self.inner);
        send2(addr, cmd)
    }
}

pub fn cmd(command: &str) -> Cmd2 {
    Cmd2::new(command)
}

pub struct Command(pub Cmd);

impl Message for Command {
    type Result = Result<Value, redis::RedisError>;
}

impl Handler<Command> for RedisActor {
    type Result = ResponseFuture<Value, redis::RedisError>;

    fn handle(&mut self, cmd: Command, _: &mut Self::Context) -> Self::Result {
        match &self.conn {
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures03::future::{FutureExt, TryFutureExt};
    use redis::Commands;

    fn setup() -> redis::Connection {
        let client = redis::Client::open("redis://127.0.0.1/").expect("Failed build client");
        client.get_connection().expect("Failed to connect")
    }

    #[test]
    fn foo() {
        let conn = setup();
        let _: bool = conn.set("foo", "ZZZ!!!").unwrap();

        let sys = System::new("test");
        let addr = RedisActor::start("redis://127.0.0.1/");
        let msg: String = cmd("GET").arg("foo").send(addr);

        Arbiter::spawn(
            msg.unit_error()
                .boxed()
                .compat()
                .map(|res| {
                    assert_eq!("ZZZ!!!", res.unwrap());
                    System::current().stop();
                })
                .map_err(|e| {
                    dbg!(e);
                    System::current().stop();
                }),
        );

        sys.run().expect("failed to run system");
    }
}
