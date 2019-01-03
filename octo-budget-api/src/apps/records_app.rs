use actix_web::{AsyncResponder, HttpResponse, Query, Scope};
use futures::{future, future::Future};

use crate::apps::{
    middlewares::auth_by_token::VerifyAuthToken, AppState, Request, Response, State,
};

mod db;

use self::db::GetRecordsMessage;
use super::index_params::Params;
use super::index_response::Data;
use crate::db::models::Record as RecordModel;

type ResponseData = Data<RecordModel>;

fn index((query_params, state, request): (Query<Params>, State, Request)) -> Response {
    let token = crate::auth_token_from_request!(request);
    let params = query_params.into_inner();

    let validation_result: Result<Params, ResponseData> = params.validate();
    match validation_result {
        Ok(Params { page, per_page }) => {
            let user_id = token.user_id;

            let message = GetRecordsMessage {
                page,
                per_page,
                user_id,
            };

            state
                .db
                .send(message)
                .from_err()
                .and_then(|result| {
                    result
                        .map(|data| HttpResponse::Ok().json(data))
                        .map_err(|e| e.into())
                })
                .responder()
        }
        Err(response_data) => Box::new(future::ok(HttpResponse::BadRequest().json(response_data))),
    }
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("/record-detail/", |r| r.get().with(index))
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use actix_web::{client::ClientRequest, http::StatusCode, test::TestServer};

    // fn setup() -> TestServer {
    //     use crate::apps::middlewares::auth_by_token::VerifyAuthToken;
    //     use dotenv::dotenv;
    //
    //     dotenv().ok().expect("Failed to parse .env file");
    //
    //     TestServer::build_with_state(|| AppState::new()).start(|app| {
    //         app.resource("/api/records/record-detail/", |r| r.get().with(index));
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
