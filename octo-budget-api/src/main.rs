// disable warnings from diesel till 1.4 gets released
#![allow(proc_macro_derive_resolution_fallback)]

extern crate bigdecimal;
extern crate chrono;
extern crate dotenv;
extern crate failure;
extern crate futures;
extern crate serde;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
extern crate djangohashers;

extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate jsonwebtoken;
extern crate r2d2;
extern crate time;

#[cfg(test)]
#[macro_use]
extern crate serde_json;
extern crate octo_budget_lib;

pub mod apps;
pub mod config;
pub mod db;

use actix_web::server;
use dotenv::dotenv;

use crate::apps::auth_app;
use crate::apps::records_app;

fn main() {
    dotenv().expect("Failed to parse .env file");
    env_logger::init();

    server::new(|| vec![auth_app::build(), records_app::build()])
        .bind(format!("{}:{}", *config::LISTEN_IP, *config::LISTEN_PORT))
        .expect("Cannot bind to IP:PORT")
        .run();
}
