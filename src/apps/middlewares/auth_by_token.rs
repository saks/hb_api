use actix_web::error::{ErrorUnauthorized, ParseError};
use actix_web::middleware::{Middleware, Started};
use actix_web::{HttpRequest, Result as WebResult};

use config;

pub struct VerifyAuthToken {
    secret: String,
}

pub type AuthUserId = Box<i64>;

impl VerifyAuthToken {
    pub fn new() -> Self {
        let secret = config::AUTH_TOKEN_SECRET.to_string();
        Self { secret }
    }

    fn verify(&self, http_header: &str) -> Result<i64, ()> {
        use frank_jwt::{decode, Algorithm};
        use time::now_utc;

        let payload = http_header.to_string();
        let (header, data) = decode(&payload, &self.secret, Algorithm::HS256).map_err(|_| ())?;

        let exp_time: i64 = header.get("exp").and_then(|exp| exp.as_i64()).ok_or(())?;

        let now = now_utc().to_timespec().sec;
        if exp_time < now {
            return Err(());
        }

        let user_id: i64 = data.get("user_id").and_then(|id| id.as_i64()).ok_or(())?;

        Ok(user_id)
    }
}

impl<AppState> Middleware<AppState> for VerifyAuthToken {
    fn start(&self, req: &HttpRequest<AppState>) -> WebResult<Started> {
        let r = req.clone();

        let auth_header = r
            .headers()
            .get("Authorization")
            .ok_or(ErrorUnauthorized(ParseError::Header))?
            .to_str()
            .map_err(ErrorUnauthorized)?;

        self.verify(auth_header)
            .map(|user_id| {
                let auth_user_id: AuthUserId = Box::new(user_id);
                req.extensions_mut().insert(auth_user_id);

                Started::Done
            })
            .map_err(|_| ErrorUnauthorized("TODO: bad token error"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    fn make_token(hours_from_now: i64, secret_str: &str) -> String {
        use frank_jwt::{encode, Algorithm};
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::hours(hours_from_now))
            .to_timespec()
            .sec;
        let header = json!({ "exp": exp });
        let payload = json!({ "user_id": 123 });
        let secret = secret_str.to_string();

        encode(header, &secret, &payload, Algorithm::HS256).expect("failed to encode token")
    }

    fn setup() {
        env::set_var("AUTH_TOKEN_SECRET", "foo");
    }

    fn teardown() {
        env::remove_var("AUTH_TOKEN_SECRET");
    }

    #[test]
    fn test_verify_token() {
        setup();

        let middleware = VerifyAuthToken::new();
        let valid_token = make_token(33, "foo");

        assert!(middleware.verify(&valid_token).is_ok());

        teardown()
    }

    #[test]
    fn test_verify_expired_token() {
        setup();

        let middleware = VerifyAuthToken::new();
        let valid_token = make_token(-33, "foo");

        assert!(middleware.verify(&valid_token).is_err());

        teardown();
    }

    #[test]
    fn test_verify_token_with_wrong_signature() {
        setup();

        let middleware = VerifyAuthToken::new();
        let valid_token = make_token(33, "bar");

        assert!(middleware.verify(&valid_token).is_err());

        teardown();
    }
}
