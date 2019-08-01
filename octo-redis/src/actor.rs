use actix::prelude::*;
use log::debug;
use redis::aio::SharedConnection;
use std::sync::Arc;

pub struct RedisActor {
    addr: String,
    conn: Option<Arc<SharedConnection>>,
}

impl RedisActor {
    /// Start new `Supervisor` with `RedisActor`.
    pub fn start<S: Into<String>>(addr: S) -> super::Addr {
        let addr = addr.into();

        Supervisor::start(|_| RedisActor { addr, conn: None })
    }

    pub fn conn(&self) -> &Option<Arc<SharedConnection>> {
        &self.conn
    }
}

impl Supervised for RedisActor {
    fn restarting(&mut self, _: &mut Self::Context) {
        // TODO
        debug!("restarting...");
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
                debug!("Connected to redis!");
                act.conn = Some(Arc::new(conn.clone()));
            })
            .map_err(|err, _act, ctx| {
                debug!("Failed to connect to redis!: {:?}", err);
                let timeout = std::time::Duration::new(1, 0);
                ctx.run_later(timeout, |_, ctx| ctx.stop());
            })
            .wait(ctx);
    }
}
