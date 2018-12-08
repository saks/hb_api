use actix::{Handler, Message};
use actix_redis::Command as RedisCommand;
use actix_redis::Error as RedisError;
use actix_web::Error as WebError;
use failure::Error;
use octo_budget_lib::auth_token::AuthToken;
use redis_async::resp::RespValue as RedisResponse;
use redis_async::resp_array;

use crate::db::{schema::auth_user, DbExecutor};

pub fn get_ordered_tags_from_redis_msg(token: &AuthToken) -> RedisCommand {
    let redis_key = crate::config::user_tags_redis_key(token.user_id);
    RedisCommand(resp_array!["zrevrange", redis_key, "0", "-1"])
}

pub fn get_ordered_tags(
    _user_tags: Vec<String>,
    _redis_tags: Result<RedisResponse, RedisError>,
) -> Result<Vec<String>, WebError> {
    Ok(Vec::new())
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

    fn handle(&mut self, mut msg: GetUserTagsMessage, _: &mut Self::Context) -> Self::Result {
        use diesel::prelude::*;

        let connection = &self.pool.get()?;

        let user_id = msg.user_id.clone();
        msg.user_id = 1111;

        auth_user::table
            .select(auth_user::tags)
            .filter(auth_user::id.eq(user_id))
            .first(connection)
            .map_err(|e| e.into())
    }
}
