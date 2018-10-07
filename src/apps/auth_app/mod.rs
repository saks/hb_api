use std::convert::Into;

use actix_web::middleware::Logger;
use actix_web::{App, AsyncResponder, FutureResponse, HttpResponse, Json, State};
use futures::{future, future::Future};

use apps::AppState;
use db::auth::Username;

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
            .send(Username(username))
            .from_err()
            .and_then(validate_user)
            .and_then(|user| validate_password(user, password).map_err(Into::into))
            .and_then(|user| Ok(generate_token(user)))
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
    // use super::*;
    // use actix_web::{http, test};

    // #[test]
    // fn test_create() {
    //     let state = AppState::new();
    //     let resp = test::TestRequest::with_state(state)
    //         .run_async(&create)
    //         .unwrap();
    //     assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    // }
}
