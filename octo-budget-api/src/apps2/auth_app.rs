use actix_web::{dev::HttpServiceFactory, web::Json, Error, HttpResponse, Result};
use futures::{self, Future};
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};

use self::utils::generate_token;
use super::forms::auth::Form;
use crate::db::messages::FindUserByName;
use crate::db::Pg;

mod response_data;
mod utils;

#[allow(non_camel_case_types)]
pub struct Service;

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

impl HttpServiceFactory for Service {
    fn register(self, config: &mut actix_web::dev::AppService) {
        use actix_web::{guard::Post, Resource};

        HttpServiceFactory::register(
            Resource::new("/create/").guard(Post()).to_async(create),
            config,
        )
    }
}

#[cfg(test)]
mod tests;
