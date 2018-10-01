use actix::{Actor, Addr, SyncArbiter, SyncContext};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use r2d2;

pub mod auth;
pub mod models;
pub mod schema;

use config;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl DbExecutor {
    pub fn new() -> Addr<Self> {
        let manager = ConnectionManager::<PgConnection>::new(config::DATABASE_URL.to_string());

        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create database connection pool.");

        SyncArbiter::start(*config::DATABASE_POOL_SIZE, move || {
            DbExecutor(pool.clone())
        })
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
