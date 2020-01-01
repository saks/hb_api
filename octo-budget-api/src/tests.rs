mod db;
pub mod redis;

pub use self::db::DbSession;
use actix_web::client::ClientRequest;
use octo_budget_lib::auth_token::AuthToken;

// ClientRequestExt
pub trait RequestJwtAuthExt {
    fn jwt_auth(self, user_id: i32) -> Self;
}

impl RequestJwtAuthExt for ClientRequest {
    fn jwt_auth(self, user_id: i32) -> Self {
        let token = AuthToken::new(user_id)
            .expire_in_hours(10)
            .encrypt(crate::config::AUTH_TOKEN_SECRET.as_bytes());

        self.header("Authorization", format!("JWT {}", token))
    }
}

// TODO: add redis
// .data(crate::redis::start())
#[macro_export]
macro_rules! await_test_server {
    ($service:ident) => {{
        actix_web::test::init_service(
            actix_web::App::new()
                .data(crate::db::ConnectionPool::new())
                .data(octo_budget_lib::auth_token::ApiJwtTokenAuthConfig::new(
                    crate::config::AUTH_TOKEN_SECRET.as_bytes(),
                ))
                .service($service),
        )
        .await
    }};
}

#[macro_export]
macro_rules! test_server {
    ($service:ident) => {{
        actix_http_test::TestServer::new(|| {
            actix_http::HttpService::new(
                actix_web::App::new()
                    .data(crate::db::start())
                    .data(crate::redis::start())
                    .data(octo_budget_lib::auth_token::ApiJwtTokenAuthConfig::new(
                        crate::config::AUTH_TOKEN_SECRET.as_bytes(),
                    ))
                    .service($service),
            )
        })
    }};
}

#[macro_export]
macro_rules! tags_vec {
    ( $( $x:expr ),* ) => {
        {
            #[allow(unused_mut)]
            let mut temp_vec: Vec<String> = Vec::new();
            $(
                temp_vec.push($x.to_string());
            )*
            temp_vec
        }
    };
}

#[macro_export]
macro_rules! assert_response_body_eq {
    ($srv:ident, $response:ident, $body:tt) => {
        use actix_web::HttpMessage;

        let bytes = $srv.execute($response.body()).unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();

        assert_eq!($body, body, "wrong response body");
    };
}

//pub fn run_future<F: 'static, Fut: 'static>(fut: Fut, callback: F)
//where
//    Fut: futures::Future,
//    F: Fn(Result<Fut::Item, Fut::Error>),
//{
//    let system = actix::System::new("test");
//
//    actix::Arbiter::spawn({
//        fut.then(move |res| {
//            callback(res);
//            actix::System::current().stop();
//            futures::future::ok(())
//        })
//    });
//
//    system.run().unwrap();
//}

// use crate::db::models::AuthUser;
// use actix_web::{client::ClientRequest, http::Method};
// use octo_budget_lib::auth_token::AuthToken;

// pub fn authenticated_request(user: &AuthUser, uri: String) -> ClientRequest {
//     let token = AuthToken::new(user.id, crate::config::AUTH_TOKEN_SECRET.as_bytes())
//         .expire_in_hours(10)
//         .to_string();
//
//     ClientRequest::build()
//         .header("Authorization", format!("JWT {}", token))
//         .uri(uri)
//         .method(Method::GET)
//         .content_type("applicaton/json")
//         .finish()
//         .unwrap()
// }

pub fn setup_env() {
    use dotenv::dotenv;

    dotenv().ok().expect("Failed to parse .env file");
}
