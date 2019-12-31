use actix::{Actor, Addr, SyncArbiter, SyncContext};
use actix_web::web;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};

pub mod messages;
pub use models::{self, schema};
pub mod pagination;

pub type Postgres = Addr<DbExecutor>;
pub type Pg = web::Data<Postgres>;
pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn start() -> Postgres {
    use crate::config::DATABASE_URL;

    SyncArbiter::start(1, move || {
        let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());

        let pool = Pool::builder()
            .min_idle(Some(1))
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
