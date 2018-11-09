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

pub mod apps;
pub mod auth_token;
pub mod config;
pub mod db;
