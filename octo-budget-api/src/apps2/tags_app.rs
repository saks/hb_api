use actix_web::{dev::HttpServiceFactory, web, Error, HttpResponse, Result};
use futures::{self, Future};
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use serde_derive::{Deserialize, Serialize};

type Redis = web::Data<octo_redis::Db>;

// use crate::db::messages::{GetUserTags, SetUserTags};
use crate::redis::helpers::read_redis_tags;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Data {
    tags: Vec<String>,
}

pub struct Service;

fn index(redis: Redis) -> impl Future<Item = HttpResponse, Error = Error> {
    __async_create(redis).boxed().compat()
}

fn update(redis: Redis) -> impl Future<Item = HttpResponse, Error = Error> {
    __async_create(redis).boxed().compat()
}

async fn __async_create(redis: Redis) -> Result<HttpResponse> {
    // let user_id = crate::auth_token_from_async_request!(req).user_id;
    let user_id = 123;
    dbg!(read_redis_tags(user_id, redis).await?);
    // let msg = octo_redis::Command::get("foo");
    // let res = octo_redis::send(redis.get_ref(), msg).await?;
    // dbg!(res);
    Ok(HttpResponse::Ok().body("{\"from\": \"tags app\"}"))
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
