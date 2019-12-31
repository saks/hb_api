use super::response_data::Data as ResponseData;
use super::*;
use crate::{
    db::builders::UserBuilder,
    test_server2,
    tests::{setup_env, DbSession},
};
use actix_http_test::TestServerRuntime;
use actix_web::http::{header, Method, StatusCode};
use actix_web::test::{call_service, read_body, TestRequest};
use serde_json::{json, Value};
use service::Service;

// fn setup() -> TestServerRuntime {
//     setup_env();
//     // test_server2!(Service)
// }

#[actix_rt::test]
async fn empty_params_sent() {
    setup_env();
    let srv = actix_web::test::init_service(
        actix_web::App::new()
            .data(crate::db::start())
            .data(octo_budget_lib::auth_token::ApiJwtTokenAuthConfig::new(
                crate::config::AUTH_TOKEN_SECRET.as_bytes(),
            ))
            .service(Service),
    )
    .await;

    let req = TestRequest::with_uri("/create/")
        .method(Method::POST)
        .header(header::CONTENT_TYPE, "application/json")
        .set_json(&json!({"username": "", "password": ""}))
        .to_request();

    let response = call_service(&mut srv, req).await;
    assert_eq!(
        StatusCode::BAD_REQUEST,
        response.status(),
        "unexpected response code"
    );

    let response_body = serde_json::from_slice::<Value>(read_body(response).await.as_ref())
        .expect("failed to parse response");
    assert_eq!(
        json!({
            "password":["This field may not be blank."],
            "username":["This field may not be blank."],
        }),
        response_body
    );
}

// #[test]
// fn test_not_json_body() {
//     let mut srv = setup();
//
//     let request = srv
//         .post("/create/")
//         .header(header::CONTENT_TYPE, "application/json")
//         .send_json(&json!(""));
//
//     let mut response = srv.block_on(request).expect("failed to send request");
//     assert_eq!(
//         StatusCode::BAD_REQUEST,
//         response.status(),
//         "unexpected response code"
//     );
//
//     let response_body = srv
//         .block_on(response.body())
//         .expect("failed to parse response");
//
//     let expected_body = bytes::Bytes::from_static(b"Json deserialize error: invalid type: string \"\", expected struct Form at line 1 column 2");
//     assert_eq!(expected_body, response_body);
// }
//
// #[test]
// fn test_ok_auth_response() {
//     let session = DbSession::new();
//     let user = session.create_user(
//         UserBuilder::default()
//             .username("ok auth user")
//             .password("dummy password"),
//     );
//
//     let mut srv = setup();
//
//     let request = srv
//         .post("/create/")
//         .header(header::CONTENT_TYPE, "application/json")
//         .send_json(&json!({"username": user.username, "password": "dummy password"}));
//
//     let mut response = srv.block_on(request).expect("failed to send request");
//     assert!(response.status().is_success(), "response is not success");
//
//     srv.block_on(response.json::<ResponseData>())
//         .expect("failed to parse response");
// }
//
// #[test]
// fn no_params_sent() {
//     let mut srv = setup();
//
//     let request = srv
//         .post("/create/")
//         .header(header::CONTENT_TYPE, "application/json")
//         .send_json(&json!({}));
//
//     let mut response = srv.block_on(request).expect("failed to send request");
//     assert_eq!(
//         StatusCode::BAD_REQUEST,
//         response.status(),
//         "unexpected response code"
//     );
//
//     let response_body = srv
//         .block_on(response.json::<Value>())
//         .expect("failed to parse response");
//
//     assert_eq!(
//         json!({"password": ["This field is required."], "username": ["This field is required."]}),
//         response_body
//     );
// }
