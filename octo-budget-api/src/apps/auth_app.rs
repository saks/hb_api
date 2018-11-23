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
pub use self::response_data::ResponseData;
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
mod test {
    use super::*;
    use actix_web::{
        client::ClientRequest,
        http::{Method, StatusCode},
        test::TestServer,
        HttpMessage,
    };
    use dotenv::dotenv;
    use serde_json::json;
    use std::str;

    fn test_server() -> TestServer {
        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.resource("/create/", |r| r.post().with(create));
        })
    }

    fn setup() {
        dotenv().ok().expect("Failed to parse .env file");
    }

    #[test]
    fn test_validation() {
        setup();

        let mut srv = test_server();

        let request = ClientRequest::build()
            .method(Method::POST)
            .uri(&srv.url("/create/"))
            .json(json!({"username":"bar","password": ""}))
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

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
        setup();

        let mut srv = test_server();

        let request = ClientRequest::build()
            .method(Method::POST)
            .uri(&srv.url("/create/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::BAD_REQUEST, response.status());

        let bytes = srv.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();

        assert_eq!(body, "");
    }
}
