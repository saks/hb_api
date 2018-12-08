use actix::{Handler, Message};
use failure::Fallible;
use failure_derive::Fail;
use redis_async::{
    resp::{FromResp, RespValue as RedisResponse},
    resp_array,
};

use super::ResponseData;
use crate::db::{schema::auth_user, DbExecutor};

type RedisResult = Result<RedisResponse, actix_redis::Error>;

#[derive(Debug, Fail)]
enum TagsError {
    #[fail(display = "Cannot read sorted tags from redis {}", _0)]
    Redis(#[cause] actix_redis::Error),
}

impl actix_web::error::ResponseError for TagsError {}

pub fn get_ordered_tags_from_redis_msg(user_id: i32) -> actix_redis::Command {
    let redis_key = crate::config::user_tags_redis_key(user_id);
    actix_redis::Command(resp_array!["zrevrange", redis_key, "0", "-1"])
}

pub fn get_ordered_tags(
    (redis_result, user_result): (RedisResult, TagsResult),
) -> Fallible<ResponseData> {
    let redis_tags = redis_response_into_tags(redis_result)?;
    let user_tags = user_result?;

    Ok(ResponseData {
        tags: sort_tags(redis_tags, user_tags),
    })
}

pub fn get_user_tags_from_db_msg(user_id: i32) -> GetUserTagsMessage {
    GetUserTagsMessage { user_id: user_id }
}

pub type TagsResult = Fallible<Vec<String>>;

pub struct GetUserTagsMessage {
    user_id: i32,
}

impl Message for GetUserTagsMessage {
    type Result = TagsResult;
}

impl Handler<GetUserTagsMessage> for DbExecutor {
    type Result = TagsResult;

    fn handle(&mut self, msg: GetUserTagsMessage, _: &mut Self::Context) -> Self::Result {
        use diesel::prelude::*;

        let connection = &self.pool.get()?;

        auth_user::table
            .select(auth_user::tags)
            .filter(auth_user::id.eq(msg.user_id))
            .first(connection)
            .map_err(|e| e.into())
    }
}

fn redis_response_into_tags(result: RedisResult) -> Result<Vec<String>, TagsError> {
    result
        .and_then(|resp| Vec::<String>::from_resp(resp).map_err(|e| e.into()))
        .map_err(|e| TagsError::Redis(e))
}

fn sort_tags(redis_tags: Vec<String>, user_tags: Vec<String>) -> Vec<String> {
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
}
