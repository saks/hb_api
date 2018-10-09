use actix_web::error::{ErrorUnauthorized, ParseError};
use actix_web::middleware::{Middleware, Started};
use actix_web::{HttpRequest, Result};

use config;

pub struct VerifyAuthToken {
    secret: String,
}

impl VerifyAuthToken {
    pub fn new() -> Self {
        let secret = config::AUTH_TOKEN_SECRET.to_string();
        Self { secret }
    }

    fn verify(&self, auth_header: &str) -> Result<Started> {
        // TODO: verify jwt token
        Ok(Started::Done)
    }
}

impl<AppState> Middleware<AppState> for VerifyAuthToken {
    fn start(&self, req: &HttpRequest<AppState>) -> Result<Started> {
        let r = req.clone();
        let auth_header = r
            .headers()
            .get("Authorization")
            .ok_or(ErrorUnauthorized(ParseError::Header))?
            .to_str()
            .map_err(ErrorUnauthorized)?;

        self.verify(auth_header)
    }
}
