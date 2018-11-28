use actix_web::{error, http, HttpResponse};
use failure_derive::Fail;
use serde::{Serialize, Serializer};

use super::Data;

#[derive(Fail, Debug, Clone, Copy, PartialEq)]
pub enum AuthError {
    #[fail(display = "Unable to log in with provided credentials.")]
    AuthFailed,
    #[fail(display = "This field may not be blank.")]
    CannotBeBlank,
    #[fail(display = "This field is required.")]
    MustPresent,
}

impl Serialize for AuthError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

impl From<AuthError> for Data {
    fn from(error: AuthError) -> Data {
        let mut data = Data::default();

        match error {
            AuthError::AuthFailed => {
                data.non_field_errors.push(error);
            }
            _ => {}
        }

        data
    }
}

impl error::ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match self {
            AuthError::AuthFailed => http::StatusCode::UNAUTHORIZED,
            AuthError::CannotBeBlank => http::StatusCode::BAD_REQUEST,
            AuthError::MustPresent => http::StatusCode::BAD_REQUEST,
        };

        let body = Data::from(*self);
        HttpResponse::build(status_code).json(body)
    }
}
