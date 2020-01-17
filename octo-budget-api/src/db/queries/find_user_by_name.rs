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
mod tests;
