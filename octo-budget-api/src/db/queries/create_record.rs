use bigdecimal::BigDecimal;
use chrono::{NaiveDateTime, Utc};
use octo_budget_lib::auth_token::UserId;

use crate::apps::forms::record::FormData;
use crate::db::{models::Record, DatabaseQuery, PooledConnection};
use crate::errors::DbResult;

pub struct CreateRecord {
    amount: BigDecimal,
    amount_currency: String,
    created_at: NaiveDateTime,
    tags: Vec<String>,
    transaction_type: String,
    user_id: i32,
}

impl CreateRecord {
    pub fn new(data: &FormData, user_id: UserId) -> Self {
        let created_at = Utc::now().naive_local();
        let user_id: i32 = user_id.into();

        Self {
            amount: data.amount.clone(),
            amount_currency: data.amount_currency.clone(),
            tags: data.tags.clone(),
            transaction_type: data.transaction_type.clone(),
            user_id,
            created_at,
        }
    }
}

impl DatabaseQuery for CreateRecord {
    type Data = i32;

    fn execute(&self, connection: PooledConnection) -> DbResult<i32> {
        use crate::db::schema::records_record::dsl::*;
        use diesel::prelude::*;
        use diesel::*;

        let record: Record = insert_into(records_record)
            .values((
                amount.eq(&self.amount),
                amount_currency.eq(&self.amount_currency),
                created_at.eq(self.created_at),
                tags.eq(&self.tags),
                transaction_type.eq(&self.transaction_type),
                user_id.eq(self.user_id),
            ))
            .get_result(&connection)?;

        Ok(record.id)
    }
}

#[cfg(test)]
mod tests;
