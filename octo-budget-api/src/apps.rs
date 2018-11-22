use actix::Addr;

use crate::db::DbExecutor;

pub mod auth_app;
pub mod middlewares;
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
