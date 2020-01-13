mod db;

pub use self::db::DbSession;
use actix_web::test::TestRequest;
use octo_budget_lib::auth_token::AuthToken;

// ClientRequestExt
pub trait RequestJwtAuthExt {
    fn jwt_auth(self, user_id: i32) -> Self;
}

impl RequestJwtAuthExt for TestRequest {
    fn jwt_auth(self, user_id: i32) -> Self {
        let token = AuthToken::new(user_id)
            .expire_in_hours(10)
            .encrypt(crate::config::AUTH_TOKEN_SECRET.as_bytes());

        self.header(
            actix_web::http::header::AUTHORIZATION,
            format!("JWT {}", token),
        )
    }
}

#[macro_export]
macro_rules! await_test_server {
    ($service:ident) => {{
        actix_web::test::init_service(
            actix_web::App::new()
                .data(crate::db::ConnectionPool::new())
                .data(crate::redis::Redis::new().await)
                .app_data(octo_budget_lib::auth_token::ApiJwtTokenAuthConfig::new(
                    crate::config::AUTH_TOKEN_SECRET.as_bytes(),
                ))
                .service($service),
        )
        .await
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

pub fn setup_env() {
    use dotenv::dotenv;

    dotenv().ok().expect("Failed to parse .env file");
}
