use actix_http::Payload;
use actix_web::{
    error::{ErrorUnauthorized, ParseError},
    FromRequest, HttpRequest,
};
use failure::Error;
use futures::future::{err, ok, Future};
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

const DEFAULT_EXPIRE_IN_HOURS: i64 = 24;

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct UserId(i32);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for UserId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<UserId> for i32 {
    fn from(id: UserId) -> i32 {
        id.0
    }
}

impl std::ops::Deref for UserId {
    type Target = i32;

    fn deref(&self) -> &i32 {
        &self.0
    }
}

impl PartialEq<UserId> for i32 {
    fn eq(&self, id: &UserId) -> bool {
        &id.0 == self
    }
}

impl UserId {
    fn auth(req: &HttpRequest) -> actix_web::Result<Self> {
        let config = req.app_data::<ApiJwtTokenAuthConfig>().ok_or_else(|| {
            error!("Application is not configured with JWT secret!");
            ErrorUnauthorized(ParseError::Header)
        })?; // TODO: add beter error

        let auth_header = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .ok_or_else(|| ErrorUnauthorized(ParseError::Header))?
            .to_str()
            .map_err(ErrorUnauthorized)?;

        let mut parts = auth_header.split_whitespace();

        if let Some(token_type) = parts.next() {
            if token_type != "JWT" {
                return Err(ErrorUnauthorized("Wrong token type!"));
            }
        }

        match parts.next() {
            Some(token) => AuthToken::from(token, config.secret)
                .map(|auth_token| auth_token.user_id())
                .map_err(|_| ErrorUnauthorized("Bad token!")),
            None => Err(ErrorUnauthorized("Wrong token format!")),
        }
    }
}

#[derive(Default, Debug)] // TODO: do I need Default
pub struct ApiJwtTokenAuthConfig {
    secret: &'static [u8],
}

impl ApiJwtTokenAuthConfig {
    pub fn new(secret: &'static [u8]) -> Self {
        Self { secret }
    }
}

impl FromRequest for UserId {
    type Error = actix_web::Error;
    type Future = Box<dyn Future<Item = Self, Error = Self::Error>>;
    type Config = ApiJwtTokenAuthConfig;

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Box::new(match UserId::auth(req) {
            Ok(user_id) => ok(user_id),
            Err(e) => err(e),
        })
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct AuthToken {
    user_id: UserId,
    expire_in_hours: i64,
}

impl AuthToken {
    pub fn new(user_id: i32) -> Self {
        let user_id = user_id.into();
        let expire_in_hours = DEFAULT_EXPIRE_IN_HOURS;

        Self {
            user_id,
            expire_in_hours,
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn encrypt(&self, secret: &[u8]) -> String {
        use jsonwebtoken::{encode, Header};

        let headers = &Header::default();
        let data = self.data();

        encode(headers, &data, secret).expect("Failed to generate token")
    }

    pub fn expire_in_hours(mut self, n: i64) -> Self {
        self.expire_in_hours = n;
        self
    }

    pub fn from(token: &str, secret: &[u8]) -> Result<Self, Error> {
        use jsonwebtoken::{decode, Validation};

        let token_data = decode::<Data>(token, secret, &Validation::default())?;
        let user_id = token_data.claims.user_id;

        Ok(Self::new(user_id))
    }

    pub fn data(&self) -> Data {
        use time::{now_utc, Duration};

        let exp = (now_utc() + Duration::hours(self.expire_in_hours))
            .to_timespec()
            .sec;

        Data {
            exp,
            user_id: self.user_id.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Data {
    pub user_id: i32,
    pub exp: i64,
    // pub username: &'a str,
    // pub email: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, Validation};

    const TEST_SECRET: &[u8] = b"foo-bar-secret";
    const TEST_USER_ID: i32 = 112233;

    #[test]
    fn test_create_token() {
        let token = AuthToken::new(TEST_USER_ID).encrypt(TEST_SECRET);
        assert_eq!(128, token.len());

        let decoded = decode::<Data>(&token, TEST_SECRET, &Validation::default()).unwrap();
        assert_eq!(TEST_USER_ID, decoded.claims.user_id);
    }

    #[test]
    #[should_panic(expected = "InvalidSignature")]
    fn test_create_token_with_invalid_secret() {
        let token = AuthToken::new(TEST_USER_ID).encrypt(TEST_SECRET);
        decode::<Data>(&token, b"secret", &Validation::default()).unwrap();
    }

    #[test]
    fn test_verify_token() {
        let valid_token = make_token(24, TEST_SECRET);
        let result = AuthToken::from(&valid_token, TEST_SECRET).unwrap().user_id;

        assert_eq!(TEST_USER_ID, result);
    }

    #[test]
    fn test_verify_expired_token() {
        let token = make_token(-33, TEST_SECRET);

        assert!(AuthToken::from(&token, TEST_SECRET).is_err());
    }

    #[test]
    fn test_verify_token_with_wrong_signature() {
        let valid_token = make_token(33, b"bar");

        assert!(AuthToken::from(&valid_token, TEST_SECRET).is_err());
    }

    fn make_token(hours_from_now: i64, secret: &[u8]) -> String {
        AuthToken::new(TEST_USER_ID)
            .expire_in_hours(hours_from_now)
            .encrypt(secret)
    }
}
