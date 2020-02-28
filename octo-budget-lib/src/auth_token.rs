use actix_http::Payload;
use actix_web::{
    error::{ErrorUnauthorized, ParseError},
    FromRequest, HttpRequest,
};
use failure::Error;
use futures::future::{err, ok, Future};
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::{fmt, pin::Pin};

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

#[derive(Default, Debug)]
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
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ApiJwtTokenAuthConfig;

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Box::pin(match UserId::auth(req) {
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
        use jsonwebtoken::{encode, Header, EncodingKey};

        let secret = EncodingKey::from_secret(secret);
        let headers = &Header::default();
        let data = self.data();

        encode(headers, &data, &secret).expect("Failed to generate token")
    }

    pub fn expire_in_hours(mut self, n: i64) -> Self {
        self.expire_in_hours = n;
        self
    }

    pub fn from(token: &str, secret: &[u8]) -> Result<Self, Error> {
        use jsonwebtoken::{decode, Validation, DecodingKey};

        let secret = DecodingKey::from_secret(secret);
        let token_data = decode::<Data>(token, &secret, &Validation::default())?;
        let user_id = token_data.claims.user_id;

        Ok(Self::new(user_id))
    }

    pub fn data(&self) -> Data {
        use time::{OffsetDateTime, Duration};

        let exp = (OffsetDateTime::now() + Duration::hours(self.expire_in_hours)).timestamp();

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
mod tests;
