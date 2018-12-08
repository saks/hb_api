use actix::Addr;
use actix_redis::RedisActor;
use std::sync::Arc;

use crate::config;
use crate::db::DbExecutor;

mod index_params;
mod index_response;
pub mod middlewares;

pub mod auth_app;
pub mod budgets_app;
pub mod records_app;
pub mod tags_app;

/// State with DbExecutor address
pub struct AppState {
    db: Addr<DbExecutor>,
    redis: Arc<Addr<RedisActor>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: DbExecutor::start(),
            redis: Arc::new(RedisActor::start(config::redis_url())),
        }
    }
}
