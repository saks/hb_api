use actix::{Handler, Message};
use actix_redis::Command;
use failure::Fallible;
use failure_derive::Fail;
use redis_async::{
    resp::{FromResp, RespValue as RedisResponse},
    resp_array,
};

use super::TagsData;
use crate::db::{schema::auth_user, DbExecutor};

type RedisResult = Result<RedisResponse, actix_redis::Error>;

#[derive(Debug, Fail)]
enum TagsError {
    #[fail(display = "Cannot read sorted tags from redis {}", _0)]
    Redis(#[cause] actix_redis::Error),
}

impl actix_web::error::ResponseError for TagsError {}

pub fn get_ordered_tags_from_redis_msg(user_id: i32) -> Command {
    let redis_key = crate::config::user_tags_redis_key(user_id);
    Command(resp_array!["zrevrange", redis_key, "0", "-1"])
}

pub fn get_ordered_tags(
    (redis_result, user_result): (RedisResult, TagsResult),
) -> Fallible<TagsData> {
    let redis_tags = redis_response_into_tags(redis_result)?;
    let user_tags = user_result?;

    Ok(TagsData {
        tags: sort_tags(redis_tags, user_tags),
    })
}

pub fn get_user_tags_from_db_msg(user_id: i32) -> GetUserTagsMessage {
    GetUserTagsMessage { user_id }
}

pub type TagsResult = Fallible<Vec<String>>;

pub struct GetUserTagsMessage {
    user_id: i32,
}

pub struct SetUserTags {
    pub tags: Vec<String>,
    pub user_id: i32,
}

impl Message for SetUserTags {
    type Result = TagsResult;
}

impl Handler<SetUserTags> for DbExecutor {
    type Result = TagsResult;

    fn handle(&mut self, msg: SetUserTags, _: &mut Self::Context) -> Self::Result {
        use diesel::prelude::*;

        let connection = &self.pool.get()?;

        let SetUserTags { user_id, tags } = msg;

        let target = auth_user::table.filter(auth_user::id.eq(user_id));
        diesel::update(target)
            .set(auth_user::tags.eq(tags))
            .execute(&*connection)?;

        Ok(vec![])
    }
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
        .map_err(TagsError::Redis)
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
    use crate::tests::redis;

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
        redis::handle_message(get_ordered_tags_from_redis_msg(1), |res| {
            let result = Vec::<String>::from_resp(res).unwrap();
            assert_eq!(tags_vec![], result);
        });
    }

    #[test]
    fn sorted_tags_if_data_exist() {
        redis::flushall();
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);

        redis::handle_message(get_ordered_tags_from_redis_msg(1), |res| {
            let result = Vec::<String>::from_resp(res).unwrap();
            assert_eq!(tags_vec!["zzz", "xxx"], result);
        });
    }

    #[test]
    #[should_panic(expected = "WRONGTYPE Operation against a key holding the wrong kind of value")]
    fn get_ordered_tags_with_redis_error() {
        redis::flushall();
        redis::exec_cmd(vec!["SET", "user_tags_1", "foo"]);

        redis::handle_message(get_ordered_tags_from_redis_msg(1), |res| {
            let user_result = Ok(tags_vec![]);
            let redis_result = Ok(res);

            get_ordered_tags((redis_result, user_result)).unwrap();
        });
    }

    #[test]
    fn get_ordered_tags_with_redis_data() {
        redis::flushall();

        // prepare sort order for tags:
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "1", "foo"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);

        redis::handle_message(get_ordered_tags_from_redis_msg(1), |redis_res| {
            let redis_result = Ok(redis_res);
            let user_result = Ok(tags_vec!["foo", "xxx", "zzz"]);

            let result = get_ordered_tags((redis_result, user_result)).unwrap();

            assert_eq!(tags_vec!["zzz", "xxx", "foo"], result.tags);
        });
    }
}
