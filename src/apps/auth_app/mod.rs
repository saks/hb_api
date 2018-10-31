use std::convert::Into;

use actix_web::middleware::Logger;
use actix_web::{App, AsyncResponder, FutureResponse, HttpResponse, Json, State};
use futures::{future, future::Future};

use crate::apps::AppState;
use crate::db::auth::FindUserMessage;

mod auth_error;
mod auth_form;
mod response_data;
mod utils;

use self::auth_form::AuthForm;
use self::response_data::ResponseData;
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

pub fn build() -> App<AppState> {
    App::with_state(AppState::new())
        .prefix("/auth/jwt")
        .middleware(Logger::default())
        .resource("/create/", |r| {
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
    use std::str;

    fn setup() {
        dotenv().ok().expect("Failed to parse .env file");
    }

    #[test]
    fn test_validation() {
        setup();

        let mut srv = TestServer::with_factory(build);

        let request = ClientRequest::build()
            .method(Method::POST)
            .uri(&srv.url("/auth/jwt/create/"))
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

        let mut srv = TestServer::with_factory(build);

        let request = ClientRequest::build()
            .method(Method::POST)
            .uri(&srv.url("/auth/jwt/create/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::BAD_REQUEST, response.status());

        let bytes = srv.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();

        assert_eq!(body, "");
    }
}
