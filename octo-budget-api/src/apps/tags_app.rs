use actix_web::{web, web::Json, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use super::helpers::sort_tags;
use crate::db::{
    messages::{GetUserTags, SetUserTags},
    ConnectionPool,
};
use crate::redis::{helpers::read_redis_tags, Redis};
use octo_budget_lib::auth_token::UserId;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Data {
    tags: Vec<String>,
}

fn ordered_tags(user_tags: Vec<String>, redis_tags: Vec<String>) -> Data {
    let tags = sort_tags(redis_tags, user_tags);
    Data { tags }
}

async fn index(
    user_id: UserId,
    redis: web::Data<Redis>,
    pool: web::Data<ConnectionPool>,
) -> Result<HttpResponse> {
    let redis_tags = read_redis_tags(user_id, &redis).await?;
    let user_tags = pool.execute(GetUserTags::new(user_id)).await?;

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags, redis_tags)))
}

async fn update(
    user_id: UserId,
    data: Json<Data>,
    redis: web::Data<Redis>,
    pool: web::Data<ConnectionPool>,
) -> Result<HttpResponse> {
    let tags = data.into_inner().tags;
    let redis_tags = read_redis_tags(user_id, &redis).await?;
    let user_tags = pool.execute(SetUserTags::new(user_id, tags)).await?;

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags, redis_tags)))
}

pub mod service {
    use super::*;
    use actix_web::dev::HttpServiceFactory;

    pub struct Service;

    impl HttpServiceFactory for Service {
        fn register(self, config: &mut actix_web::dev::AppService) {
            use actix_web::{
                guard::{Get, Put},
                Resource,
            };

            HttpServiceFactory::register(Resource::new("/").guard(Put()).to(update), config);
            HttpServiceFactory::register(Resource::new("/").guard(Get()).to(index), config);
        }
    }
}
