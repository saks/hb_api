use actix::Addr;

use db::DbExecutor;

pub mod auth_app;

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
