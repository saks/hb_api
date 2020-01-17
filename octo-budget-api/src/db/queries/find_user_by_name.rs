use crate::errors::{add_table_name, DbResult};
use diesel::prelude::*;
use std::convert::Into;

use crate::db::{models::AuthUser, schema::auth_user, DatabaseQuery, PooledConnection};

pub struct FindUserByName(String);

impl FindUserByName {
    pub fn new(username: impl Into<String>) -> Self {
        Self(username.into())
    }
}

impl DatabaseQuery for FindUserByName {
    type Data = AuthUser;

    fn execute(&self, connection: PooledConnection) -> DbResult<Self::Data> {
        let user = auth_user::table
            .filter(auth_user::username.eq(&self.0))
            .first(&connection)
            .map_err(add_table_name("auth_user"))?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{builders::UserBuilder, ConnectionPool};

    #[actix_rt::test]
    async fn not_found_err() {
        let pool = ConnectionPool::new();
        let error = pool
            .execute(FindUserByName::new("foo".to_string()))
            .await
            .expect_err("Is not expected to find anything");

        assert_eq!(
            "Failed to find record from table auth_user",
            error.to_string()
        );
    }

    #[actix_rt::test]
    async fn found() {
        let pool = ConnectionPool::new();
        let session = pool.start_session();
        let user = session.create_user(UserBuilder::default());

        let result = pool
            .execute(FindUserByName::new(user.username.to_owned()))
            .await;

        assert!(result.is_ok(), "failed to find user");
        assert_eq!(user, result.unwrap());
    }
}
