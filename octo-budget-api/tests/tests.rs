// extern crate actix_web;
// extern crate chrono;
// extern crate diesel;
// extern crate dotenv;
// extern crate octo_budget_api;
// #[macro_use]
// extern crate serde_json;
// extern crate djangohashers;

// use actix_web::{
//     http::{Method, StatusCode},
//     test::TestServer,
//     HttpMessage,
// };
// use dotenv::dotenv;
// use octo_budget_api::apps::auth_app::build;
// use octo_budget_api::auth_token::AuthToken;
// use std::str;

// mod support;

// #[test]
// fn insert() {
//     use diesel::query_dsl::QueryDsl;
//     use diesel::*;
//     use octo_budget_api::db::schema::auth_user::dsl::*;
//
//     let mut session = support::Session::with_transaction();
//
//     session.create_user("ok_insert_user", "dummy password");
//
//     let count = auth_user.count().get_result(session.conn());
//     assert_eq!(Ok(1), count);
// }
//
// #[test]
// fn successful_authentication() {
//     dotenv().ok().expect("Failed to parse .env file");
//
//     let mut srv = TestServer::with_factory(build);
//     let mut session = support::Session::new();
//
//     let user = session.create_user("ok auth user", "dummy password");
//
//     let request = srv
//         .client(Method::POST, "/auth/jwt/create/")
//         .json(json!({ "username": user.username, "password": "dummy password" }))
//         .unwrap();
//
//     let response = srv.execute(request.send()).unwrap();
//
//     // response is success
//     assert_eq!(StatusCode::OK, response.status());
//
//     let bytes = srv.execute(response.body()).unwrap();
//     let body = str::from_utf8(&bytes).unwrap();
//     let body_json: serde_json::Value = serde_json::from_str(&body).unwrap();
//     let token_string = body_json.get("token").unwrap().as_str().unwrap();
//
//     // returned token is valid
//     let expected_token = AuthToken::new(user.id);
//     assert_eq!(Ok(expected_token), AuthToken::verify(&token_string));
// }
//
// #[test]
// fn authentication_with_wrong_password() {
//     dotenv().ok().expect("Failed to parse .env file");
//
//     let mut srv = TestServer::with_factory(build);
//     let mut session = support::Session::new();
//
//     let user = session.create_user("bad pass user", "dummy password");
//
//     let request = srv
//         .client(Method::POST, "/auth/jwt/create/")
//         .json(json!({ "username": user.username, "password": "wrong password" }))
//         .unwrap();
//
//     let response = srv.execute(request.send()).unwrap();
//
//     assert_eq!(StatusCode::UNAUTHORIZED, response.status());
//
//     let bytes = srv.execute(response.body()).unwrap();
//     let body = str::from_utf8(&bytes).unwrap();
//     let expected = r#"{"non_field_errors":["Unable to log in with provided credentials."]}"#;
//     assert_eq!(expected, body);
// }
