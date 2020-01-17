use crate::db::{schema::auth_user, DatabaseQuery, PooledConnection};
use crate::errors::{add_table_name, DbResult};
use octo_budget_lib::auth_token::UserId;

pub type TagsResult = DbResult<Vec<String>>;

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
            .map_err(add_table_name("auth_user"))?;

        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::{builders::UserBuilder, ConnectionPool},
        tests::DbSession,
    };

    #[actix_rt::test]
    async fn get_user_tags() {
        let session = DbSession::new();

        let tags = vec!["foo", "bar"];
        let user = session.create_user(UserBuilder::default().tags(tags.to_owned()));

        let result_tags = find(user.id.into()).await.expect("Failed to find record");

        assert_eq!(tags, result_tags);
    }

    #[actix_rt::test]
    async fn user_not_found_error() {
        let error = find(123.into())
            .await
            .expect_err("Is not expected to find anything");

        assert_eq!(
            "Failed to find record from table auth_user",
            error.to_string()
        );
    }

    async fn find(user_id: UserId) -> DbResult<Vec<String>> {
        let conn_pool = ConnectionPool::new();
        let query = GetUserTags::new(user_id);

        conn_pool.execute(query).await
    }
}
