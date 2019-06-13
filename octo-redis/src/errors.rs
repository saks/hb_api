use failure::Fail;
use redis::Value;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Actix mailbox error {}", _0)]
    ActixMailbox(#[cause] actix::MailboxError),
    #[fail(display = "Redis error {}", _0)]
    Redis(#[cause] redis::RedisError),
    #[fail(display = "Redis error {:?}", _0)]
    UnexpecetdRedisResponse(Value),
    #[fail(display = "Redis error {:?}", _0)]
    UnexpectedValue(redis::RedisError),
}

impl actix_http::ResponseError for Error {}

impl std::convert::From<redis::RedisError> for Error {
    fn from(err: redis::RedisError) -> Self {
        Error::UnexpectedValue(err)
    }
}
