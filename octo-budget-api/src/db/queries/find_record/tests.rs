use super::*;
use crate::{
    db::{builders::UserBuilder, ConnectionPool},
    tests::DbSession,
};

#[actix_rt::test]
async fn find_by_user_id() {
    let session = DbSession::new();

    let user = session.create_user(UserBuilder::default());
    let record = session.create_record2(user.id);

    let result_record = find(record.id, user.id.into())
        .await
        .expect("Failed to find record");

    assert_eq!(record.id, result_record.id);
}

#[actix_rt::test]
async fn does_not_return_record_of_other_user() {
    let session = DbSession::new();

    let owner = session.create_user(UserBuilder::default().username("foo"));
    let other_user = session.create_user(UserBuilder::default().username("bar"));
    let record = session.create_record2(other_user.id);

    let error = find(record.id, owner.id.into())
        .await
        .expect_err("Is not expected to find anything");

    assert_eq!(
        "Failed to find record from table records_record",
        error.to_string()
    );
}

#[actix_rt::test]
async fn filters_by_id() {
    let session = DbSession::new();

    let owner = session.create_user(UserBuilder::default().username("foo"));
    let record = session.create_record2(owner.id);

    let error = find(record.id + 1, owner.id.into())
        .await
        .expect_err("Is not expected to find anything");

    assert_eq!(
        "Failed to find record from table records_record",
        error.to_string()
    );
}

async fn find(id: i32, user_id: UserId) -> DbResult<Record> {
    let conn_pool = ConnectionPool::new();
    let query = FindRecord::new(id, user_id);

    conn_pool.execute(query).await
}
