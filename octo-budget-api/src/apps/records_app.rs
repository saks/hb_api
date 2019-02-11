use actix_web::{HttpResponse, Json, Path, Query, Responder, Result as WebResult};
use actix_web_async_await::{await, compat};

use crate::apps::{forms::record::Form, middlewares::VerifyAuthToken, Request, Scope, State};
use crate::redis::helpers::{decrement_tags, increment_tags};

use super::index_params::Params;
use crate::db::messages::{CreateRecord, FindRecord, GetRecords, UpdateRecord};

async fn index((params, state, req): (Query<Params>, State, Request)) -> WebResult<impl Responder> {
    let token = crate::auth_token_from_async_request!(req);
    let params = params.into_inner().validate()?;

    let message = GetRecords {
        page: params.page,
        per_page: params.per_page,
        user_id: token.user_id,
    };

    let result = await!(state.db.send(message))?;

    Ok(HttpResponse::Ok().json(result?))
}

async fn create((form, state, req): (Json<Form>, State, Request)) -> WebResult<impl Responder> {
    let token = crate::auth_token_from_async_request!(req);
    let data = form.into_inner().validate()?;

    await!(state.db.send(CreateRecord::new(&data, &token)))??;
    await!(increment_tags(token.user_id, data.tags, state.redis()))?;

    Ok(HttpResponse::Ok().json(""))
}

async fn update(
    (params, form, state, req): (Path<i32>, Json<Form>, State, Request),
) -> WebResult<impl Responder> {
    let token = crate::auth_token_from_async_request!(req);
    let data = form.into_inner().validate()?;
    let id = params.into_inner();

    let record = await!(state.db.send(FindRecord::new(id, token.user_id)))??;
    await!(state.db.send(UpdateRecord::new(record.id, &data, &token)))??;

    await!(decrement_tags(token.user_id, record.tags, state.redis()))?;
    await!(increment_tags(token.user_id, data.tags, state.redis()))?;

    Ok(HttpResponse::Ok().json(""))
}

pub fn scope(scope: Scope) -> Scope {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("/record-detail/", |r| {
            r.get().with(compat(index));
            r.post().with(compat(create));
        })
        .resource("record-detail/{id}/", |r| {
            r.put().with(compat(update));
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_response_body_eq, db::builders::UserBuilder, tests};
    use actix_web::{
        client::ClientRequest,
        http::{Method, StatusCode},
        test::TestServer,
    };

    fn setup() -> TestServer {
        tests::setup_env();
        setup_test_server()
    }

    fn setup_test_server() -> TestServer {
        use crate::apps::{middlewares::VerifyAuthToken, AppState};

        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.middleware(VerifyAuthToken::default())
                .resource("/record-detail/", |r| {
                    r.get().with(compat(index));
                    r.post().with(compat(create));
                })
                .resource("/record-detail/{id}/", |r| {
                    r.put().with(compat(update));
                });
        })
    }

    #[test]
    fn auth_required_for_records_index() {
        let mut srv = setup();

        let request = ClientRequest::build()
            .uri(&srv.url("/record-detail/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn auth_required_for_records_create() {
        let mut srv = setup();

        let request = ClientRequest::build()
            .uri(&srv.url("/record-detail/"))
            .method(Method::POST)
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn auth_required_for_records_update() {
        let mut srv = setup();

        let request = ClientRequest::build()
            .uri(&srv.url("/record-detail/123"))
            .method(Method::PUT)
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn wrong_json_schema_error_for_create() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());
        let mut request = tests::authenticated_request(&user, srv.url("/record-detail/"));
        request.set_body(r#"{}"#);
        request.set_method(Method::POST);

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::BAD_REQUEST, response.status());
        assert_response_body_eq!(srv, response, r#""#);
    }

    #[test]
    fn create_happy_path() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());
        let mut request = tests::authenticated_request(&user, srv.url("/record-detail/"));
        request.set_body(
            r###"{
            "user":"",
            "amount":{"amount":123,"currency":{"code":"CAD","name":"Canadian Dollar"}},
            "transaction_type":"EXP",
            "tags":["foo"],
            "created_at":0
        }"###,
        );
        request.set_method(Method::POST);

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status());
        assert_response_body_eq!(srv, response, "\"\"");
    }

    #[test]
    fn update_happy_path() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());
        let record = session.create_record2(user.id);

        let url = srv.url(format!("/record-detail/{}/", record.id).as_str());
        let mut request = tests::authenticated_request(&user, url);
        request.set_body(
            r###"{
            "user":"",
            "amount":{"amount":123,"currency":{"code":"CAD","name":"Canadian Dollar"}},
            "transaction_type":"EXP",
            "tags":["foo"],
            "created_at":0
        }"###,
        );
        request.set_method(Method::PUT);

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status());
        assert_response_body_eq!(srv, response, "\"\"");
    }

    #[test]
    fn validation_error_for_update() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());
        let record = session.create_record2(user.id);

        let url = srv.url(format!("/record-detail/{}/", record.id).as_str());
        let mut request = tests::authenticated_request(&user, url);
        request.set_body(
            r###"{
            "user":"",
            "amount":{"amount":123,"currency":{"code":"USD","name":"Canadian Dollar"}},
            "transaction_type":"INC",
            "tags":["foo"],
            "created_at":0
        }"###,
        );
        request.set_method(Method::PUT);

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::BAD_REQUEST, response.status());
        assert_response_body_eq!(
            srv,
            response,
            "{\"currency_code\":[\"\\\"USD\\\" is not a valid choice.\"]}"
        );
    }

    #[test]
    fn valiadtion_errors_for_create() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default());
        let mut request = tests::authenticated_request(&user, srv.url("/record-detail/"));
        request.set_body(
            r###"{
            "user":"",
            "amount":{"amount":123,"currency":{"code":"USD","name":"Canadian Dollar"}},
            "transaction_type":"INC",
            "tags":["foo"],
            "created_at":0
        }"###,
        );
        request.set_method(Method::POST);

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::BAD_REQUEST, response.status());
        assert_response_body_eq!(
            srv,
            response,
            "{\"currency_code\":[\"\\\"USD\\\" is not a valid choice.\"]}"
        );
    }

    #[test]
    fn index_return_empty_list() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default().tags(vec!["foo"]));
        let request = tests::authenticated_request(&user, srv.url("/record-detail/"));
        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");
        assert_response_body_eq!(
            srv,
            response,
            r#"{"total":0,"results":[],"next":false,"previous":false}"#
        );
    }
}
