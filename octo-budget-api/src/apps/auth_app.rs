use actix_web::{
    web::{self, block, Json},
    HttpResponse, Result,
};

use self::utils::generate_token;
use super::forms::auth::{self, Form};
use crate::db::{messages::FindUserByName, PgPool};

mod response_data;
mod utils;

async fn create(form: Json<Form>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    dbg!(1);
    let auth::Data { username, password } = form.into_inner().validate()?;
    dbg!(2);
    let user = block(move || FindUserByName::new(username).query(&pool)).await?;

    Form::validate_password(&user, &password)?;

    Ok(HttpResponse::Ok().json(generate_token(&user)))
}

pub mod service {
    use super::*;
    use actix_web::dev::HttpServiceFactory;

    pub struct Service;

    impl HttpServiceFactory for Service {
        fn register(self, config: &mut actix_web::dev::AppService) {
            use actix_web::{guard::Post, Resource};

            HttpServiceFactory::register(Resource::new("/create/").guard(Post()).to(create), config)
        }
    }
}

#[cfg(test)]
mod tests;
