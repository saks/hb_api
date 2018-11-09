use actix_web::error::{ErrorUnauthorized, ParseError};
use actix_web::middleware::{Middleware, Started};
use actix_web::{http::header, HttpRequest, Result as WebResult};

use crate::auth_token::AuthToken;

pub struct VerifyAuthToken;

impl VerifyAuthToken {
    pub fn new() -> Self {
        Self {}
    }
}

impl<AppState> Middleware<AppState> for VerifyAuthToken {
    fn start(&self, req: &HttpRequest<AppState>) -> WebResult<Started> {
        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
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
            Some(token) => AuthToken::verify(token)
                .map(|auth_token| {
                    req.extensions_mut().insert(auth_token);

                    Started::Done
                })
                .map_err(|_| ErrorUnauthorized("TODO: bad token error")),
            None => {
                return Err(ErrorUnauthorized("Wrong token format!"));
            }
        }
    }
}
