use actix_web::{
    dev::HttpServiceFactory, guard::Post, web::Json, Error, HttpResponse, Resource, Result,
};
use actix_web_async_compat::async_compat;
use futures::{self, Future};
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};

use self::utils::generate_token;
use super::forms::auth::Form;
use crate::db::messages::FindUserByName;
use crate::db::Pg;

mod response_data;
mod utils;

#[allow(non_camel_case_types)]
pub struct resource;

fn create(form: Json<Form>, db: Pg) -> impl Future<Item = HttpResponse, Error = Error> {
    __async_create(form, db).boxed().compat()
}

async fn __async_create(form: Json<Form>, db: Pg) -> Result<HttpResponse> {
    let data = form.into_inner().validate()?;
    let user = Box::new(db.send(FindUserByName(data.username)))
        .compat()
        .await??;

    Form::validate_password(&user, &data.password)?;

    Ok(HttpResponse::Ok().json(generate_token(&user)))
}

impl HttpServiceFactory for resource {
    fn register(self, config: &mut actix_web::dev::AppService) {
        HttpServiceFactory::register(
            Resource::new("/create/").guard(Post()).to_async(create),
            config,
        )
    }
}
