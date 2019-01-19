use actix_web::{HttpResponse, Query, Responder, Result, Scope};
use actix_web_async_await::{await, compat};

use crate::apps::index_params::Params;
use crate::apps::{middlewares::VerifyAuthToken, AppState, Request, State};
use crate::db::messages::GetBudgets;

async fn index((params, state, req): (Query<Params>, State, Request)) -> Result<impl Responder> {
    let token = crate::auth_token_from_async_request!(req);
    let params = params.into_inner().validate()?;

    let message = GetBudgets {
        page: params.page,
        per_page: params.per_page,
        user_id: token.user_id,
    };

    let result = await!(state.db.send(message))??;

    Ok(HttpResponse::Ok().json(result))
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("/budget-detail/", |r| r.get().with(compat(index)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_response_body_eq, db::builders::UserBuilder, tests};
    use actix_web::{client::ClientRequest, http::StatusCode, test::TestServer};

    fn setup_test_server() -> TestServer {
        use crate::apps::middlewares::VerifyAuthToken;

        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.middleware(VerifyAuthToken::default())
                .resource("/budget-detail/", |r| r.get().with(compat(index)));
        })
    }

    fn setup() -> TestServer {
        tests::setup_env();
        setup_test_server()
    }

    #[test]
    fn test_auth_required_for_index() {
        let mut srv = setup();

        let request = ClientRequest::build()
            .uri(&srv.url("/budget-detail/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn empty_list() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());
        let request = tests::authenticated_request(&user, srv.url("/budget-detail/"));
        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");
        assert_response_body_eq!(
            srv,
            response,
            r#"{"total":0,"results":[],"next":false,"previous":false}"#
        );
    }
}
