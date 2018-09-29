extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate futures;

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

extern crate failure;

use actix_web::server;

mod apps;
pub mod db;

use apps::auth;

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    ::std::env::set_var(
        "DATABASE_URL",
        "postgres://postgres:@172.18.0.3:5432/postgres",
    );
    env_logger::init();

    // db::print_users();

    server::new(|| vec![auth::app()])
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
