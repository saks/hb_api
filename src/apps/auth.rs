use actix_web::{middleware, App, AsyncResponder, FutureResponse, HttpResponse, Json, State};

use futures::future::Future;

use djangohashers;
use time::{now_utc, Duration};

use apps::AppState;
use db::auth::AuthenticateUser;

#[derive(Debug)]
pub enum AuthError {
    Token,
}

pub fn create_token(user_id: i32) -> Result<String, AuthError> {
    use frank_jwt::{encode, Algorithm};

    let exp = (now_utc() + Duration::days(1)).to_timespec().sec;
    let payload = json!({ "user_id": user_id });
    let header = json!({ "exp": exp });
    let secret = "secret123".to_string(); // TODO: read from env var

    encode(header, &secret, &payload, Algorithm::HS256).map_err(|_| AuthError::Token)
}

pub fn check_password(password: &str, hash: &str) -> Result<(), AuthError> {
    match djangohashers::check_password(password, hash) {
        Ok(true) => Ok(()),
        _ => Err(AuthError::Token),
    }
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
                .or_else(|_| Ok(HttpResponse::Unauthorized().into()))
        }).responder()
}

pub fn app() -> App<AppState> {
    App::with_state(AppState::new())
        .prefix("/auth")
        .middleware(middleware::Logger::default())
        .resource("/", |r| {
            r.post().with_config(index, |((cfg, _),)| {
                cfg.limit(1024); // <- limit size of the payload
            })
        })
}
