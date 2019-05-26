use actix::Addr;
use actix_redis::RedisActor;
use std::sync::Arc;

use crate::config;

pub type Redis = Arc<Addr<RedisActor>>;

pub fn start() -> Redis {
    Arc::new(RedisActor::start(config::redis_url()))
}

pub mod helpers;
