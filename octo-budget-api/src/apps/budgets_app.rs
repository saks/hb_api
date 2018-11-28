use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Query, Scope, State};
use futures::{future, future::Future};

use crate::apps::{middlewares::auth_by_token::VerifyAuthToken, AppState};
use octo_budget_lib::auth_token::AuthToken;

// mod db;

use super::index_params::Params;
use super::index_response::Data;
use crate::db::models::Budget;

type ResponseData = Data<Budget>;

fn auth_error_response() -> FutureResponse<HttpResponse> {
    Box::new(future::ok(HttpResponse::Unauthorized().finish()))
}

fn index(
    (_state, _request): (State<AppState>, HttpRequest<AppState>),
) -> FutureResponse<HttpResponse> {
    let _token: AuthToken = match _request.extensions_mut().remove() {
        Some(token) => token,
        _ => return auth_error_response(),
    };

    Box::new(future::ok(HttpResponse::Unauthorized().finish()))
    // let params = query_params.into_inner();
    //
    // match params.validate() {
    //     Ok(Params { page, per_page }) => {
    //         let user_id = token.user_id;
    //
    //         let message = GetRecordsMessage {
    //             page,
    //             per_page,
    //             user_id,
    //         };
    //
    //         state
    //             .db
    //             .send(message)
    //             .from_err()
    //             .and_then(|result| {
    //                 result
    //                     .map(|data| HttpResponse::Ok().json(data))
    //                     .map_err(|e| e.into())
    //             })
    //             .responder()
    //     }
    //     Err(response_data) => Box::new(future::ok(HttpResponse::BadRequest().json(response_data))),
    // }
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .middleware(VerifyAuthToken::new())
        .resource("/budget-detail", |r| r.get().with(index))
}
