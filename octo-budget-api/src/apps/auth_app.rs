use actix_web::{web::Json, Error, HttpResponse, Result};
use futures::Future;
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};

use self::utils::generate_token;
use super::forms::auth::Form;
use crate::db::{messages::FindUserByName, Pg};

mod response_data;
mod utils;

async fn create(form: Json<Form>, db: Pg) -> Result<HttpResponse> {
    let data = form.into_inner().validate()?;
    let user = Box::new(db.send(FindUserByName(data.username)))
        .compat()
        .await??;

    Form::validate_password(&user, &data.password)?;

    Ok(HttpResponse::Ok().json(generate_token(&user)))
}

pub mod service {
    use super::*;
    use actix_web::dev::HttpServiceFactory;

    pub struct Service;

    fn __create(form: Json<Form>, db: Pg) -> impl Future<Item = HttpResponse, Error = Error> {
        create(form, db).boxed().compat()
    }

    impl HttpServiceFactory for Service {
        fn register(self, config: &mut actix_web::dev::AppService) {
            use actix_web::{guard::Post, Resource};

            HttpServiceFactory::register(
                Resource::new("/create/").guard(Post()).to_async(__create),
                config,
            )
        }
    }

}

#[cfg(test)]
mod tests;
