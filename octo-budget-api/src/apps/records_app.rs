use actix_web::{
    get, post, put,
    web::{self, Json, Path, Query},
    HttpResponse, Result,
};
use octo_budget_lib::auth_token::UserId;

use super::forms::record::Form;
use super::index_params::Params;
use crate::db::{
    queries::{CreateRecord, FindRecord, GetRecords, UpdateRecord},
    ConnectionPool,
};
use crate::redis::{
    helpers::{decrement_tags, increment_tags},
    Redis,
};

#[get("/record-detail/")]
async fn index(
    user_id: UserId,
    params: Query<Params>,
    pool: web::Data<ConnectionPool>,
) -> Result<HttpResponse> {
    let params = params.into_inner().validate()?;

    let message = GetRecords {
        page: params.page,
        per_page: params.per_page,
        user_id: user_id.into(),
    };

    let records = pool.execute(message).await?;

    Ok(HttpResponse::Ok().json(records))
}

#[post("/record-detail/")]
async fn create(
    user_id: UserId,
    form: Json<Form>,
    pool: web::Data<ConnectionPool>,
    redis: web::Data<Redis>,
) -> Result<HttpResponse> {
    use serde_json::json;

    let data = form.into_inner().validate()?;
    let id = pool.execute(CreateRecord::new(&data, user_id)).await?;

    increment_tags(user_id, data.tags, &redis).await?;

    Ok(HttpResponse::Ok().json(json!({ "id": id })))
}

#[put("/record-detail/{id}/")]
async fn update(
    user_id: UserId,
    record_id: Path<i32>,
    form: Json<Form>,
    pool: web::Data<ConnectionPool>,
    redis: web::Data<Redis>,
) -> Result<HttpResponse> {
    let record_id = record_id.into_inner();
    let data = form.into_inner().validate()?;

    let record = pool.execute(FindRecord::new(record_id, user_id)).await?;
    pool.execute(UpdateRecord::new(record.id, &data, user_id))
        .await?;

    decrement_tags(user_id, record.tags, &redis).await?;
    increment_tags(user_id, data.tags, &redis).await?;

    Ok(HttpResponse::Ok().json(""))
}

pub mod service {
    use super::*;
    use actix_web::dev::HttpServiceFactory;

    pub struct Service;

    impl HttpServiceFactory for Service {
        fn register(self, config: &mut actix_web::dev::AppService) {
            HttpServiceFactory::register(index, config);
            HttpServiceFactory::register(create, config);
            HttpServiceFactory::register(update, config);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::service::Service;
    use crate::{
        await_test_server,
        db::builders::UserBuilder,
        tests::{self, setup_env, RequestJwtAuthExt as _},
    };
    use actix_web::{
        http::{Method, StatusCode},
        test::{call_service, read_body, TestRequest},
    };
    use bigdecimal::BigDecimal;
    use serde_json::{json, Value};

    #[actix_rt::test]
    async fn index_when_no_records() {
        setup_env();

        let session = tests::DbSession::new();
        let mut service = await_test_server!(Service);

        let user = session.create_user(UserBuilder::default().tags(vec!["foo"]));
        let request = TestRequest::with_uri("/record-detail/")
            .jwt_auth(user.id)
            .to_request();

        let response = call_service(&mut service, request).await;

        assert!(response.status().is_success(), "response is not success");

        let response_body = read_body(response).await;
        let response_body = serde_json::from_slice::<Value>(&response_body)
            .expect(&format!("Failed to deserialize: {:?}", response_body));

        assert_eq!(
            json!({"total": 0, "results": [], "next": false, "previous": false}),
            response_body
        );
    }

    #[actix_rt::test]
    async fn index_requires_auth() {
        setup_env();

        let mut service = await_test_server!(Service);
        let request = TestRequest::with_uri("/record-detail/").to_request();
        let response = call_service(&mut service, request).await;

        assert_eq!(
            StatusCode::UNAUTHORIZED,
            response.status(),
            "wrong status code"
        );
    }

    #[actix_rt::test]
    async fn update_requires_auth() {
        setup_env();

        let mut service = await_test_server!(Service);
        let request = TestRequest::with_uri("/record-detail/123/")
            .method(Method::PUT)
            .to_request();
        let response = call_service(&mut service, request).await;

        assert_eq!(
            StatusCode::UNAUTHORIZED,
            response.status(),
            "wrong status code"
        );
    }

    #[actix_rt::test]
    async fn create_requires_auth() {
        setup_env();

        let mut service = await_test_server!(Service);
        let request = TestRequest::with_uri("/record-detail/")
            .method(Method::POST)
            .to_request();
        let response = call_service(&mut service, request).await;

        assert_eq!(
            StatusCode::UNAUTHORIZED,
            response.status(),
            "wrong status code"
        );
    }

    #[actix_rt::test]
    async fn create_happy_path() {
        setup_env();

        let session = tests::DbSession::new();
        let mut service = await_test_server!(Service);

        let user = session.create_user(UserBuilder::default());

        let payload = json!({
            "amount": {"amount": 999.12, "currency": { "code": "CAD", "name": "Canadian Dollar" }},
            "transaction_type": "EXP",
            "tags": ["foo", "bar"],
        });

        let request = TestRequest::with_uri("/record-detail/")
            .method(Method::POST)
            .jwt_auth(user.id)
            .set_json(&payload)
            .to_request();

        let response = call_service(&mut service, request).await;

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");

        let response_body = read_body(response).await;
        let response_body = serde_json::from_slice::<Value>(&response_body)
            .expect(&format!("Failed to deserialize: {:?}", response_body));

        // make sure that record was created properly
        let new_record_id = response_body.get("id").unwrap().as_i64().unwrap() as i32;
        let updated_record = session.find_record(new_record_id);

        assert_eq!(BigDecimal::from(999.12), updated_record.amount);
        assert_eq!("EXP", updated_record.transaction_type);
        assert_eq!(vec!["foo", "bar"], updated_record.tags);
    }

    #[actix_rt::test]
    async fn update_happy_path() {
        setup_env();

        let session = tests::DbSession::new();
        let mut service = await_test_server!(Service);

        let user = session.create_user(UserBuilder::default());
        let record = session.create_record2(user.id);

        let payload = json!({
            "amount": {"amount": 999, "currency": { "code": "CAD", "name": "Canadian Dollar" }},
            "transaction_type": "INC",
            "tags": ["foo"],
        });

        let request = TestRequest::with_uri(&format!("/record-detail/{}/", record.id))
            .method(Method::PUT)
            .set_json(&payload)
            .jwt_auth(user.id)
            .to_request();

        let response = call_service(&mut service, request).await;

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");

        let response_body = read_body(response).await;
        let response_body = serde_json::from_slice::<Value>(&response_body)
            .expect(&format!("Failed to deserialize: {:?}", response_body));

        assert_eq!(json!(""), response_body);

        // make sure that record was updated
        let updated_record = session.find_record(record.id);

        assert_eq!(BigDecimal::from(999), updated_record.amount);
        assert_eq!("INC", updated_record.transaction_type);
        assert_eq!(vec!["foo"], updated_record.tags);
    }
}
