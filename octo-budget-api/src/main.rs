#![feature(await_macro, async_await)]

#[macro_use]
extern crate diesel;

use env_logger;

mod apps2;
mod config;
mod db;
mod errors;
mod redis;
// mod redis2;
// use octo_redis::RedisActor;

#[cfg(test)]
mod tests;

use actix_web::{web, App, HttpServer};
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

use actix_web::middleware::Logger;

fn main() -> Result<(), std::io::Error> {
    dotenv().expect("Failed to parse .env file");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .data(db::start())
            .data(redis::start())
            .wrap(middlewares::force_https::ForceHttps::new(
                config::is_force_https(),
            ))
            .wrap(Logger::default())
            .service(apps2::frontend_app::index)
            .service(
                web::scope("/public")
                    .wrap(middlewares::pwa_cache_headers::PwaCacheHeaders)
                    .service(actix_files::Files::new("/", "./reactapp/build")),
            )
            .service(web::scope("/auth/jwt").service(apps2::AuthService))
            .service(web::scope("/api/tags").service(apps2::TagsService))
    })
    .bind(format!(
        "{}:{}",
        config::LISTEN_IP.as_str(),
        config::PORT.as_str()
    ))
    .expect("Cannot bind to IP:PORT")
    .run()?;

    Ok(())
}
