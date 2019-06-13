use actix_web::{web::Json, Error, HttpResponse, Result};
use futures::{self, Future};
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use serde::{Deserialize, Serialize};

use super::helpers::sort_tags;
use crate::db::messages::{GetUserTags, SetUserTags};
use crate::db::Pg;
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

async fn index(redis: Redis, pg: Pg, user_id: UserId) -> Result<HttpResponse> {
    let redis_tags = read_redis_tags(user_id, redis).await?;
    let user_tags = Box::new(pg.send(GetUserTags::new(user_id)))
        .compat()
        .await??;

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags, redis_tags)))
}

async fn update(redis: Redis, pg: Pg, user_id: UserId, data: Json<Data>) -> Result<HttpResponse> {
    let tags = data.into_inner().tags;
    let redis_tags = read_redis_tags(user_id, redis).await?;
    let user_tags = Box::new(pg.send(SetUserTags::new(user_id, tags)))
        .compat()
        .await??;

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags, redis_tags)))
}

pub mod service {
    use super::*;
    use actix_web::dev::HttpServiceFactory;

    pub struct Service;

    fn __index(
        redis: Redis,
        pg: Pg,
        user_id: UserId,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        super::index(redis, pg, user_id).boxed().compat()
    }

    fn __update(
        redis: Redis,
        pg: Pg,
        user_id: UserId,
        data: Json<Data>,
    ) -> impl Future<Item = HttpResponse, Error = Error> {
        super::update(redis, pg, user_id, data).boxed().compat()
    }

    impl HttpServiceFactory for Service {
        fn register(self, config: &mut actix_web::dev::AppService) {
            use actix_web::{
                guard::{Get, Put},
                Resource,
            };

            HttpServiceFactory::register(
                Resource::new("/").guard(Put()).to_async(__update),
                config,
            );
            HttpServiceFactory::register(Resource::new("/").guard(Get()).to_async(__index), config);
        }
    }
}
