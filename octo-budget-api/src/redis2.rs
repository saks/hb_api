use futures03::compat::Future01CompatExt as _;
use futures03::{FutureExt as _, TryFutureExt as _};
use redis::r#async::SharedConnection;
use redis::RedisResult;
use std::sync::Arc;

pub type RedisConnection = Arc<SharedConnection>;
pub type Redis = Addr<RedisActor>;

use actix::prelude::*;
use actix::{Actor, Addr, Context};
pub struct RedisActor {
    addr: String,
    conn: Option<Arc<SharedConnection>>,
}

impl RedisActor {
    /// Start new `Supervisor` with `RedisActor`.
    pub fn start<S: Into<String>>(addr: S) -> Addr<RedisActor> {
        let addr = addr.into();

        Supervisor::start(|_| RedisActor { addr, conn: None })
    }
}

impl Supervised for RedisActor {
    fn restarting(&mut self, _: &mut Self::Context) {
        // TODO
        // self.cell.take();
        // for tx in self.queue.drain(..) {
        //     let _ = tx.send(Err(Error::Disconnected));
        // }
    }
}

impl Actor for RedisActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to parse redis url");
        client
            .get_shared_async_connection()
            .into_actor(self)
            .map(|conn, act, _ctx| {
                println!("Connected to redis!");
                act.conn = Some(Arc::new(conn.clone()));
                ()
            })
            .map_err(|err, _act, ctx| {
                println!("Failed to connect to redis!: {:?}", err);
                let timeout = std::time::Duration::new(1, 0);
                ctx.run_later(timeout, |_, ctx| ctx.stop());
            })
            .wait(ctx);

        // Resolver::from_registry()
        //     .send(Connect::host(self.addr.as_str()))
        //     .into_actor(self)
        //     .map(|res, act, ctx| match res {};
    }
}

// pub fn start() -> Redis {
//     use crate::config::DATABASE_URL;
//
//     SyncArbiter::start(1, move || {
//         let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());
//
//         let pool = Pool::builder()
//             .min_idle(Some(1))
//             .max_size(1) // max pool size
//             .build(manager)
//             .expect("Failed to create database connection pool.");
//
//         DbExecutor { pool }
//     })
// }

fn connect() -> impl futures::Future<Item = SharedConnection, Error = redis::RedisError> {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to parse redis url");

    client.get_shared_async_connection()
}

// async fn connect() -> RedisResult<SharedConnection> {
//     let client = redis::Client::open("redis://127.0.0.1/").expect("Failed to parse redis url");
//     let x = client
//         .get_shared_async_connection()
//         .compat()
//         .await
//         .expect("Failed to connect to redis");
//
//     Ok(x)
// }
//
// pub fn get_connection() -> SharedConnection {
//     let future01 = connect().unit_error().boxed().compat();
//
//     let mut rt = actix_rt::System::new("redis sys");
//     let x = rt.block_on(future01);
//     match x {
//         Ok(Ok(conn)) => return conn,
//         _ => panic!("Failed to connect to redis"),
//     }
// }

pub async fn get(key: &str, conn: RedisConnection) -> RedisResult<String> {
    let conn = (*conn).clone();

    let (_conn1, foo) = redis::cmd("GET")
        .arg(key)
        .query_async::<_, String>(conn)
        .compat()
        .await?;

    Ok(foo)
}

// struct RedisError;
// struct RedisResponse;
pub struct Command(pub redis::Cmd);
// type Res = Box<actix::fut::ActorFuture<Item = (), Actor = RedisActor, Error = ()>>;
// type Res = Result<String, ()>;
// type Res = ResponseFuture<redis::Value, redis::ErrorKind>;
type Res = Box<Future<Item = Result<redis::Value, redis::ErrorKind>, Error = ()>>;

impl Message for Command {
    type Result = Res;
}

impl Handler<Command> for RedisActor {
    type Result = Res;

    fn handle(&mut self, cmd: Command, _: &mut Self::Context) -> Res {
        match &self.conn {
            Some(conn) => {
                let conn = (**conn).clone();
                let fut = cmd.0.query_async::<_, redis::Value>(conn);
                // .map(|(_, val)| {
                //     dbg!(val);
                //     ()
                // })
                // .map_err(|e| {
                //     dbg!(e);
                //     ()
                // });

                // let _ = Box::new(fut);
                // Ok(String::new())
                // let x = futures03::future::ready(());
                // Box::new(futures03::future::ready(redis::Value::Nil))
                // Box::new(fut)
                fut
            }
            None => panic!("No redis connection in RedisActor!"),
        }
    }
}
