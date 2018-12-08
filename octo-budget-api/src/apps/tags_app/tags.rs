use actix::{Handler, Message};
use actix_redis::{Command as RedisCommand, Error as RedisError};
use actix_web::{error::ErrorInternalServerError as InternalServerError, Error as WebError};
use failure::Error;
use octo_budget_lib::auth_token::AuthToken;
use redis_async::{
    resp::{FromResp, RespValue as RedisResponse},
    resp_array,
};
use serde_derive::Serialize;

use crate::db::{schema::auth_user, DbExecutor};

type RedisResult = Result<RedisResponse, RedisError>;

fn redis_response_into_tags(result: RedisResult) -> Result<Vec<String>, WebError> {
    result
        .and_then(|x| Vec::<String>::from_resp(x).map_err(|e| e.into()))
        .map_err(|_| InternalServerError("Cannot read tags from redis"))
}

#[derive(Serialize, Default)]
pub struct ResponseData {
    tags: Vec<String>,
}

pub fn get_ordered_tags_from_redis_msg(token: &AuthToken) -> RedisCommand {
    let redis_key = crate::config::user_tags_redis_key(token.user_id);
    RedisCommand(resp_array!["zrevrange", redis_key, "0", "-1"])
}

pub fn get_ordered_tags(
    (redis_result, user_result): (RedisResult, Result<Vec<String>, Error>),
) -> Result<ResponseData, WebError> {
    let user_tags = user_result?;
    let redis_tags = redis_response_into_tags(redis_result)?;

    let mut data = ResponseData::default();

    if user_tags.is_empty() {
        return Ok(data);
    }

    for item in redis_tags {
        if user_tags.contains(&item) {
            data.tags.push(item);
        }
    }

    for tag in user_tags {
        if !data.tags.contains(&tag) {
            data.tags.push(tag);
        }
    }

    Ok(data)
}

pub fn get_user_tags_from_db_msg(token: &AuthToken) -> GetUserTagsMessage {
    GetUserTagsMessage {
        user_id: token.user_id,
    }
}

pub type UserTagsResult = Result<Vec<String>, Error>;

pub struct GetUserTagsMessage {
    user_id: i32,
}

impl Message for GetUserTagsMessage {
    type Result = UserTagsResult;
}

impl Handler<GetUserTagsMessage> for DbExecutor {
    type Result = UserTagsResult;

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
