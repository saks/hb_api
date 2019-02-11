use actix_web::{HttpResponse, Json, Responder, Result};
use actix_web_async_await::{await, compat};
use serde_derive::{Deserialize, Serialize};

use super::{helpers::sort_tags, middlewares::VerifyAuthToken, Request, Scope, State};

use crate::db::messages::{GetUserTags, SetUserTags};
use crate::redis::helpers::read_redis_tags;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Data {
    tags: Vec<String>,
}

fn ordered_tags(user_tags: Vec<String>, redis_tags: Vec<String>) -> Data {
    let tags = sort_tags(redis_tags, user_tags);
    Data { tags }
}

async fn index((state, req): (State, Request)) -> Result<impl Responder> {
    let user_id = crate::auth_token_from_async_request!(req).user_id;

    let user_tags = await!(state.db.send(GetUserTags::new(user_id)))?;
    let redis_tags = await!(read_redis_tags(user_id, state.redis()));

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags?, redis_tags?)))
}

async fn update((data, state, req): (Json<Data>, State, Request)) -> Result<impl Responder> {
    let user_id = crate::auth_token_from_async_request!(req).user_id;
    let tags = data.into_inner().tags;

    let user_tags = await!(state.db.send(SetUserTags::new(user_id, tags)))?;
    let redis_tags = await!(read_redis_tags(user_id, state.redis()));

    Ok(HttpResponse::Ok().json(ordered_tags(user_tags?, redis_tags?)))
}

pub fn scope(scope: Scope) -> Scope {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("", |r| {
            r.get().with(compat(index));
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

    fn setup_test_server() -> TestServer {
        use crate::apps::{middlewares::VerifyAuthToken, AppState};

        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.middleware(VerifyAuthToken::default())
                .resource("/api/tags/", |r| {
                    r.get().with(compat(index));
                    r.put().with(compat(update));
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
