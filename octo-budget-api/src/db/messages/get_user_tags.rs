use crate::db::{schema::auth_user, DbExecutor};
use crate::errors::Error;
use actix::{Handler, Message};
use octo_budget_lib::auth_token::UserId;

pub type TagsResult = Result<Vec<String>, Error>;

pub struct GetUserTags {
    user_id: UserId,
}

impl Message for GetUserTags {
    type Result = TagsResult;
}

impl GetUserTags {
    pub fn new(user_id: UserId) -> Self {
        GetUserTags { user_id }
    }
}

impl Handler<GetUserTags> for DbExecutor {
    type Result = TagsResult;

    fn handle(&mut self, msg: GetUserTags, _: &mut Self::Context) -> Self::Result {
        use diesel::prelude::*;

        let connection = &self.pool.get()?;
        let user_id: i32 = msg.user_id.into();

        auth_user::table
            .select(auth_user::tags)
            .filter(auth_user::id.eq(user_id))
            .first(connection)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::UserNotFound(msg.user_id),
                err => Error::UnknownDb(err),
            })
    }
}
