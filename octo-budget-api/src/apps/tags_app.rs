use actix_web::{AsyncResponder, Error as WebError, HttpResponse, Json, Scope};
use futures::future::Future;
use serde_derive::{Deserialize, Serialize};

use crate::apps::{
    middlewares::auth_by_token::VerifyAuthToken, AppState, Request, Response, State,
};

pub mod db;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TagsData {
    tags: Vec<String>,
}

fn index((state, req): (State, Request)) -> Response {
    let token = crate::auth_token_from_request!(req);

    let get_redis_tags = state
        .redis
        .clone()
        .send(db::get_ordered_tags_from_redis_msg(token.user_id));

    let get_user_tags = state.db.send(db::get_user_tags_from_db_msg(token.user_id));

    get_user_tags
        .join(get_redis_tags)
        .map_err(WebError::from)
        .and_then(|res| Ok(db::get_ordered_tags(res)?))
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

fn update((tags_data, state, req): (Json<TagsData>, State, Request)) -> Response {
    let user_id = crate::auth_token_from_request!(req).user_id;
    let tags = tags_data.into_inner().tags;

    let message = db::SetUserTags { tags, user_id };
    let set_user_tags = state.db.send(message);
    let get_redis_tags = state
        .redis
        .clone()
        .send(db::get_ordered_tags_from_redis_msg(user_id));

    set_user_tags
        .join(get_redis_tags)
        .map_err(WebError::from)
        .and_then(|res| Ok(db::get_ordered_tags(res)?))
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("", |r| {
            r.get().with(index);
            r.put().with(update);
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

    fn setup_test_server() -> TestServer {
        use crate::apps::middlewares::auth_by_token::VerifyAuthToken;

        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.middleware(VerifyAuthToken::default())
                .resource("/api/tags/", |r| {
                    r.get().with(index);
                    r.put().with(update);
                });
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
            .uri(&srv.url("/api/tags/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn test_auth_required_for_update() {
        let mut srv = setup();

        let request = ClientRequest::build()
            .uri(&srv.url("/api/tags/"))
            .method(Method::PUT)
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn test_auth_success_with_tags() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default().tags(vec!["foo"]));
        let request = tests::authenticated_request(&user, srv.url("/api/tags/"));
        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status(), "wrong status code");
        assert_response_body_eq!(srv, response, r#"{"tags":["foo"]}"#);
    }

    #[test]
    fn test_auth_success_with_ordered() {
        use crate::tests::redis;

        tests::setup_env();

        let mut session = tests::DbSession::new();
        let user = session.create_user(UserBuilder::default().tags(vec!["foo", "xxx", "zzz"]));

        redis::flushall();

        // prepare sort order for tags:
        let redis_key = crate::config::user_tags_redis_key(user.id);
        redis::exec_cmd(vec!["ZADD", &redis_key, "1", "foo"]);
        redis::exec_cmd(vec!["ZADD", &redis_key, "3", "zzz"]);
        redis::exec_cmd(vec!["ZADD", &redis_key, "2", "xxx"]);

        let mut srv = setup_test_server();
        let request = tests::authenticated_request(&user, srv.url("/api/tags/"));
        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status());
        assert_response_body_eq!(srv, response, r#"{"tags":["zzz","xxx","foo"]}"#);
    }

    #[test]
    fn test_add_and_remove_user_tags() {
        let mut session = tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(UserBuilder::default().tags(vec!["foo"]));

        let mut request = tests::authenticated_request(&user, srv.url("/api/tags/"));
        request.set_body(r#"{"tags":["bar"]}"#);
        request.set_method(Method::PUT);

        let response = srv.execute(request.send()).unwrap();

        // check response
        assert_eq!(StatusCode::OK, response.status());
        assert_response_body_eq!(srv, response, r#"{"tags":["bar"]}"#);

        // make sure that user was updated
        assert_eq!(vec!["bar"], user.reload(session).tags);
    }
}
