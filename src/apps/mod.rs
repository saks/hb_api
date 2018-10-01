use actix::Addr;
pub mod auth;

use db::DbExecutor;

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
