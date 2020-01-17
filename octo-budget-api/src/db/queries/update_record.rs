use bigdecimal::BigDecimal;
use octo_budget_lib::auth_token::UserId;

use crate::apps::forms::record::FormData;
use crate::db::{DatabaseQuery, PooledConnection};
use crate::errors::{DbError, DbResult};

pub struct UpdateRecord {
    amount: BigDecimal,
    amount_currency: String,
    tags: Vec<String>,
    transaction_type: String,
    user_id: UserId,
    id: i32,
}

impl UpdateRecord {
    pub fn new(id: i32, data: &FormData, user_id: UserId) -> Self {
        Self {
            amount: data.amount.clone(),
            amount_currency: data.amount_currency.clone(),
            tags: data.tags.clone(),
            transaction_type: data.transaction_type.clone(),
            user_id,
            id,
        }
    }
}

impl DatabaseQuery for UpdateRecord {
    type Data = ();

    fn execute(&self, connection: PooledConnection) -> DbResult<()> {
        use crate::db::schema::records_record::dsl::*;
        use diesel::prelude::*;

        let current_user_id: i32 = self.user_id.into();

        let target = records_record
            .filter(user_id.eq(current_user_id))
            .filter(id.eq(self.id));

        let result = diesel::update(target)
            .set((
                amount.eq(&self.amount),
                amount_currency.eq(&self.amount_currency),
                tags.eq(&self.tags),
                transaction_type.eq(&self.transaction_type),
            ))
            .execute(&connection);

        match result {
            Ok(1) => Ok(()),
            Ok(0) => Err(DbError::NotUpdated("records_record", self.id)),
            Ok(_) => Err(DbError::UnexpectedResult("More than one record updated")),
            Err(err) => Err(DbError::Unknown(err)),
        }
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests;
