extern crate failure;
extern crate futures;
extern crate serde;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
// #[macro_use]
// extern crate validator_derive;
extern crate validator;

#[macro_use]
extern crate diesel;
extern crate dotenv;

extern crate djangohashers;

extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate frank_jwt;
extern crate r2d2;
extern crate time;

#[macro_use]
extern crate serde_json;

use actix_web::server;
use dotenv::dotenv;

mod apps;
mod config;
mod db;

use apps::auth_app;
use apps::records_app;

fn main() {
    dotenv().ok().expect("Failed to parse .env file");
    env_logger::init();

    server::new(|| vec![auth_app::build(), records_app::build()])
        .bind(format!("{}:{}", *config::LISTEN_IP, *config::LISTEN_PORT))
        .expect("Cannot bind to IP:PORT")
        .run();
}
