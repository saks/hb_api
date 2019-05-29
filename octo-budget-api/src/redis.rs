use actix_web::web::Data;
use octo_redis::{Db, RedisActor};

use crate::config;

pub type Redis = Data<Db>;

pub fn start() -> Db {
    dbg!(config::redis_url());
    RedisActor::start(config::redis_url())
}

pub mod helpers;
