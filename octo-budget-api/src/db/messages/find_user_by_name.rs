use diesel::prelude::*;
use failure::Error;
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

    fn execute(&self, connection: PooledConnection) -> Result<Self::Data, Error> {
        let user = auth_user::table
            .filter(auth_user::username.eq(&self.0))
            .first(&connection)?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::builders::UserBuilder;

    #[actix_rt::test]
    async fn not_found_err() {
        let result = find_by_name("foo".to_string()).await;

        assert_eq!("Err(NotFound)", format!("{:?}", result));
    }

    #[actix_rt::test]
    async fn found() {
        let session = crate::tests::DbSession::new();
        let user = session.create_user(UserBuilder::default());

        let result = find_by_name(user.username.to_owned()).await;

        assert!(result.is_ok(), "failed to find user");
        assert_eq!(user, result.unwrap());
    }

    async fn find_by_name(username: String) -> Result<AuthUser, Error> {
        let conn_pool = crate::db::ConnectionPool::new();
        let query = FindUserByName::new(username);

        conn_pool.execute(query).await
    }
}
