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
    // println!("params: {:?}", params);
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

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::client::ClientRequest;
    use actix_web::http::StatusCode;
    use actix_web::test::TestServer;
    use dotenv::dotenv;

    fn setup() {
        dotenv().ok().expect("Failed to parse .env file");
    }

    fn make_token(hours_from_now: i64, secret_str: &str) -> String {
        use frank_jwt::{encode, Algorithm};
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::hours(hours_from_now))
            .to_timespec()
            .sec;
        let header = json!({ "exp": exp });
        let payload = json!({ "foo": 123 });
        let secret = secret_str.to_string();

        encode(header, &secret, &payload, Algorithm::HS256).expect("failed to encode token")
    }

    #[test]
    fn test_auth_required_for_records_app() {
        setup();

        let mut srv = TestServer::with_factory(build);

        let request = ClientRequest::build()
            .uri(&srv.url("/api/records/record-detail/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    }

    #[test]
    fn test_auth_success_for_records_app() {
        setup();

        let mut srv = TestServer::with_factory(build);
        let token = make_token(12, "foo");

        let request = ClientRequest::build()
            .header("Authorization", token)
            .uri(&srv.url("/api/records/record-detail/"))
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(StatusCode::OK, response.status());
        // TODO: check body
    }
}
