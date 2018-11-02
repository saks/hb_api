use actix::{Actor, Addr, SyncArbiter, SyncContext};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use r2d2;

pub mod auth;
pub mod models;
pub mod pagination;
pub mod schema;

use crate::config;

/// This is db executor actor. We are going to run 3 of them in parallel.
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

impl DbExecutor {
    pub fn new() -> Addr<Self> {
        let manager = ConnectionManager::<PgConnection>::new(config::DATABASE_URL.to_string());

        let pool = r2d2::Pool::builder()
            .max_size(1) // max pool size
            .build(manager)
            .expect("Failed to create database connection pool.");

        SyncArbiter::start(1, move || DbExecutor(pool.clone()))
    }
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
