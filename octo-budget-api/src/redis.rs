use actix_web::web::Data;
// use octo_redis::{Addr, RedisActor};

use crate::config;

pub type Redis = Data<Addr>;

pub fn start() -> Addr {
    config::redis_url();
    RedisActor::start(config::REDIS_URL.to_string())
}

pub mod helpers;
