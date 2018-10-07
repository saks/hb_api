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
            .and_then(|result| validate_password(result, password))
            .and_then(generate_token)
            .responder(),
        Err(data) => Box::new(future::ok(HttpResponse::BadRequest().json(data))),
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

// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn test_create_token() {
//         use frank_jwt::{decode, validate_signature, Algorithm};
//         use std::env;
//
//         let secret = "foo".to_string();
//         env::set_var("AUTH_TOKEN_SECRET", &secret);
//
//         let token = create_token(123);
//
//         assert_eq!(125, token.len());
//
//         let data = decode(&token, &secret, Algorithm::HS256).unwrap();
//         let (_header, data) = data;
//
//         assert_eq!(123, data["user_id"]);
//
//         let validation_result = validate_signature(&token, &secret, Algorithm::HS256);
//         assert_eq!(Ok(true), validation_result);
//
//         env::remove_var("AUTH_TOKEN_SECRET");
//     }
//
//     //     fn make_user(password_hash: &'static str) -> UserModel {
//     //         UserModel {
//     //             id: 123,
//     //             username: "".to_string(),
//     //             password: password_hash.to_string(),
//     //             email: "".to_string(),
//     //             is_active: true,
//     //         }
//     //     }
//     //
//     //     fn make_creds(password: &'static str) -> Credentials {
//     //         Credentials {
//     //             username: "foo".to_string(),
//     //             password: password.to_string(),
//     //         }
//     //     }
//     //
//     //     #[test]
//     //     fn test_authenticate_user_success() {
//     //         let user = make_user(
//     //             "pbkdf2_sha256$100000$Nk15JZg3MdZa$BKvnIMgDEAH1B6/ns9xw9PdQNP8Fq8rSHnrZ+8l0xCo=",
//     //         );
//     //         let find_result = Ok((Some(user.clone()), make_creds("zxcasdqwe123")));
//     //
//     //         let response = authenticate_user(find_result);
//     //         let expected_response = HttpResponse::Ok().json(json!({"token":create_token(user.id)}));
//     //
//     //         assert_eq!(expected_response.body(), response.unwrap().body());
//     //     }
//     //
//     //     #[test]
//     //     fn test_authenticate_user_no_user_found() {
//     //         let response = authenticate_user(Ok((None, make_creds(""))));
//     //         let expeted_response = HttpResponse::Ok()
//     //             .json(json!({"non_field_errors":["Unable to log in with provided credentials."]}));
//     //
//     //         assert_eq!(expeted_response.body(), response.unwrap().body());
//     //     }
//     //
//     //     #[test]
//     //     fn test_authenticate_user_invalid_password() {
//     //         let user = make_user(
//     //             "pbkdf2_sha256$100000$Nk15JZg3MdZa$BKvnIMgDEAH1B6/ns9xw9PdQNP8Fq8rSHnrZ+8l0xCo=",
//     //         );
//     //         let response = authenticate_user(Ok((Some(user.clone()), make_creds("foo"))));
//     //         let expected_response = HttpResponse::Ok()
//     //             .json(json!({"non_field_errors":["Unable to log in with provided credentials."]}));
//     //
//     //         assert_eq!(expected_response.body(), response.unwrap().body());
//     //     }
//     //
//     //     // TODO
//     //     #[test]
//     //     fn test_auth_form_no_password() {
//     //         let form = AuthForm {
//     //             username: Some("foo".to_string()),
//     //             password: None,
//     //         };
//     //         let result = form.validate();
//     //         let expected_result = ResponseData { password_errors: vec![], ...ResponseData::default() };
//     //     }
// }
