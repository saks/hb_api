use actix_web::{middleware, App, AsyncResponder, FutureResponse, HttpResponse, Json, State};
use futures::future::Future;

use apps::AppState;
use db::auth::AuthenticateUser;
use failure::Error;

#[derive(Debug, Fail)]
pub enum AuthError {
    #[fail(display = "Failed to generate auth token")]
    EncodeAuthToken,
    #[fail(display = "Token is not valid")]
    InvalidToken,
}

pub fn create_token(user_id: i32) -> Result<String, Error> {
    use config;
    use frank_jwt::{encode, Algorithm};
    use time::{now_utc, Duration};

    let exp = (now_utc() + Duration::days(1)).to_timespec().sec;
    let payload = json!({ "user_id": user_id });
    let header = json!({ "exp": exp });
    let secret = &config::AUTH_TOKEN_SECRET.to_string();

    encode(header, secret, &payload, Algorithm::HS256)
        .map_err(|_| AuthError::EncodeAuthToken.into())
}

pub fn check_password(password: &str, hash: &str) -> Result<(), Error> {
    use djangohashers;

    match djangohashers::check_password(password, hash) {
        Ok(true) => Ok(()),
        _ => Err(AuthError::InvalidToken.into()),
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
