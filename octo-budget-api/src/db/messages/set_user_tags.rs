use crate::db::{schema::auth_user, DbExecutor};
use crate::errors::Error;
use actix::{Handler, Message};

pub type TagsResult = Result<Vec<String>, Error>;

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

        let connection = &self.pool.get()?;

        let SetUserTags { user_id, tags } = msg;

        let target = auth_user::table.filter(auth_user::id.eq(user_id));
        diesel::update(target)
            .set(auth_user::tags.eq(&tags))
            .execute(connection)
            .map_err(Error::UnknownDb)?;

        Ok(tags)
    }
}
