use actix::prelude::*;
use actix_web::{
    error, middleware, App, AsyncResponder, Error, FutureResponse, HttpResponse, Json, State,
};

use futures::future::Future;

use frank_jwt;
use time::{now_utc, Duration};

use db::{db_executor, DbExecutor, FindUser};
use djangohashers::check_password;

fn create_token(user_id: i32) -> Result<String, frank_jwt::Error> {
    let exp = (now_utc() + Duration::days(1)).to_timespec().sec;
    let payload = json!({ "user_id": user_id });
    let header = json!({ "exp": exp });
    let secret = "secret123".to_string(); // TODO: read from env var
    frank_jwt::encode(header, &secret, &payload, frank_jwt::Algorithm::HS256)
}

#[derive(Deserialize, Debug)]
pub struct AuthForm {
    username: String,
    password: String,
}

/// State with DbExecutor address
pub struct AppState {
    db: Addr<DbExecutor>,
}

fn index((auth_form, state): (Json<AuthForm>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let find_user = FindUser {
        username: auth_form.username.clone(),
    };
    state
        .db
        .send(find_user)
        .from_err()
        .and_then(move |res| match res {
            Ok(user) => {
                let hash = &user.password.clone();
                let password = &auth_form.password;

                match check_password(password, hash) {
                    Ok(is_valid) => {
                        if is_valid {
                            match create_token(user.id) {
                                Ok(token) => Ok(HttpResponse::Ok().json(token)),
                                Err(_) => Ok(HttpResponse::Unauthorized().into()),
                            }
                        } else {
                            Ok(HttpResponse::Unauthorized().into())
                        }
                    }
                    Err(_) => Ok(HttpResponse::Unauthorized().into()),
                }
            }
            Err(_) => Ok(HttpResponse::Unauthorized().into()),
        }).responder()
}

pub fn app() -> App<AppState> {
    App::with_state(AppState { db: db_executor() })
        .prefix("/auth")
        .middleware(middleware::Logger::default())
        .resource("/{username}", |r| {
            r.post().with_config(index, |((cfg, _),)| {
                cfg.limit(1024); // <- limit size of the payload
            })
        })
}
