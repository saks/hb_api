use actix::Addr;
use actix_redis::RedisActor;
use std::sync::Arc;

use crate::config;

pub type Redis = Arc<Addr<RedisActor>>;

pub fn start() -> std::sync::Arc<actix::address::Addr<actix_redis::redis::RedisActor>> {
    std::sync::Arc::new(RedisActor::start(config::redis_url()))
}

// pub mod helpers;
