use actix_http::Error;
use actix_web::{
    dev::HttpServiceFactory,
    web::{Json, Path, Query},
    HttpResponse, Result,
};
use futures::Future;
use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use octo_budget_lib::auth_token::UserId;

use super::forms::record::Form;
use super::index_params::Params;
use crate::db::{
    messages::{CreateRecord, FindRecord, GetRecords, UpdateRecord},
    Pg,
};
use crate::redis::{
    helpers::{decrement_tags, increment_tags},
    Redis,
};

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
    use serde_json::json;

    let data = form.into_inner().validate()?;

    let id = Box::new(pg.send(CreateRecord::new(&data, user_id)))
        .compat()
        .await??;
    increment_tags(user_id, data.tags, redis).await?;

    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}

async fn update(
    params: Path<i32>,
    form: Json<Form>,
    db: Pg,
    redis: Redis,
    user_id: UserId,
) -> Result<HttpResponse> {
    let record_id = params.into_inner();
    let data = form.into_inner().validate()?;

    let record = Box::new(db.send(FindRecord::new(record_id, user_id)))
        .compat()
        .await??;
    Box::new(db.send(UpdateRecord::new(record.id, &data, user_id)))
        .compat()
        .await??;

    decrement_tags(user_id, record.tags, redis.clone()).await?;
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

fn __update(
    params: Path<i32>,
    form: Json<Form>,
    db: Pg,
    redis: Redis,
    user_id: UserId,
) -> impl Future<Item = HttpResponse, Error = Error> {
    update(params, form, db, redis, user_id).boxed().compat()
}

pub struct Service;

impl HttpServiceFactory for Service {
    fn register(self, config: &mut actix_web::dev::AppService) {
        use actix_web::{
            guard::{Get, Post, Put},
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
        HttpServiceFactory::register(
            Resource::new("/record-detail/{id}")
                .guard(Put())
                .to_async(__update),
            config,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Service;
    use crate::tests::RequestJwtAuthExt as _;
    use crate::{db::builders::UserBuilder, test_server, tests};
    use actix_http::http::Method;
    use actix_http_test::TestServerRuntime;
    use actix_web::client::ClientRequest;
    use actix_web::http::{header, StatusCode};
    use bigdecimal::BigDecimal;
    use octo_budget_lib::auth_token::{AuthToken, UserId};
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

        let request = srv.get("/record-detail/").jwt_auth(user.id).send();

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

    #[test]
    fn create_happy_path() {
        let session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());

        let payload = json!({
            "amount": {"amount": 999.12, "currency": { "code": "CAD", "name": "Canadian Dollar" }},
            "transaction_type": "EXP",
            "tags": ["foo"],
        });

        let request = srv
            .post("/record-detail/")
            .jwt_auth(user.id)
            .send_json(&payload);

        let mut response = srv.block_on(request).expect("failed to send request");

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");

        let response_body = srv
            .block_on(response.json::<Value>())
            .expect("failed to parse response");

        // make sure that record was created properly
        let new_record_id = response_body.get("id").unwrap().as_i64().unwrap() as i32;
        let updated_record = session.find_record(new_record_id);

        assert_eq!(BigDecimal::from(999.12), updated_record.amount);
        assert_eq!("EXP", updated_record.transaction_type);
        assert_eq!(vec!["foo"], updated_record.tags);
    }

    #[test]
    fn update_happy_path() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());
        let record = session.create_record2(user.id);

        let payload = json!({
            "amount": {"amount": 999, "currency": { "code": "CAD", "name": "Canadian Dollar" }},
            "transaction_type": "INC",
            "tags": ["foo"],
        });

        let url = srv.url(format!("/record-detail/{}", record.id).as_ref());
        let request = srv
            .request(Method::PUT, url)
            .jwt_auth(user.id)
            .send_json(&payload);

        let mut response = srv.block_on(request).expect("failed to send request");

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");

        let response_body = srv
            .block_on(response.json::<Value>())
            .expect("failed to parse response");

        assert_eq!(json!(""), response_body);

        // make sure that record was updated
        let updated_record = session.find_record(record.id);

        assert_eq!(BigDecimal::from(999), updated_record.amount);
        assert_eq!("INC", updated_record.transaction_type);
        assert_eq!(vec!["foo"], updated_record.tags);
    }

    #[test]
    fn index_requires_auth() {
        let mut srv = setup();

        let request = srv.get("/record-detail/").send();

        let response = srv.block_on(request).expect("failed to send request");

        assert_eq!(StatusCode::UNAUTHORIZED, response.status(), "wrong status code");
    }

    #[test]
    fn update_requires_auth() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());
        let record = session.create_record2(user.id);

        let payload = json!({
            "amount": {"amount": 999, "currency": { "code": "CAD", "name": "Canadian Dollar" }},
            "transaction_type": "INC",
            "tags": ["foo"],
        });

        let url = srv.url(format!("/record-detail/{}", record.id).as_ref());
        let request = srv
            .request(Method::PUT, url)
            .send_json(&payload);

        let response = srv.block_on(request).expect("failed to send request");

        assert_eq!(StatusCode::UNAUTHORIZED, response.status(), "wrong status code");
    }

    #[test]
    fn create_requires_auth() {
        let mut srv = setup();

        let payload = json!({
            "amount": {"amount": 999.12, "currency": { "code": "CAD", "name": "Canadian Dollar" }},
            "transaction_type": "EXP",
            "tags": ["foo"],
        });

        let request = srv
            .post("/record-detail/")
            .send_json(&payload);

        let response = srv.block_on(request).expect("failed to send request");

        assert_eq!(StatusCode::UNAUTHORIZED, response.status(), "wrong status code");
    }
}
