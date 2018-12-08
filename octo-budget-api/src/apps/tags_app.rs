use actix_web::{
    AsyncResponder, Error as WebError, FutureResponse, HttpRequest, HttpResponse, Scope, State,
};
use futures::{future, future::Future};
use serde_derive::Serialize;

use crate::apps::{middlewares::auth_by_token::VerifyAuthToken, AppState};
use octo_budget_lib::auth_token::AuthToken;

mod tags;

#[derive(Serialize, Default)]
pub struct ResponseData {
    tags: Vec<String>,
}

fn show((state, req): (State<AppState>, HttpRequest<AppState>)) -> FutureResponse<HttpResponse> {
    let token = match req.extensions_mut().remove::<AuthToken>() {
        Some(token) => token,
        _ => return Box::new(future::ok(HttpResponse::Unauthorized().finish())),
    };

    let get_redis_tags = state
        .redis
        .clone()
        .send(tags::get_ordered_tags_from_redis_msg(token.user_id));

    let get_user_tags = state
        .db
        .send(tags::get_user_tags_from_db_msg(token.user_id));

    get_redis_tags
        .join(get_user_tags)
        .map_err(WebError::from)
        .and_then(|res| Ok(tags::get_ordered_tags(res)?))
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("", |r| r.get().with(show))
}

#[cfg(test)]
mod test {
    // use super::*;
    // use actix_web::{client::ClientRequest, http::StatusCode, test::TestServer};

    // fn setup() -> TestServer {
    //     use crate::apps::middlewares::auth_by_token::VerifyAuthToken;
    //     use dotenv::dotenv;
    //
    //     dotenv().ok().expect("Failed to parse .env file");
    //
    //     TestServer::build_with_state(|| AppState::new()).start(|app| {
    //         app.resource("/api/records/record-detail/", |r| r.get().with(show));
    //     })
    // }

    // fn make_token(hours_from_now: i64) -> String {
    //     use crate::config;
    //     AuthToken::new(123, config::auth_token_secret())
    //         .expire_in_hours(hours_from_now)
    //         .to_string()
    // }
    //
    // #[test]
    // fn get_empty_list_of_records() {}

    // #[test]
    // fn test_auth_required_for_records_app() {
    //     setup();
    //
    //     let mut srv = test_server();
    //
    //     let request = ClientRequest::build()
    //         .uri(&srv.url("/api/records/record-detail/"))
    //         .finish()
    //         .unwrap();
    //
    //     let response = srv.execute(request.send()).unwrap();
    //
    //     assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    // }

    // #[test]
    // fn test_auth_success_for_records_app() {
    //     setup();
    //
    //     let mut srv = test_server();
    //     let token = format!("JWT {}", make_token(12));
    //
    //     let request = ClientRequest::build()
    //         .header("Authorization", token)
    //         .uri(&srv.url("/api/records/record-detail/"))
    //         .finish()
    //         .unwrap();
    //
    //     let response = srv.execute(request.send()).unwrap();
    //
    //     assert_eq!(StatusCode::OK, response.status());
    //     // TODO: check body
    // }
}
