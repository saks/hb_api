use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use failure::Fail;
// use octo_budget_lib::auth_token::UserId;
use diesel::result::Error as DieselError;
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
    Redis(redis::RedisError),

    // #[fail(display = "Cannot find user by id: `{}'", _0)]
    // UserNotFound(UserId),
    //
    // #[fail(display = "Cannot find record")]
    // NotFound,
    //
    // #[fail(display = "Unknown database error {}", _0)]
    // UnknownDb(#[cause] diesel::result::Error),
    #[fail(display = "Unexpected error {}", _0)]
    Unknown(#[cause] failure::Error),
    // #[fail(display = "Cannot get database connection: {}", _0)]
    // Connection(#[cause] r2d2::Error),
    //
    // #[fail(display = "Cannot get database connection: {}", _0)]
    // Connection2(#[cause] diesel::r2d2::Error),
}

#[derive(Debug, Fail)]
pub enum DbError {
    #[fail(display = "Thread pool is gone")]
    ThreadPoolIsGone,

    // TODO: add search query
    #[fail(display = "Failed to find record from table {}", _0)]
    NotFound(&'static str),

    #[fail(display = "Cannot get database connection: {}", _0)]
    NoConnection(#[cause] r2d2::Error),

    #[fail(display = "Unknown database error {}", _0)]
    Unknown(#[cause] diesel::result::Error),

    #[fail(display = "Cannot update {} with id: `{}'", _0, _1)]
    NotUpdated(&'static str, i32),

    #[fail(display = "Unexpected query result: {}", _0)]
    UnexpectedResult(&'static str),
}

pub fn add_table_name(table_name: &'static str) -> impl Fn(DieselError) -> DbError {
    move |error: DieselError| match error {
        DieselError::NotFound => DbError::NotFound(table_name),
        _ => DbError::Unknown(error),
    }
}

pub type DbResult<T> = Result<T, DbError>;

impl From<r2d2::Error> for DbError {
    fn from(error: r2d2::Error) -> Self {
        DbError::NoConnection(error)
    }
}

impl From<diesel::result::Error> for DbError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            DieselError::NotFound => Self::NotFound("Unspecified table".into()),
            _ => Self::Unknown(error),
        }
    }
}

impl actix_web::error::ResponseError for DbError {
    fn error_response(&self) -> HttpResponse {
        match self {
            DbError::NotFound(_n) => HttpResponse::new(StatusCode::NOT_FOUND),
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

// pub type OctoApiResult<T> = Result<T, Error>;

impl From<redis::RedisError> for Error {
    fn from(error: redis::RedisError) -> Self {
        Error::Redis(error)
    }
}

impl From<failure::Error> for Error {
    fn from(error: failure::Error) -> Self {
        Error::Unknown(error)
    }
}

impl From<actix::MailboxError> for Error {
    fn from(error: actix::MailboxError) -> Self {
        Error::Unknown(error.into())
    }
}

// impl From<diesel::r2d2::Error> for Error {
//     fn from(error: diesel::r2d2::Error) -> Self {
//         Error::Connection2(error)
//     }
// }

// impl From<diesel::result::Error> for Error {
//     fn from(error: diesel::result::Error) -> Self {
//         match error {
//             diesel::result::Error::NotFound => Error::NotFound,
//             err => Error::UnknownDb(err),
//         }
//     }
// }

impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
