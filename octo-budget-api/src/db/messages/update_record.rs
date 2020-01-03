use std::result;

use crate::errors::Error;
use bigdecimal::BigDecimal;
use octo_budget_lib::auth_token::UserId;

use crate::apps::forms::record::FormData;
use crate::db::{DatabaseQuery, PooledConnection};

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

    fn execute(&self, connection: PooledConnection) -> Result<(), failure::Error> {
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

        let res = match result {
            Ok(1) => Ok(()),
            Ok(0) => Err(Error::RecordNotUpdated("records_record", self.id)),
            Ok(_) => Err(Error::UnknownMsg("More than one record updated")),
            Err(err) => Err(Error::UnknownDb(err)),
        }?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::builders::UserBuilder;
    use crate::tests::DbSession;

    #[actix_rt::test]
    async fn no_record_updated() {
        let conn_pool = crate::db::ConnectionPool::new();
        let query = UpdateRecord {
            amount: BigDecimal::from(10.0),
            amount_currency: "CAD".into(),
            tags: vec![],
            transaction_type: "INC".into(),
            user_id: 1.into(),
            id: 1,
        };

        let res = conn_pool.execute(query).await;

        assert!(res.is_err(), "result is not an error");
        assert_eq!(
            "Cannot update records_record with id: `1'",
            format!("{}", res.unwrap_err())
        );
    }

    #[actix_rt::test]
    async fn happy_path() {
        let conn_pool = crate::db::ConnectionPool::new();
        let session = DbSession::new();
        let user = session.create_user(UserBuilder::default().password("dummy password"));
        let records = session.create_records2(user.id, 1);

        let query = UpdateRecord {
            amount: BigDecimal::from(10.0),
            amount_currency: "CAD".into(),
            tags: vec![],
            transaction_type: "INC".into(),
            user_id: user.id.into(),
            id: records[0].id,
        };

        let res = conn_pool.execute(query).await;

        assert!(res.is_ok(), "result is not Ok, {:?}", res);
        assert_eq!((), res.unwrap());
    }

    #[actix_rt::test]
    async fn check_update_result() {
        use crate::db::messages::GetRecords;

        let conn_pool = crate::db::ConnectionPool::new();
        let mut session = DbSession::new();
        let user = session.create_user(UserBuilder::default().password("dummy password"));
        let record = session.create_record2(user.id);

        let query = UpdateRecord {
            amount: BigDecimal::from(10.0),
            amount_currency: "USD".into(),
            tags: vec!["foo".to_string()],
            transaction_type: "INC".into(),
            user_id: user.id.into(),
            id: record.id,
        };

        let res = conn_pool.execute(query).await;

        // make sure that update was OK:
        assert!(res.is_ok(), "result is not Ok, {:?}", res);
        assert_eq!((), res.unwrap());

        // verify changes in the DB:
        let res = conn_pool
            .execute(GetRecords {
                user_id: user.id,
                page: 1,
                per_page: 1,
            })
            .await;
        assert!(res.is_ok(), "result is not Ok, {:?}", res);

        let data = res.unwrap();
        let updated_record = data.results.get(0).expect("data has no records");

        assert_ne!(record.amount, updated_record.amount);
        assert_ne!(record.amount_currency, updated_record.amount_currency);
        assert_ne!(record.tags, updated_record.tags);
        assert_ne!(record.transaction_type, updated_record.transaction_type);
    }
}
