use actix_web::{dev::HttpServiceFactory, web, Error, HttpRequest, HttpResponse, Result};
use futures::{self, Future};
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use serde_derive::{Deserialize, Serialize};

use super::helpers::sort_tags;
use crate::db::messages::{GetUserTags, SetUserTags};
use crate::db::Pg;
use crate::redis::helpers::read_redis_tags;
use crate::redis::Redis;
use octo_budget_lib::auth_token::AuthToken;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Data {
    tags: Vec<String>,
}

fn ordered_tags(user_tags: Vec<String>, redis_tags: Vec<String>) -> Data {
    let tags = sort_tags(redis_tags, user_tags);
    Data { tags }
}

pub struct Service;

fn index(
    redis: Redis,
    pg: Pg,
    req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let token = req.extensions().get::<AuthToken>().map(|t| t.user_id);
    return __async_index(redis, pg, token).boxed().compat();
}

async fn __async_index(redis: Redis, pg: Pg, user_id: Option<i32>) -> Result<HttpResponse> {
    let user_id = user_id.ok_or_else(|| HttpResponse::Unauthorized().finish())?;
    let redis_tags = read_redis_tags(user_id, redis).await?;
    let user_tags = Box::new(pg.send(GetUserTags::new(user_id)))
        .compat()
        .await??;

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags, redis_tags)))
}

fn update(
    redis: Redis,
    pg: Pg,
    _req: HttpRequest,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // let user_id = crate::auth_token_from_async_request!(req).user_id;
    let user_id = 9;
    __async_update(redis, pg, user_id).boxed().compat()
}

async fn __async_update(redis: Redis, pg: Pg, user_id: i32) -> Result<HttpResponse> {
    let redis_tags = read_redis_tags(user_id, redis).await?;
    let user_tags = Box::new(pg.send(GetUserTags::new(user_id)))
        .compat()
        .await??;

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags, redis_tags)))
}

impl HttpServiceFactory for Service {
    fn register(self, config: &mut actix_web::dev::AppService) {
        use actix_web::{
            guard::{Get, Put},
            Resource,
        };

        HttpServiceFactory::register(Resource::new("/").guard(Put()).to_async(update), config);
        HttpServiceFactory::register(Resource::new("/").guard(Get()).to_async(index), config);
    }
}
