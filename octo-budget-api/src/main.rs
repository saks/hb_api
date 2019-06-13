#![feature(await_macro, async_await)]

#[macro_use]
extern crate diesel;

mod apps;
mod config;
mod db;
mod errors;
mod redis;

#[cfg(test)]
mod tests;

use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use octo_budget_lib::auth_token::ApiJwtTokenAuthConfig;

fn main() -> Result<(), std::io::Error> {
    dotenv().expect("Failed to parse .env file");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .data(db::start())
            .data(redis::start())
            .data(ApiJwtTokenAuthConfig::new(
                config::AUTH_TOKEN_SECRET.as_bytes(),
            ))
            .wrap(middlewares::force_https::ForceHttps::new(
                config::is_force_https(),
            ))
            .wrap(Logger::default())
            .service(apps::frontend_app::index)
            .service(
                web::scope("/public")
                    .wrap(middlewares::pwa_cache_headers::PwaCacheHeaders)
                    .service(actix_files::Files::new("/", "./reactapp/build")),
            )
            .service(web::scope("/auth/jwt").service(apps::AuthService))
            .service(web::scope("/api/tags").service(apps::TagsService))
            .service(web::scope("/api/user").service(apps::users_app::show))
            .service(web::scope("/api/records").service(apps::RecordsService))
            .service(web::scope("/api/budgets").service(apps::BudgetsService))
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
