use actix_web::middleware::Logger;
use actix_web::{App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Query, State};
use futures::{future, future::Future};

use crate::apps::middlewares::auth_by_token::VerifyAuthToken;
use crate::apps::AppState;
use crate::auth_token::AuthToken;

mod db;
mod params;
mod response_data;

use self::db::GetRecordsMessage;
use self::params::Params;

fn auth_error_response() -> FutureResponse<HttpResponse> {
    Box::new(future::ok(HttpResponse::Unauthorized().finish()))
}

fn index(
    (query_params, state, request): (Query<Params>, State<AppState>, HttpRequest<AppState>),
) -> FutureResponse<HttpResponse> {
    let params = query_params.into_inner();

    match params.validate() {
        Ok(Params { page, per_page }) => {
            let token: AuthToken = match request.extensions_mut().remove() {
                Some(token) => token,
                None => {
                    return auth_error_response();
                }
            };
            let user_id = token.data.user_id;

            let message = GetRecordsMessage {
                page,
                per_page,
                user_id,
            };

            state
                .db
                .send(message)
                .from_err()
                .and_then(|result| {
                    result
                        .map(|data| HttpResponse::Ok().json(data))
                        .map_err(|e| e.into())
                })
                .responder()
        }
        Err(response_data) => Box::new(future::ok(HttpResponse::BadRequest().json(response_data))),
    }
}

pub fn build() -> App<AppState> {
    App::with_state(AppState::new())
        .prefix("/api/records/record-detail")
        .middleware(Logger::default())
        .middleware(VerifyAuthToken::new())
        .resource("/", |r| r.get().with(index))
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

    fn make_token(hours_from_now: i64, secret: &[u8]) -> String {
        use crate::auth_token::Claims;
        use jsonwebtoken::{encode, Header};
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::hours(hours_from_now))
            .to_timespec()
            .sec;
        let user_id = 123;
        let my_claims = Claims { user_id, exp };

        encode(&Header::default(), &my_claims, secret).expect("Failed to generate token")
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
        use std::env;
        env::set_var("AUTH_TOKEN_SECRET", "foo-bar-secret");

        let mut srv = TestServer::with_factory(build);
        let token = make_token(12, b"foo-bar-secret");

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
