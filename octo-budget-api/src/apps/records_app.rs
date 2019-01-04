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
    use super::*;
    use crate::{assert_response_body_eq, db::builders::UserBuilder, tests};
    use actix_web::{client::ClientRequest, http::StatusCode, test::TestServer};

    fn setup() -> TestServer {
        tests::setup_env();
        setup_test_server()
    }

    fn setup_test_server() -> TestServer {
        use crate::apps::middlewares::auth_by_token::VerifyAuthToken;

        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.middleware(VerifyAuthToken::default())
                .resource("/api/records/record-detail/", |r| r.get().with(index));
        })
    }

    #[test]
    fn auth_required_for_records_app() {
        let mut srv = setup();

        let request = ClientRequest::build()
            .uri(&srv.url("/api/records/record-detail/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn empty_list_of_records() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default().tags(vec!["foo"]));
        let request = tests::authenticated_request(&user, srv.url("/api/records/record-detail/"));
        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");
        assert_response_body_eq!(
            srv,
            response,
            r#"{"total":0,"results":[],"next":false,"previous":false}"#
        );
    }
}
