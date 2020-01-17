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
mod tests {
    use super::*;
    use crate::{
        db::{builders::UserBuilder, queries::GetUserTags, ConnectionPool},
        tags_vec,
    };

    #[actix_rt::test]
    async fn set_user_tags() {
        let conn_pool = ConnectionPool::new();
        let session = conn_pool.start_session();

        let user = session.create_user(UserBuilder::default().tags(vec!["foo", "bar"]));

        let new_tags = tags_vec!["zzz", "xxx"];
        conn_pool
            .execute(SetUserTags::new(user.id.into(), new_tags.to_owned()))
            .await
            .expect("Failed to set user tags");

        let result_tags = conn_pool
            .execute(GetUserTags::new(user.id.into()))
            .await
            .expect("Failed to get user tags");

        assert_eq!(new_tags, result_tags);
    }
}
