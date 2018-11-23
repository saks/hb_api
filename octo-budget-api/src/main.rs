// disable warnings from diesel till 1.4 gets released
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use env_logger;

pub mod apps;
pub mod config;
pub mod db;

use actix_web::{middleware::Logger, server, App};
use dotenv::dotenv;

use crate::apps::{auth_app, records_app, AppState};

fn main() {
    dotenv().expect("Failed to parse .env file");
    env_logger::init();

    server::new(|| {
        App::with_state(AppState::new())
            .middleware(Logger::default())
            .scope("/auth/jwt", auth_app::scope)
            .scope("/api/records/", records_app::scope)
    })
    .bind(format!("{}:{}", *config::LISTEN_IP, *config::LISTEN_PORT))
    .expect("Cannot bind to IP:PORT")
    .run();
}
