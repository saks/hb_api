use crate::db::{schema::auth_user, DatabaseQuery, PooledConnection};
use crate::errors::{DbError, DbResult};
use octo_budget_lib::auth_token::UserId;
use std::sync::Arc;

type DataType = Arc<Vec<String>>;
pub type TagsResult = DbResult<DataType>;

pub struct SetUserTags {
    tags: DataType,
    user_id: UserId,
}

impl SetUserTags {
    pub fn new(user_id: UserId, tags: Vec<String>) -> Self {
        let tags = tags.into();
        Self { user_id, tags }
    }
}

impl DatabaseQuery for SetUserTags {
    type Data = DataType;

    fn execute(&self, connection: PooledConnection) -> TagsResult {
        use diesel::prelude::*;

        let owner_user_id: i32 = self.user_id.into();
        let target = auth_user::table.filter(auth_user::id.eq(owner_user_id));

        diesel::update(target)
            .set(auth_user::tags.eq(&*self.tags))
            .execute(&connection)
            .map_err(DbError::Unknown)?;

        Ok(self.tags.clone())
    }
}

#[cfg(test)]
mod tests;
