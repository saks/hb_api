use actix_web::{dev::HttpServiceFactory, Error, HttpResponse, Result};
use futures::{self, Future};
use futures03::{FutureExt as _, TryFutureExt as _};
use serde_derive::{Deserialize, Serialize};

use crate::db::messages::{GetUserTags, SetUserTags};
use crate::redis::helpers::read_redis_tags;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Data {
    tags: Vec<String>,
}

pub struct Service;

fn index() -> impl Future<Item = HttpResponse, Error = Error> {
    __async_create().boxed().compat()
}

fn update() -> impl Future<Item = HttpResponse, Error = Error> {
    __async_create().boxed().compat()
}

async fn __async_create() -> Result<HttpResponse> {
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
