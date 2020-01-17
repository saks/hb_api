use super::*;
use crate::db::{builders::UserBuilder, ConnectionPool};

#[actix_rt::test]
async fn not_found_err() {
    let pool = ConnectionPool::new();
    let error = pool
        .execute(FindUserByName::new("foo".to_string()))
        .await
        .expect_err("Is not expected to find anything");

    assert_eq!(
        "Failed to find record from table auth_user",
        error.to_string()
    );
}

#[actix_rt::test]
async fn found() {
    let pool = ConnectionPool::new();
    let session = pool.start_session();
    let user = session.create_user(UserBuilder::default());

    let result = pool
        .execute(FindUserByName::new(user.username.to_owned()))
        .await;

    assert!(result.is_ok(), "failed to find user");
    assert_eq!(user, result.unwrap());
}
