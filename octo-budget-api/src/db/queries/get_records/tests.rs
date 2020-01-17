use super::*;
use crate::{
    db::{builders::UserBuilder, ConnectionPool},
    tests::DbSession,
};

#[actix_rt::test]
async fn empty_result() {
    let conn_pool = ConnectionPool::new();
    let query = GetRecords {
        page: 1,
        per_page: 10,
        user_id: 123,
    };

    let data = conn_pool
        .execute(query)
        .await
        .expect("failed to get records");
    assert_eq!(0, data.total);
    assert_eq!(false, data.next);
    assert_eq!(false, data.previous);
    assert!(data.results.is_empty());
}

#[actix_rt::test]
async fn first_page_result() {
    let mut session = DbSession::new();
    let user = session.create_user(UserBuilder::default().password("dummy password"));
    session.create_records(user.id, 12);

    let query = GetRecords {
        page: 1,
        per_page: 10,
        user_id: user.id,
    };
    let conn_pool = ConnectionPool::new();

    let data = conn_pool
        .execute(query)
        .await
        .expect("failed to get records");

    assert_eq!(12, data.total);
    assert_eq!(false, data.previous);
    assert_eq!(true, data.next);
    assert_eq!(10, data.results.len());
}

#[actix_rt::test]
async fn second_page_result() {
    let mut session = DbSession::new();
    let user = session.create_user(UserBuilder::default().password("dummy password"));
    session.create_records(user.id, 12);

    let query = GetRecords {
        page: 2,
        per_page: 10,
        user_id: user.id,
    };
    let conn_pool = ConnectionPool::new();

    let data = conn_pool
        .execute(query)
        .await
        .expect("failed to get records");

    assert_eq!(12, data.total);
    assert_eq!(true, data.previous);
    assert_eq!(false, data.next);
    assert_eq!(2, data.results.len());
}

#[actix_rt::test]
async fn records_for_correct_user() {
    let mut session = DbSession::new();
    let user1 = session.create_user(
        UserBuilder::default()
            .username("user1")
            .password("dummy password"),
    );
    session.create_records(user1.id, 2);

    let user2 = session.create_user(
        UserBuilder::default()
            .username("user2")
            .password("dummy password"),
    );
    session.create_records(user2.id, 2);

    let conn_pool = ConnectionPool::new();
    let query = GetRecords {
        page: 1,
        per_page: 10,
        user_id: user1.id,
    };

    let data = conn_pool
        .execute(query)
        .await
        .expect("failed to get records");

    assert_eq!(2, data.total);
    assert_eq!(false, data.previous);
    assert_eq!(false, data.next);
    assert_eq!(2, data.results.len());
}
