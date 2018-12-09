use actix_web::{
    AsyncResponder, Error as WebError, FutureResponse, HttpRequest, HttpResponse, Scope, State,
};
use futures::{future, future::Future};
use serde_derive::Serialize;

use crate::apps::{middlewares::auth_by_token::VerifyAuthToken, AppState};
use octo_budget_lib::auth_token::AuthToken;

mod tags;

#[derive(Serialize, Default, Debug)]
pub struct ResponseData {
    tags: Vec<String>,
}

fn index((state, req): (State<AppState>, HttpRequest<AppState>)) -> FutureResponse<HttpResponse> {
    let token = match req.extensions_mut().remove::<AuthToken>() {
        Some(token) => token,
        _ => return Box::new(future::ok(HttpResponse::Unauthorized().finish())),
    };

    let get_redis_tags = state
        .redis
        .clone()
        .send(tags::get_ordered_tags_from_redis_msg(token.user_id));

    let get_user_tags = state
        .db
        .send(tags::get_user_tags_from_db_msg(token.user_id));

    get_redis_tags
        .join(get_user_tags)
        .map_err(WebError::from)
        .and_then(|res| Ok(tags::get_ordered_tags(res)?))
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

pub fn scope(scope: Scope<AppState>) -> Scope<AppState> {
    scope
        .middleware(VerifyAuthToken::default())
        .resource("", |r| r.get().with(index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::builders::UserBuilder;
    use actix_web::{client::ClientRequest, http::StatusCode, test::TestServer, HttpMessage};
    use std::str;

    fn setup_env() {
        use dotenv::dotenv;

        dotenv().ok().expect("Failed to parse .env file");
    }

    fn setup_test_server() -> TestServer {
        use crate::apps::middlewares::auth_by_token::VerifyAuthToken;

        TestServer::build_with_state(|| AppState::new()).start(|app| {
            app.middleware(VerifyAuthToken::default())
                .resource("/api/tags/", |r| r.get().with(index));
        })
    }

    fn setup() -> TestServer {
        setup_env();
        setup_test_server()
    }

    #[test]
    fn test_auth_required_for_records_app() {
        let mut srv = setup();

        let request = ClientRequest::build()
            .uri(&srv.url("/api/tags/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn test_auth_success_with_tags() {
        let mut session = crate::tests::DbSession::new();
        let mut srv = setup();

        let user = session.create_user(
            UserBuilder::default()
                .tags(vec!["foo"])
                .password("dummy password"),
        );
        let token = AuthToken::new(user.id, crate::config::AUTH_TOKEN_SECRET.as_bytes())
            .expire_in_hours(10)
            .to_string();

        let request = ClientRequest::build()
            .header("Authorization", format!("JWT {}", token))
            .uri(&srv.url("/api/tags/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());

        let bytes = srv.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();

        assert_eq!(r#"{"tags":["foo"]}"#, body);
    }

    #[test]
    fn test_auth_success_with_ordered() {
        use crate::tests::redis;

        setup_env();

        let mut session = crate::tests::DbSession::new();
        let user = session.create_user(
            UserBuilder::default()
                .tags(vec!["foo", "xxx", "zzz"])
                .password("dummy password"),
        );

        redis::flushall();

        // prepare sort order for tags:
        let redis_key = crate::config::user_tags_redis_key(user.id);
        redis::exec_cmd(vec!["ZADD", &redis_key, "1", "foo"]);
        redis::exec_cmd(vec!["ZADD", &redis_key, "3", "zzz"]);
        redis::exec_cmd(vec!["ZADD", &redis_key, "2", "xxx"]);

        let token = AuthToken::new(user.id, crate::config::AUTH_TOKEN_SECRET.as_bytes())
            .expire_in_hours(10)
            .to_string();

        let mut srv = setup_test_server();

        let request = ClientRequest::build()
            .header("Authorization", format!("JWT {}", token))
            .uri(&srv.url("/api/tags/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());

        let bytes = srv.execute(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();

        assert_eq!(r#"{"tags":["zzz","xxx","foo"]}"#, body);
    }
}
