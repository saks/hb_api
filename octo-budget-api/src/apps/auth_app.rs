use std::convert::Into;

use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, Scope, State};
use futures::{future, future::Future};

use crate::apps::AppState;
use crate::db::auth::FindUserMessage;

mod auth_error;
mod auth_form;
mod response_data;
mod utils;

use self::auth_form::AuthForm;
pub use self::response_data::Data;
use self::utils::{generate_token, validate_password, validate_user};

fn create((form_json, state): (Json<AuthForm>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let form = form_json.into_inner();

    match form.validate() {
        Ok((username, password)) => state
            .db
            .send(FindUserMessage(username))
            .from_err()
            .and_then(validate_user)
            .and_then(|user| validate_password(user, password).map_err(Into::into))
            .and_then(|user| Ok(generate_token(&user)))
            .and_then(|response_data| Ok(HttpResponse::Ok().json(response_data)))
            .responder(),
        Err(response_data) => Box::new(future::ok(HttpResponse::BadRequest().json(response_data))),
    }
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope.resource("/create/", |r| {
        r.post().with_config(create, |((cfg, _),)| {
            cfg.limit(1024); // <- limit size of the payload
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config, tests::DbSession};
    use actix_web::{
        client::{ClientRequest, ClientResponse},
        http::{Method, StatusCode},
        test::TestServer,
        HttpMessage,
    };
    use dotenv::dotenv;
    use serde_json::json;
    use std::str;

    fn setup() -> TestServer {
        dotenv().ok().expect("Failed to parse .env file");
        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.resource("/create/", |r| r.post().with_async(create));
        })
    }

    fn response_json(srv: &mut TestServer, response: ClientResponse) -> serde_json::Value {
        let bytes = srv.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
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

        let bytes = srv.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();

        assert_eq!(
            body,
            json!({"password":["This field may not be blank."]}).to_string()
        );
    }

    #[test]
    fn test_not_json_body() {
        let mut srv = setup();

        let response = request_new_token(&mut srv, json!(""));

        assert_eq!(StatusCode::BAD_REQUEST, response.status());

        let bytes = srv.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();

        assert_eq!(body, "");
    }

    #[test]
    fn test_ok_auth_response() {
        let mut srv = setup();
        let mut session = DbSession::new();

        let user = session.create_user("ok auth user", "dummy password");

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
        let mut session = DbSession::new();

        let user = session.create_user("ok auth user", "dummy password");

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
        let mut session = DbSession::new();

        let user = session.create_user("bad pass user", "dummy password");

        let response = request_new_token(
            &mut srv,
            json!({ "username": user.username, "password": "wrong password" }),
        );

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn test_invalid_password_response_body() {
        let mut srv = setup();
        let mut session = DbSession::new();

        let user = session.create_user("bad pass user", "dummy password");

        let response = request_new_token(
            &mut srv,
            json!({ "username": user.username, "password": "wrong password" }),
        );

        let body_json = response_json(&mut srv, response);

        let expected = json!({"non_field_errors":["Unable to log in with provided credentials."]});
        assert_eq!(expected, body_json);
    }
}
