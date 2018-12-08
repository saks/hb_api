use actix_web::{
    AsyncResponder, Error as WebError, FutureResponse, HttpRequest, HttpResponse, Query, Scope,
    State,
};
use futures::{future, future::Future};

use crate::apps::{middlewares::auth_by_token::VerifyAuthToken, AppState};
use octo_budget_lib::auth_token::AuthToken;

mod tags;
// mod db;

// use self::db::GetRecordsMessage;
use super::index_params::Params;
// use super::index_response::Data;
// use crate::db::models::Record as RecordModel;

// type ResponseData = Data<RecordModel>;

fn auth_error_response() -> FutureResponse<HttpResponse> {
    Box::new(future::ok(HttpResponse::Unauthorized().finish()))
}

fn show(
    (_query_params, state, request): (Query<Params>, State<AppState>, HttpRequest<AppState>),
) -> FutureResponse<HttpResponse> {
    let token: AuthToken = match request.extensions_mut().remove() {
        Some(token) => token,
        _ => return auth_error_response(),
    };

    let get_redis_tags = state
        .redis
        .clone()
        .send(tags::get_ordered_tags_from_redis_msg(&token));

    let get_user_tags = state.db.send(tags::get_user_tags_from_db_msg(&token));
    let mut user_tags: Vec<String> = vec![];

    get_user_tags
        .and_then(|user_tags_result| {
            // println!("user tags result: {:?}", user_tags_result);
            // XXX: handle user not found error: Err(DeserializationError(UnexpectedNullError))
            // user_tags = user_tags_result.unwrap_or_else(|_| vec![]);
            get_redis_tags
        })
        .map_err(WebError::from)
        .and_then(move |_redis_tags_result| {
            // println!("redis tags result: {:?}", redis_tags_result);
            // tags::get_ordered_tags(user_tags, _redis_tags_result)
            let x: Vec<String> = vec![];
            Ok(x)
        })
        .and_then(|tags| {
            println!("tags result: {:?}", tags);
            Ok(HttpResponse::Ok().json(tags))
        })
        .responder()

    // state
    //     .redis
    //     .clone()
    //     .send(tags::get_ordered_tags_from_redis_msg(&token))
    //     .map_err(WebError::from)
    //     .and_then(tags::get_ordered_tags)
    //     .and_then(|res| {
    //         println!("XXX: {:?}", res);
    //         //
    //         Ok(HttpResponse::Ok().json("[1,2,3]"))
    //     })
    //     .responder()
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .middleware(VerifyAuthToken::new())
        .resource("/{user_id}/", |r| r.get().with(show))
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
