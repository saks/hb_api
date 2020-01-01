use super::response_data::Data as ResponseData;
use super::*;
use crate::{
    await_test_server,
    db::builders::UserBuilder,
    tests::{setup_env, DbSession},
};
use actix_web::http::{header, Method, StatusCode};
use actix_web::test::{call_service, read_response, read_response_json, TestRequest};
use bytes::Bytes;
use serde_json::{json, Value};
use service::Service;

fn login_request(body: Value) -> actix_http::Request {
    TestRequest::with_uri("/create/")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .set_json(&body)
        .to_request()
}

#[actix_rt::test]
async fn bad_request_returned_when_empty_params_sent() {
    setup_env();

    let mut service = await_test_server!(Service);
    let request = login_request(json!({"username": "", "password": "" }));
    let response = call_service(&mut service, request).await;

    assert_eq!(
        StatusCode::BAD_REQUEST,
        response.status(),
        "unexpected response code"
    );
}

#[actix_rt::test]
async fn errors_returned_when_empty_params_sent() {
    setup_env();

    let mut service = await_test_server!(Service);
    let request = login_request(json!({"username": "", "password": "" }));
    let body: Value = read_response_json(&mut service, request).await;

    assert_eq!(
        json!({
            "password":["This field may not be blank."],
            "username":["This field may not be blank."],
        }),
        body
    );
}

#[actix_rt::test]
async fn response_code_when_not_json_body() {
    setup_env();

    let mut service = await_test_server!(Service);
    let request = login_request(json!(""));
    let response = call_service(&mut service, request).await;

    assert_eq!(
        StatusCode::BAD_REQUEST,
        response.status(),
        "unexpected response code"
    );
}

#[actix_rt::test]
async fn response_body_when_not_json_body() {
    setup_env();

    let mut service = await_test_server!(Service);
    let request = login_request(json!(""));
    let body = read_response(&mut service, request).await;

    assert_eq!(Bytes::from_static(b""), body);
}

#[actix_rt::test]
async fn responds_400_when_no_params_sent() {
    setup_env();

    let mut service = await_test_server!(Service);
    let request = login_request(json!({}));
    let response = call_service(&mut service, request).await;

    assert_eq!(
        StatusCode::BAD_REQUEST,
        response.status(),
        "unexpected response code"
    );
}

#[actix_rt::test]
async fn responds_with_errors_when_no_params_sent() {
    setup_env();

    let mut service = await_test_server!(Service);
    let request = login_request(json!({}));
    let body: Value = read_response_json(&mut service, request).await;

    assert_eq!(
        json!({"password": ["This field is required."], "username": ["This field is required."]}),
        body
    );
}

#[actix_rt::test]
async fn when_ok_auth_response_code_success() {
    let session = DbSession::new();
    let user = session.create_user(
        UserBuilder::default()
            .username("ok auth user")
            .password("dummy password"),
    );

    setup_env();

    let mut service = await_test_server!(Service);
    let request = login_request(json!({"username": user.username, "password": "dummy password"}));
    let response = call_service(&mut service, request).await;

    assert!(response.status().is_success(), "response is not success");
}

#[actix_rt::test]
async fn when_ok_auth_response_body_correct() {
    let session = DbSession::new();
    let user = session.create_user(
        UserBuilder::default()
            .username("ok auth user")
            .password("dummy password"),
    );

    setup_env();

    let mut service = await_test_server!(Service);
    let request = login_request(json!({"username": user.username, "password": "dummy password"}));
    let _data: ResponseData = read_response_json(&mut service, request).await;

    // TODO: check data content
}
