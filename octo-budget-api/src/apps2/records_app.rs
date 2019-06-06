use actix_http::Error;
use actix_web::{
    dev::HttpServiceFactory,
    web::{Json, Query},
    HttpResponse, Result,
};
use futures::Future;
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use octo_budget_lib::auth_token::UserId;

use super::forms::record::Form;
use super::index_params::Params;
use crate::db::{
    messages::{CreateRecord, GetRecords},
    Pg,
};
use crate::redis::{
    helpers::{decrement_tags, increment_tags},
    Redis,
};

pub struct Service;

async fn index(params: Query<Params>, pg: Pg, user_id: UserId) -> Result<HttpResponse> {
    let params = params.into_inner().validate()?;

    let message = GetRecords {
        page: params.page,
        per_page: params.per_page,
        user_id: user_id.into(),
    };

    let records = Box::new(pg.send(message)).compat().await??;

    Ok(HttpResponse::Ok().json(records))
}

async fn create(form: Json<Form>, pg: Pg, redis: Redis, user_id: UserId) -> Result<HttpResponse> {
    let data = form.into_inner().validate()?;

    Box::new(pg.send(CreateRecord::new(&data, user_id)))
        .compat()
        .await??;
    increment_tags(user_id, data.tags, redis).await?;

    Ok(HttpResponse::Ok().json(""))
}

fn __index(
    params: Query<Params>,
    pg: Pg,
    user_id: UserId,
) -> impl Future<Item = HttpResponse, Error = Error> {
    index(params, pg, user_id).boxed().compat()
}

fn __create(
    form: Json<Form>,
    pg: Pg,
    redis: Redis,
    user_id: UserId,
) -> impl Future<Item = HttpResponse, Error = Error> {
    create(form, pg, redis, user_id).boxed().compat()
}

impl HttpServiceFactory for Service {
    fn register(self, config: &mut actix_web::dev::AppService) {
        use actix_web::{
            guard::{Get, Post},
            Resource,
        };

        HttpServiceFactory::register(
            Resource::new("/record-detail/")
                .guard(Get())
                .to_async(__index),
            config,
        );
        HttpServiceFactory::register(
            Resource::new("/record-detail/")
                .guard(Post())
                .to_async(__create),
            config,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Service;
    use crate::{db::builders::UserBuilder, test_server, tests};
    use actix_http_test::TestServerRuntime;
    use actix_web::http::{header, StatusCode};
    use octo_budget_lib::auth_token::AuthToken;
    use serde_json::{json, Value};

    fn setup() -> TestServerRuntime {
        tests::setup_env();
        test_server!(Service)
    }

    #[test]
    fn index_return_empty_list() {
        let session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default().tags(vec!["foo"]));

        let token = AuthToken::new(user.id)
            .expire_in_hours(10)
            .encrypt(crate::config::AUTH_TOKEN_SECRET.as_bytes());

        let request = srv
            .get("/record-detail/")
            .header("Authorization", format!("JWT {}", token))
            .send();

        let mut response = srv.block_on(request).expect("failed to send request");
        let response_body = srv
            .block_on(response.json::<Value>())
            .expect("failed to parse response");

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");

        assert_eq!(
            json!({"total": 0, "results": [], "next": false, "previous": false}),
            response_body
        );
    }
}
