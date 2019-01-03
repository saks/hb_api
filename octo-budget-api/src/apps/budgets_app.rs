use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Query, Scope, State};
use futures::{future, future::Future};

use crate::apps::{middlewares::auth_by_token::VerifyAuthToken, AppState};

mod db;

use self::db::GetBudgetsMessage;
use super::index_params::Params;
use super::index_response::Data;
use crate::db::models::SerializedBudget;

type ResponseData = Data<SerializedBudget>;

fn index(
    (query_params, state, request): (Query<Params>, State<AppState>, HttpRequest<AppState>),
) -> FutureResponse<HttpResponse> {
    let token = crate::auth_token_from_request!(request);

    let params = query_params.into_inner();

    let validation_result: Result<Params, ResponseData> = params.validate();
    match validation_result {
        Ok(Params { page, per_page }) => {
            let user_id = token.user_id;

            let message = GetBudgetsMessage {
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
        .resource("/budget-detail/", |r| r.get().with(index))
}
