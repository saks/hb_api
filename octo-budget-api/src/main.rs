// disable warnings from diesel till 1.4 gets released
#![allow(proc_macro_derive_resolution_fallback)]

#![feature(await_macro, futures_api, async_await)]

#[macro_use]
extern crate diesel;

use env_logger;

pub mod apps;
pub mod config;
pub mod db;

#[cfg(test)]
mod tests;

use actix_web::{middleware::Logger, server, App};
use dotenv::dotenv;

use crate::apps::{auth_app, budgets_app, records_app, tags_app, AppState};

// try async/await
use actix_web::{http, Responder, Result};
use actix_web_async_await::{await, compat};
use crate::apps::{
    Request, State,
};

async fn index((state, _req): (State, Request)) -> Result<impl Responder> {
    let user_id = 701;
    let tags = await!(state.db.send(tags_app::db::get_user_tags_from_db_msg(user_id)))?;

    println!("X: {:?}", tags);

    // Proceed with normal response
    Ok(format!("Works!"))
}

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
            .route("/{id}/{name}/index.html", http::Method::GET, compat(index))
    })
    .bind(format!(
        "{}:{}",
        config::LISTEN_IP.as_str(),
        config::LISTEN_PORT.as_str()
    ))
    .expect("Cannot bind to IP:PORT")
    .run();
}
