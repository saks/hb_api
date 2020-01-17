use super::*;
use crate::{
    db::{builders::UserBuilder, queries::GetUserTags, ConnectionPool},
    tags_vec,
};

#[actix_rt::test]
async fn set_user_tags() {
    let conn_pool = ConnectionPool::new();
    let session = conn_pool.start_session();

    let user = session.create_user(UserBuilder::default().tags(vec!["foo", "bar"]));

    let new_tags = tags_vec!["zzz", "xxx"];
    conn_pool
        .execute(SetUserTags::new(user.id.into(), new_tags.to_owned()))
        .await
        .expect("Failed to set user tags");

    let result_tags = conn_pool
        .execute(GetUserTags::new(user.id.into()))
        .await
        .expect("Failed to get user tags");

    assert_eq!(new_tags, result_tags);
}
