// disable warnings from diesel till 1.4 gets released
#![allow(proc_macro_derive_resolution_fallback)]
#![feature(await_macro, futures_api, async_await)]

#[macro_use]
extern crate diesel;

use env_logger;

pub mod apps;
pub mod config;
pub mod db;
mod errors;
mod redis;

#[cfg(test)]
mod tests;

use actix_web::{middleware::Logger, server, App};
use dotenv::dotenv;

use crate::apps::{auth_app, budgets_app, records_app, tags_app, users_app, AppState};

fn main() {
    dotenv().expect("Failed to parse .env file");
    env_logger::init();

    server::new(|| {
        App::with_state(AppState::default())
            .middleware(Logger::default())
            .scope("/auth/jwt", auth_app::scope)
            .scope("/api/records/", records_app::scope)
            .scope("/api/budgets/", budgets_app::scope)
            .scope("/api/tags/", tags_app::scope)
            .scope("/api/user/", users_app::scope)
    })
    .bind(format!(
        "{}:{}",
        config::LISTEN_IP.as_str(),
        config::LISTEN_PORT.as_str()
    ))
    .expect("Cannot bind to IP:PORT")
    .run();
}
