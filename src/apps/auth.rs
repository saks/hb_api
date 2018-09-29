use actix::prelude::*;
use actix_web::{middleware, App, AsyncResponder, FutureResponse, HttpResponse, Json, State};

use futures::future::Future;

use frank_jwt;
use time::{now_utc, Duration};

use db::{db_executor, AuthenticateUser, DbExecutor};

pub fn create_token(user_id: i32) -> Result<String, frank_jwt::Error> {
    let exp = (now_utc() + Duration::days(1)).to_timespec().sec;
    let payload = json!({ "user_id": user_id });
    let header = json!({ "exp": exp });
    let secret = "secret123".to_string(); // TODO: read from env var
    frank_jwt::encode(header, &secret, &payload, frank_jwt::Algorithm::HS256)
}

/// State with DbExecutor address
pub struct AppState {
    db: Addr<DbExecutor>,
}

fn index(
    (auth_user, state): (Json<AuthenticateUser>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(auth_user.into_inner())
        .from_err()
        .and_then(|res| {
            res.map(|token| HttpResponse::Ok().json(token))
                .or(Ok(HttpResponse::Unauthorized().into()))
        }).responder()
}

pub fn app() -> App<AppState> {
    App::with_state(AppState { db: db_executor() })
        .prefix("/auth")
        .middleware(middleware::Logger::default())
        .resource("/", |r| {
            r.post().with_config(index, |((cfg, _),)| {
                cfg.limit(1024); // <- limit size of the payload
            })
        })
}
