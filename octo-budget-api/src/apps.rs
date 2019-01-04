use actix::Addr;
use actix_redis::RedisActor;
use actix_web::{FutureResponse, HttpRequest, HttpResponse, State as WebState};
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

// type aliases, just for convenience
pub type State = WebState<AppState>;
pub type Request = HttpRequest<AppState>;
pub type Response = FutureResponse<HttpResponse>;

/// State with DbExecutor address
pub struct AppState {
    pub db: Addr<DbExecutor>,
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
