use actix::{Actor, Addr, SyncArbiter, SyncContext};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use r2d2;

pub mod messages;
pub mod models;
pub mod pagination;
pub mod schema;

pub type Postgres = Addr<DbExecutor>;

pub fn start() -> Postgres {
    use crate::config::DATABASE_URL;

    SyncArbiter::start(1, move || {
        let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());

        let pool = r2d2::Pool::builder()
            .max_size(1) // max pool size
            .build(manager)
            .expect("Failed to create database connection pool.");

        DbExecutor { pool }
    })
}

pub struct DbExecutor {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[cfg(test)]
pub mod builders;
