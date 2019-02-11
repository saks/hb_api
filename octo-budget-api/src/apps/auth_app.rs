use actix_web::{HttpResponse, Json, Responder, Result as WebResult};
use actix_web_async_await::{await, compat};

use crate::apps::{Scope, State};
use crate::db::messages::FindUserByName;

mod response_data;
mod utils;

use self::utils::generate_token;
use super::forms::auth::Form;

async fn create((form, state): (Json<Form>, State)) -> WebResult<impl Responder> {
    let data = form.into_inner().validate()?;

    let user = await!(state.db.send(FindUserByName(data.username)))??;
    Form::validate_password(&user, &data.password)?;

    Ok(HttpResponse::Ok().json(generate_token(&user)))
}

pub fn scope(scope: Scope) -> Scope {
    scope.resource("/create/", |r| {
        r.post().with_config(compat(create), |((cfg, _),)| {
            cfg.limit(1024); // <- limit size of the payload
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::builders::UserBuilder;
    use crate::{assert_response_body_eq, config, tests};
    use actix_web::{
        client::{ClientRequest, ClientResponse},
        http::{Method, StatusCode},
        test::TestServer,
    };
    use serde_json::json;

    fn setup() -> TestServer {
        tests::setup_env();
        setup_test_server()
    }

    fn setup_test_server() -> TestServer {
        use crate::apps::AppState;

        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.resource("/create/", |r| r.post().with(compat(create)));
        })
    }

    fn response_json(srv: &mut TestServer, response: ClientResponse) -> serde_json::Value {
        use actix_web::HttpMessage;

        let bytes = srv.execute(response.body()).unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        let body_json: serde_json::Value = serde_json::from_str(&body).unwrap();

        body_json
    }

    fn request_new_token(srv: &mut TestServer, body: serde_json::Value) -> ClientResponse {
        let request = ClientRequest::build()
            .method(Method::POST)
            .uri(&srv.url("/create/"))
            .json(body)
            .unwrap();

        srv.execute(request.send()).unwrap()
    }

    #[test]
    fn test_validation() {
        let mut srv = setup();
        let response = request_new_token(&mut srv, json!({"username":"bar","password": ""}));

        assert_eq!(StatusCode::BAD_REQUEST, response.status());
        assert_response_body_eq!(
            srv,
            response,
            r#"{"password":["This field may not be blank."]}"#
        );
    }

    #[test]
    fn test_not_json_body() {
        let mut srv = setup();

        let response = request_new_token(&mut srv, json!(""));

        assert_eq!(StatusCode::BAD_REQUEST, response.status());
        assert_response_body_eq!(srv, response, "");
    }

    #[test]
    fn test_ok_auth_response() {
        let mut srv = setup();
        let mut session = tests::DbSession::new();

        let user = session.create_user(
            UserBuilder::default()
                .username("ok auth user")
                .password("dummy password"),
        );

        let response = request_new_token(
            &mut srv,
            json!({ "username": user.username, "password": "dummy password" }),
        );

        assert_eq!(StatusCode::OK, response.status());
    }

    #[test]
    fn test_ok_auth_token() {
        use octo_budget_lib::auth_token::AuthToken;

        let mut srv = setup();
        let mut session = tests::DbSession::new();

        let user = session.create_user(
            UserBuilder::default()
                .username("ok auth user")
                .password("dummy password"),
        );

        let response = request_new_token(
            &mut srv,
            json!({ "username": user.username, "password": "dummy password" }),
        );

        let body_json = response_json(&mut srv, response);
        let token_string = body_json.get("token").unwrap().as_str().unwrap();

        // returned token is valid
        let token = AuthToken::from(&token_string, config::AUTH_TOKEN_SECRET.as_bytes()).unwrap();

        assert_eq!(user.id, token.user_id);
    }

    #[test]
    fn test_invalid_password_response() {
        let mut srv = setup();
        let mut session = tests::DbSession::new();

        let user = session.create_user(
            UserBuilder::default()
                .username("bad pass user")
                .password("dummy password"),
        );

        let response = request_new_token(
            &mut srv,
            json!({ "username": user.username, "password": "wrong password" }),
        );

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn test_invalid_password_response_body() {
        let mut srv = setup();
        let mut session = tests::DbSession::new();

        let user = session.create_user(
            UserBuilder::default()
                .username("bad pass user")
                .password("dummy password"),
        );

        let response = request_new_token(
            &mut srv,
            json!({ "username": user.username, "password": "wrong password" }),
        );

        assert_response_body_eq!(
            srv,
            response,
            r#"{"non_field_errors":["Unable to log in with provided credentials."]}"#
        );
    }
}
