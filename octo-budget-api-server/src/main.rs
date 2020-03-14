use actix_web::{middleware::Logger, App, HttpServer};
use dotenv::dotenv;

use octo_budget_api::{config, db::ConnectionPool, redis::Redis, routes::init_routes};
use octo_budget_lib::auth_token::ApiJwtTokenAuthConfig;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect("Failed to parse .env file");
    env_logger::init();

    let redis = Redis::new().await;

    HttpServer::new(move || {
        App::new()
            .data(ConnectionPool::new())
            .data(redis.clone())
            .app_data(ApiJwtTokenAuthConfig::new(
                config::AUTH_TOKEN_SECRET.as_bytes(),
            ))
            .wrap(middlewares::force_https::ForceHttps::new(
                config::is_force_https(),
            ))
            .wrap(Logger::default())
            .configure(init_routes)
    })
    .bind(format!(
        "{}:{}",
        config::LISTEN_IP.as_str(),
        config::PORT.as_str()
    ))?
    .run()
    .await
}
