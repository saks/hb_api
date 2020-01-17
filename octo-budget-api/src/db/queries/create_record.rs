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
mod tests {
    use super::*;

    use crate::{
        db::{builders::UserBuilder, queries::FindRecord, ConnectionPool},
        tags_vec,
        tests::DbSession,
    };

    #[actix_rt::test]
    async fn find_by_user_id() {
        let conn_pool = ConnectionPool::new();
        let session = DbSession::new();

        let user = session.create_user(UserBuilder::default());

        let created_at = NaiveDateTime::from_timestamp(1, 0);
        let amount_currency = "CAD".to_string();
        let amount = BigDecimal::from(112233);
        let tags = tags_vec!["foo", "bar"];
        let transaction_type = "EXP".to_string();

        let query = CreateRecord {
            amount: amount.to_owned(),
            amount_currency: amount_currency.to_owned(),
            created_at,
            tags: tags.to_owned(),
            transaction_type: transaction_type.to_owned(),
            user_id: user.id,
        };

        let id = conn_pool
            .execute(query)
            .await
            .expect("Failed to find record");

        let record = conn_pool
            .execute(FindRecord::new(id, user.id.into()))
            .await
            .expect("Failed to find record");

        assert_eq!(id, record.id);
        assert_eq!(amount, record.amount);
        assert_eq!(amount_currency, record.amount_currency);
        assert_eq!(created_at, record.created_at);
        assert_eq!(tags, record.tags);
    }
}
