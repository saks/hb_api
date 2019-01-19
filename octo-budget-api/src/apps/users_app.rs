use actix_web::{HttpResponse, Responder, Result as WebResult};
use actix_web_async_await::compat;

use crate::apps::{middlewares::VerifyAuthToken, Request, Scope};

async fn show(req: Request) -> WebResult<impl Responder> {
    let _token = crate::auth_token_from_async_request!(req);

    Ok(HttpResponse::Ok().json(""))
}

pub fn scope(scope: Scope) -> Scope {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("/{user_id}/", |r| {
            r.get().with(compat(show));
        })
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{assert_response_body_eq, db::builders::UserBuilder, tests};
//     use actix_web::{client::ClientRequest, http::StatusCode, test::TestServer};
//
//     fn setup() -> TestServer {
//         tests::setup_env();
//         setup_test_server()
//     }
//
//     fn setup_test_server() -> TestServer {
//         use crate::apps::middlewares::VerifyAuthToken;
//
//         TestServer::build_with_state(|| AppState::new()).start(|app| {
//             app.middleware(VerifyAuthToken::default())
//                 .resource("/api/records/record-detail/", |r| {
//                     r.get().with(compat(index))
//                 });
//         })
//     }
//
//     #[test]
//     fn auth_required_for_records_app() {
//         let mut srv = setup();
//
//         let request = ClientRequest::build()
//             .uri(&srv.url("/api/records/record-detail/"))
//             .finish()
//             .unwrap();
//
//         let response = srv.execute(request.send()).unwrap();
//
//         assert_eq!(StatusCode::UNAUTHORIZED, response.status());
//     }
//
//     #[test]
//     fn empty_list_of_records() {
//         let mut session = tests::DbSession::new();
//         let mut srv = setup();
//
//         let user = session.create_user(UserBuilder::default().tags(vec!["foo"]));
//         let request = tests::authenticated_request(&user, srv.url("/api/records/record-detail/"));
//         let response = srv.execute(request.send()).unwrap();
//
//         assert_eq!(StatusCode::OK, response.status(), "wrong status code");
//         assert_response_body_eq!(
//             srv,
//             response,
//             r#"{"total":0,"results":[],"next":false,"previous":false}"#
//         );
//     }
// }
