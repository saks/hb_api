#![feature(await_macro, futures_api, async_await)]

#[macro_use]
extern crate diesel;

use env_logger;

// pub mod apps;
pub mod config;
// pub mod db;
// mod errors;
// mod redis;

#[cfg(test)]
mod tests;

use actix_web::{http::Method, middleware::Logger, HttpServer, App};
use dotenv::dotenv;

// use crate::apps::{
//     auth_app, budgets_app, frontend, middlewares, records_app, tags_app, users_app, AppState,
// };

// fn main() {
//     dotenv().expect("Failed to parse .env file");
//     env_logger::init();
//
//     HttpServer::new(|| {
//     //     App::with_state(AppState::default())
//     //         .middleware(middlewares::ForceHttps::default())
//     //         .middleware(Logger::default())
//     //         .resource("/", |r| r.method(Method::GET).f(frontend::index))
//     //         .scope("/public", frontend::scope)
//     //         .scope("/auth/jwt", auth_app::scope)
//     //         .scope("/api/records/", records_app::scope)
//     //         .scope("/api/budgets/", budgets_app::scope)
//     //         .scope("/api/tags/", tags_app::scope)
//     //         .scope("/api/user/", users_app::scope)
//     })
//     .bind(format!(
//         "{}:{}",
//         config::LISTEN_IP.as_str(),
//         config::PORT.as_str()
//     ))
//     .unwrap()
//     .run().unwrap();
// }

use actix_web_async_compat::async_compat;
use futures::Future;
use tokio_async_await::await;
use actix_web::{Error, Result, HttpRequest, HttpResponse, web};

#[async_compat]
async fn index2(_req: HttpRequest) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("OK"))
}

fn main() {
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/welcome2").route(web::get().to_async(index2)))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}
