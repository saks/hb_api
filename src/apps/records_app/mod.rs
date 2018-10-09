// use std::convert::Into;

use actix_web::middleware::Logger;
use actix_web::{App, FutureResponse, HttpResponse, Query, State};
use futures::future;
// use actix_web::{App, AsyncResponder, FutureResponse, HttpResponse, Query, State};
// use futures::{future, future::Future};

use apps::middlewares::auth_by_token::VerifyAuthToken;
use apps::AppState;

#[derive(Deserialize, Debug, Default, Clone)]
struct Params {
    #[serde(default)]
    page: u32,
}

fn index((params_path, _state): (Query<Params>, State<AppState>)) -> FutureResponse<HttpResponse> {
    let params = params_path.into_inner();
    println!("params: {:?}", params);
    Box::new(future::ok(HttpResponse::Ok().json("TODO")))
}

pub fn build() -> App<AppState> {
    App::with_state(AppState::new())
        .prefix("/api/records/record-detail")
        .middleware(Logger::default())
        .middleware(VerifyAuthToken::new())
        .resource("/", |r| {
            r.get().with_config(index, |((_cfg, _),)| {
                // cfg.limit(1024); // <- limit size of the payload
            })
        })
}
