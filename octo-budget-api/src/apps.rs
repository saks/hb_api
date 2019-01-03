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

#[macro_export]
macro_rules! auth_token_from_request {
    ($request:ident) => {
        match $request
            .extensions_mut()
            .remove::<octo_budget_lib::auth_token::AuthToken>()
        {
            Some(token) => token,
            _ => {
                return Box::new(futures::future::ok(
                    actix_web::HttpResponse::Unauthorized().finish(),
                ))
            }
        }
    };
}

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
