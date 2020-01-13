use actix_web::{get, put, web, web::Json, HttpResponse, Result};
use serde::{Deserialize, Serialize};

use super::helpers::sort_tags;
use crate::db::{
    queries::{GetUserTags, SetUserTags},
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

#[get("/")]
async fn index(
    user_id: UserId,
    redis: web::Data<Redis>,
    pool: web::Data<ConnectionPool>,
) -> Result<HttpResponse> {
    let redis_tags = read_redis_tags(user_id, &redis).await?;
    let user_tags = pool.execute(GetUserTags::new(user_id)).await?;

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags, redis_tags)))
}

#[put("/")]
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
            HttpServiceFactory::register(index, config);
            HttpServiceFactory::register(update, config);
        }
    }
}
