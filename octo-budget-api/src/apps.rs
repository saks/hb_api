use actix_web::{FutureResponse, HttpRequest, HttpResponse, State as WebState};

use crate::db::{self as db, Postgres};
use crate::redis::{self as redis, Redis};

pub mod forms;
pub mod helpers;
pub mod index_params;
pub mod index_response;
pub mod middlewares;

pub mod auth_app;
pub mod budgets_app;
pub mod records_app;
pub mod tags_app;

#[macro_export]
macro_rules! auth_token_from_async_request {
    ($request:ident) => {
        match $request
            .extensions_mut()
            .remove::<octo_budget_lib::auth_token::AuthToken>()
        {
            Some(token) => token,
            _ => return Ok(HttpResponse::Unauthorized().finish()),
        }
    };
}

// type aliases, just for convenience
pub type State = WebState<AppState>;
pub type Request = HttpRequest<AppState>;
pub type Response = FutureResponse<HttpResponse>;

pub struct AppState {
    db: Postgres,
    redis: Redis,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: db::start(),
            redis: redis::start(),
        }
    }

    pub fn redis(&self) -> Redis {
        self.redis.clone()
    }
}
