use actix::Addr;

use crate::db::DbExecutor;

mod index_params;
mod index_response;
pub mod middlewares;

pub mod auth_app;
pub mod budgets_app;
pub mod records_app;

/// State with DbExecutor address
pub struct AppState {
    db: Addr<DbExecutor>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            db: DbExecutor::new(),
        }
    }
}
