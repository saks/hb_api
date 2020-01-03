use actix::{Actor, Addr, SyncArbiter, SyncContext};
use actix_web::web;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};

pub mod messages;
pub use models::{self, schema};
pub mod pagination;

pub type PooledConnection =
    r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub trait DatabaseQuery {
    type Data: Send;
    fn execute(&self, pool: PooledConnection) -> Result<Self::Data, failure::Error>;
}

pub struct ConnectionPool(Pool<ConnectionManager<PgConnection>>);

use actix_web::web::block;
impl ConnectionPool {
    pub fn new() -> Self {
        Self(create_pool())
    }

    pub async fn execute<T: DatabaseQuery + Send + 'static>(
        &self,
        query: T,
    ) -> Result<T::Data, failure::Error> {
        use actix_http::error::BlockingError;
        let connection = self.0.get()?;

        block(move || query.execute(connection))
            .await
            .map_err(|e| match e {
                BlockingError::Error(err) => err,
                BlockingError::Canceled => failure::format_err!("Thread pool is gone"),
            })
    }
}

fn create_pool() -> Pool<ConnectionManager<PgConnection>> {
    use crate::config::DATABASE_URL;

    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());

    Pool::builder()
        .min_idle(Some(1))
        .max_size(1) // max pool size
        .build(manager)
        .expect("Failed to create database connection pool.")
}

pub struct DbExecutor {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[cfg(test)]
pub mod builders;
