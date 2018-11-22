// disable warnings from diesel till 1.4 gets released
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use env_logger;

pub mod apps;
pub mod config;
pub mod db;

use actix_web::server;
use dotenv::dotenv;

use crate::apps::{auth_app, records_app};

fn main() {
    dotenv().expect("Failed to parse .env file");
    env_logger::init();

    server::new(|| vec![auth_app::build(), records_app::build()])
        .bind(format!("{}:{}", *config::LISTEN_IP, *config::LISTEN_PORT))
        .expect("Cannot bind to IP:PORT")
        .run();
}
