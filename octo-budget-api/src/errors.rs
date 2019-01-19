use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use failure_derive::Fail;
use serde::{Serialize, Serializer};

#[derive(Fail, Debug, Clone, Copy, PartialEq)]
pub enum ValidationError {
    #[fail(display = "Unable to log in with provided credentials.")]
    AuthFailed,
    #[fail(display = "This field may not be blank.")]
    CannotBeBlank,
    #[fail(display = "This field is required.")]
    MustPresent,
}

impl actix_web::error::ResponseError for ValidationError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(StatusCode::BAD_REQUEST)
    }
}

impl Serialize for ValidationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.to_string())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Cannot read sorted tags from redis {}", _0)]
    Redis(#[cause] actix_redis::Error),

    #[fail(display = "Bad response from redis `{}'", _0)]
    BadRedisResponse(String),

    #[fail(display = "Cannot find user by id: `{}'", _0)]
    UserNotFound(i32),

    #[fail(display = "Cannot update {} with id: `{}'", _0, _1)]
    RecordNotUpdated(&'static str, i32),

    #[fail(display = "Unknown database error {}", _0)]
    UnknownDb(#[cause] diesel::result::Error),

    #[fail(display = "Unexpected error {}", _0)]
    Unknown(#[cause] failure::Error),

    #[fail(display = "Unexpected error: {}", _0)]
    UnknownMsg(&'static str),

    #[fail(display = "Cannot get database connection")]
    Connection,
}

impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::UserNotFound(_) | Error::RecordNotUpdated(..) => {
                HttpResponse::new(StatusCode::NOT_FOUND)
            }
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
