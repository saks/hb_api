#![feature(async_await)]

use actix::prelude::*;
use failure::Fail;
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use redis::r#async::SharedConnection;
use redis::Cmd;
use std::sync::Arc;

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
}

impl actix_http::ResponseError for Error {}

use futures::Future;
pub async fn send(db: Db, msg: Command) -> Result<redis::Value, Error> {
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

pub struct Command(pub Cmd);

impl Command {
    pub fn get(key: &str) -> Self {
        Self(redis::cmd("GET").arg(key).clone())
    }
}

impl Message for Command {
    type Result = Result<redis::Value, redis::RedisError>;
}

impl Handler<Command> for RedisActor {
    type Result = ResponseFuture<redis::Value, redis::RedisError>;

    fn handle(&mut self, cmd: Command, _: &mut Self::Context) -> Self::Result {
        match &self.conn {
            Some(conn) => {
                // println!("executing command");

                let conn = (**conn).clone();
                let fut = cmd
                    .0
                    .query_async::<_, redis::Value>(conn)
                    .map(|(_conn, res)| res);

                Box::new(fut)
            }
            None => panic!("No redis connection in RedisActor!"),
        }
    }
}
