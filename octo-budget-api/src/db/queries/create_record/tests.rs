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
