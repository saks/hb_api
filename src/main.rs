// disable warnings from diesel till 1.4 gets released
#![allow(proc_macro_derive_resolution_fallback)]

extern crate actix_web;
extern crate dotenv;
extern crate octo_budget_api;

use actix_web::server;
use dotenv::dotenv;

use octo_budget_api::apps::auth_app;
use octo_budget_api::apps::records_app;
use octo_budget_api::config;

fn main() {
    dotenv().expect("Failed to parse .env file");
    env_logger::init();

    server::new(|| vec![auth_app::build(), records_app::build()])
        .bind(format!("{}:{}", *config::LISTEN_IP, *config::LISTEN_PORT))
        .expect("Cannot bind to IP:PORT")
        .run();
}
