use actix_web::{dev::HttpServiceFactory, Error, HttpResponse, Result};
use futures::{self, Future};
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use serde_derive::{Deserialize, Serialize};

// use crate::db::messages::{GetUserTags, SetUserTags};
// use crate::redis::helpers::read_redis_tags;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Data {
    tags: Vec<String>,
}

pub struct Service;

fn index(req: actix_web::HttpRequest) -> impl Future<Item = HttpResponse, Error = Error> {
    let redis = req.get_app_data::<octo_redis::Db>().unwrap();
    let redis = (*redis).clone();

    __async_create(redis).boxed().compat()
}

// fn update(
//     redis: crate::redis2::RedisConnection,
// ) -> impl Future<Item = HttpResponse, Error = Error> {
//     __async_create(redis).boxed().compat()
// }

async fn __async_create(redis: octo_redis::Db) -> Result<HttpResponse> {
    let msg = octo_redis::Command::get("foo");
    // let result = redis.send(msg);

    let res = octo_redis::send(redis, msg).await?;
    dbg!(res);
    // let res = result.compat().await;
    // let cmd = redis::cmd("Get").arg("some-key").clone();
    // let _cmd = crate::redis2::Command(cmd);
    // let y = redis.send(cmd);
    // let x = Box::new(y).compat().await;
    // let value = crate::redis2::get("some-key", redis)
    //     .await
    //     .expect("failed to read key from redis");
    // println!("key from redis: `{:?}'", value);
    Ok(HttpResponse::Ok().body("{\"from\": \"tags app\"}"))
}

impl HttpServiceFactory for Service {
    fn register(self, config: &mut actix_web::dev::AppService) {
        use actix_web::{
            guard::{Get, Put},
            Resource,
        };

        // HttpServiceFactory::register(Resource::new("/").guard(Put()).to_async(update), config);
        HttpServiceFactory::register(Resource::new("/").guard(Get()).to_async(index), config);
    }
}
