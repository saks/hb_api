use actix::{Handler, Message};
use actix_redis::Command;
use actix_web_async_await::await;
use failure::Fallible;
use failure_derive::Fail;
use redis_async::{
    resp::{FromResp, RespValue::Array},
    resp_array,
};
use std::convert::Into;

use super::Redis;
use crate::db::{schema::auth_user, DbExecutor};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Cannot read sorted tags from redis {}", _0)]
    Redis(#[cause] actix_redis::Error),

    #[fail(display = "Bad response from redis `{}'", _0)]
    BadRedisResponse(String),

    #[fail(display = "Cannot find user by id: `{}'", _0)]
    UserNotFound(i32),

    #[fail(display = "Unknown database error {}", _0)]
    UnknownDb(#[cause] diesel::result::Error),

    #[fail(display = "Unexpected error {}", _0)]
    Unknown(#[cause] failure::Error),

    #[fail(display = "Cannot get database connection")]
    Connection,
}

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::UserNotFound(_) => HttpResponse::new(StatusCode::NOT_FOUND),
            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

pub async fn read_redis_tags(user_id: i32, redis: Redis) -> Fallible<Vec<String>> {
    use self::Error::BadRedisResponse;

    let redis_key = crate::config::user_tags_redis_key(user_id);

    let command = Command(resp_array!["zrevrange", redis_key, "0", "-1"]);
    let response = await!(redis.send(command))?.map_err(Error::Redis)?;

    let tags = match response {
        // Here we assume that if returned value is of Array type, then query has succeeded.
        res @ Array(..) => Vec::from_resp(res).map_err(|e| BadRedisResponse(format!("{:?}", e))),
        res @ _ => Err(BadRedisResponse(format!("{:?}", res))),
    }?;

    Ok(tags)
}

pub type TagsResult = Result<Vec<String>, Error>;

pub struct GetUserTags {
    user_id: i32,
}

impl Message for GetUserTags {
    type Result = TagsResult;
}

impl GetUserTags {
    pub fn new(user_id: i32) -> Self {
        GetUserTags { user_id }
    }
}

impl Handler<GetUserTags> for DbExecutor {
    type Result = TagsResult;

    fn handle(&mut self, msg: GetUserTags, _: &mut Self::Context) -> Self::Result {
        use diesel::prelude::*;

        let connection = &self.pool.get().map_err(|_| Error::Connection)?;

        auth_user::table
            .select(auth_user::tags)
            .filter(auth_user::id.eq(msg.user_id))
            .first(connection)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::UserNotFound(msg.user_id),
                err @ _ => Error::UnknownDb(err),
            })
    }
}

pub struct SetUserTags {
    tags: Vec<String>,
    user_id: i32,
}

impl SetUserTags {
    pub fn new(user_id: i32, tags: Vec<String>) -> Self {
        Self { user_id, tags }
    }
}

impl Message for SetUserTags {
    type Result = TagsResult;
}

impl Handler<SetUserTags> for DbExecutor {
    type Result = TagsResult;

    fn handle(&mut self, msg: SetUserTags, _: &mut Self::Context) -> Self::Result {
        use diesel::prelude::*;

        let connection = &self.pool.get().map_err(|_| Error::Connection)?;

        let SetUserTags { user_id, tags } = msg;

        let target = auth_user::table.filter(auth_user::id.eq(user_id));
        diesel::update(target)
            .set(auth_user::tags.eq(&tags))
            .execute(connection)
            .map_err(Error::UnknownDb)?;

        Ok(tags)
    }
}

pub fn sort_tags(redis_tags: Vec<String>, user_tags: Vec<String>) -> Vec<String> {
    let mut result = Vec::with_capacity(user_tags.len());

    if user_tags.is_empty() {
        return result;
    }

    for item in redis_tags {
        if user_tags.contains(&item) {
            result.push(item);
        }
    }

    for tag in user_tags {
        if !result.contains(&tag) {
            result.push(tag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags_vec;
    use crate::tests::{self as tests, redis};
    use tokio_async_await::compat::backward::Compat;

    #[test]
    fn sorting_tags_with_empty_user_tags() {
        let user_tags = tags_vec![];
        let redis_tags = tags_vec!["foo"];
        let sorted = sort_tags(redis_tags, user_tags);

        assert_eq!(tags_vec![], sorted);
    }

    #[test]
    fn sorting_tags_with_user_tags_not_matching_ones_from_redis() {
        let user_tags = tags_vec!["bar"];
        let redis_tags = tags_vec!["foo"];
        let sorted = sort_tags(redis_tags, user_tags);

        assert_eq!(tags_vec!["bar"], sorted);
    }

    #[test]
    fn sorting_tags_with_order_defined_by_redis_tags() {
        let user_tags = tags_vec!["foo", "bar", "buz"];
        let redis_tags = tags_vec!["buz", "foo", "bar"];
        let sorted = sort_tags(redis_tags, user_tags);

        assert_eq!(tags_vec!["buz", "foo", "bar"], sorted);
    }

    #[test]
    fn sorted_tags_if_no_data_stores() {
        redis::flushall();

        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Fallible<Vec<String>>| {
                assert_eq!(Vec::<String>::new(), result.unwrap());
            },
        );
    }

    #[test]
    fn sorted_tags_if_data_exist() {
        redis::flushall();
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);

        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Fallible<Vec<String>>| {
                assert_eq!(vec!["zzz", "xxx"], result.unwrap());
            },
        );
    }

    #[test]
    #[should_panic(expected = "WRONGTYPE Operation against a key holding the wrong kind of value")]
    fn get_ordered_tags_with_redis_error() {
        redis::flushall();
        redis::exec_cmd(vec!["SET", "user_tags_1", "foo"]);

        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Fallible<Vec<String>>| {
                result.unwrap();
            },
        );
    }

    #[test]
    fn sort_tags_with_redis_data() {
        redis::flushall();

        // prepare sort order for tags:
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "1", "foo"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);

        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Fallible<Vec<String>>| {
                let redis_tags = result.unwrap();
                let user_tags = tags_vec!["foo", "xxx", "zzz"];
                let sorted = sort_tags(redis_tags, user_tags);

                assert_eq!(tags_vec!["zzz", "xxx", "foo"], sorted);
            },
        );
    }
}
