use super::*;
use crate::db::builders::UserBuilder;

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
        comment: String::new(),
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
    let session = conn_pool.start_session();

    let user = session.create_user(UserBuilder::default().password("dummy password"));
    let records = session.create_records2(user.id, 1);

    let query = UpdateRecord {
        amount: BigDecimal::from(10.0),
        amount_currency: "CAD".into(),
        tags: vec![],
        transaction_type: "INC".into(),
        user_id: user.id.into(),
        id: records[0].id,
        comment: "".into(),
    };

    let res = conn_pool.execute(query).await;

    assert!(res.is_ok(), "result is not Ok, {:?}", res);
    assert_eq!((), res.unwrap());
}

#[actix_rt::test]
async fn check_update_result() {
    use crate::db::queries::GetRecords;

    let conn_pool = crate::db::ConnectionPool::new();
    let session = conn_pool.start_session();

    let user = session.create_user(UserBuilder::default().password("dummy password"));
    let record = session.create_record2(user.id);

    let query = UpdateRecord {
        amount: BigDecimal::from(10.0),
        amount_currency: "USD".into(),
        tags: vec!["foo".to_string()],
        transaction_type: "INC".into(),
        user_id: user.id.into(),
        id: record.id,
        comment: "".into(),
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
