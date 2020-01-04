use crate::db::{schema::auth_user, DatabaseQuery, PooledConnection};
use crate::errors::Error;
use octo_budget_lib::auth_token::UserId;

pub type TagsResult = Result<Vec<String>, failure::Error>;

pub struct GetUserTags {
    user_id: UserId,
}

impl GetUserTags {
    pub fn new(user_id: UserId) -> Self {
        GetUserTags { user_id }
    }
}

impl DatabaseQuery for GetUserTags {
    type Data = Vec<String>;

    fn execute(&self, connection: PooledConnection) -> TagsResult {
        use diesel::prelude::*;

        let owner_user_id: i32 = self.user_id.into();

        let tags = auth_user::table
            .select(auth_user::tags)
            .filter(auth_user::id.eq(owner_user_id))
            .first(&connection)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::UserNotFound(self.user_id),
                err => Error::UnknownDb(err),
            })?;

        Ok(tags)
    }
}
