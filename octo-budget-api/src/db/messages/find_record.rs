use crate::db::{models::Record, DatabaseQuery, PooledConnection};
use crate::errors::Error;
use octo_budget_lib::auth_token::UserId;

pub struct FindRecord {
    user_id: UserId,
    id: i32,
}

impl FindRecord {
    pub fn new(id: i32, user_id: UserId) -> Self {
        Self { id, user_id }
    }
}

impl DatabaseQuery for FindRecord {
    type Data = Record;

    fn execute(&self, connection: PooledConnection) -> Result<Record, failure::Error> {
        use crate::db::schema::records_record::dsl::*;
        use diesel::prelude::*;

        let owner_user_id: i32 = self.user_id.into();

        let record = records_record
            .filter(user_id.eq(owner_user_id))
            .filter(id.eq(self.id))
            .first(&connection)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::UserNotFound(self.user_id),
                err => Error::UnknownDb(err),
            })?;

        Ok(record)
    }
}

// TODO: add tests
// 1. test that it filters records by owner id
