use actix_web::{
    error::{ErrorUnauthorized, ParseError},
    http::header,
    middleware::{Middleware, Started},
    HttpRequest, Result as WebResult,
};
use octo_budget_lib::auth_token::AuthToken;

use crate::config;

#[derive(Default)]
pub struct VerifyAuthToken;

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
            Some(token) => AuthToken::from(token, config::AUTH_TOKEN_SECRET.as_bytes())
                .map(|auth_token| {
                    req.extensions_mut().insert(auth_token); // TODO: insert UserId type (wrapper around i32)
                    Started::Done
                })
                .map_err(|_| ErrorUnauthorized("TODO: bad token error")),
            None => Err(ErrorUnauthorized("Wrong token format!")),
        }
    }
}
