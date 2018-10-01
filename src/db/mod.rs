use actix::{Actor, Addr, SyncArbiter, SyncContext};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use r2d2;
use std::env;

pub mod auth;
pub mod models;
pub mod schema;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl DbExecutor {
    pub fn new() -> Addr<Self> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let manager = ConnectionManager::<PgConnection>::new(database_url);

        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        SyncArbiter::start(3, move || DbExecutor(pool.clone()))
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
