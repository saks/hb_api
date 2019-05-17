use actix::{Actor, Addr, SyncArbiter, SyncContext};
use actix_web::web;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};

pub mod messages;
pub mod models;
pub mod pagination;
pub mod schema;

pub type Postgres = Addr<DbExecutor>;
pub type Pg = web::Data<Postgres>;

pub fn start() -> Postgres {
    use crate::config::DATABASE_URL;

    SyncArbiter::start(1, move || {
        let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());

        let pool = Pool::builder()
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
