use super::Redis;
use crate::{config::user_tags_redis_key, errors::Error};
use octo_budget_lib::auth_token::UserId;
use redis::Pipeline;

pub async fn increment_tags(
    user_id: UserId,
    tags: Vec<String>,
    redis: &Redis,
) -> Result<(), Error> {
    let key = user_tags_redis_key(user_id);

    let mut pipeline = Pipeline::with_capacity(tags.len());

    for tag in &tags {
        pipeline.cmd("zincrby").arg(&key).arg("1").arg(tag);
    }

    redis.execute(pipeline).await
}

pub async fn decrement_tags(
    user_id: UserId,
    tags: Vec<String>,
    redis: &Redis,
) -> Result<(), Error> {
    let key = user_tags_redis_key(user_id);

    let mut pipeline = Pipeline::with_capacity(tags.len());

    for tag in &tags {
        pipeline.cmd("zincrby").arg(&key).arg("-1").arg(tag);
    }

    pipeline.cmd("zremrangebyscore").arg(&key).arg("0").arg("0");

    redis.execute(pipeline).await
}

pub async fn read_redis_tags(user_id: UserId, redis: &Redis) -> Result<Vec<String>, Error> {
    let redis_key = user_tags_redis_key(user_id);

    redis::cmd("zrevrange")
        .arg(redis_key)
        .arg("0")
        .arg("-1")
        .query_async(&mut redis.connection())
        .await
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags_vec;
    use redis;

    mod test_redis {
        use crate::redis::{Redis, RedisConnection};
        use serde::export::fmt::Display;

        pub struct Session(Redis);

        impl Session {
            pub async fn new() -> Self {
                let redis = crate::redis::Redis::new().await;

                let _: () = redis::cmd("flushall")
                    .query_async(&mut redis.connection())
                    .await
                    .expect("failed to cleanup redis");

                Self(redis)
            }

            pub async fn zadd<T: redis::ToRedisArgs + Display>(
                &mut self,
                user_id: T,
                score: T,
                tag: T,
            ) {
                let key = format!("user_tags_{}", user_id);
                let mut conn = self.conn();
                let _: () = redis::cmd("zadd")
                    .arg(key)
                    .arg(score)
                    .arg(tag)
                    .query_async(&mut conn)
                    .await
                    .expect("failed to execute zadd");
            }

            pub fn conn(&mut self) -> RedisConnection {
                self.0.connection()
            }

            pub fn redis(&self) -> &Redis {
                &self.0
            }
        }
    }

    #[actix_rt::test]
    async fn sorted_tags_if_no_data_stored() {
        let session = test_redis::Session::new().await;

        let result = read_redis_tags(user_id_1(), session.redis()).await;

        assert_eq!(tags_vec!(), result.unwrap());
    }

    #[actix_rt::test]
    async fn sorted_tags_if_data_exist() {
        let mut session = test_redis::Session::new().await;
        let user_id = "1";

        session.zadd(user_id, "2", "xxx").await;
        session.zadd(user_id, "3", "zzz").await;

        let tags = read_redis_tags(user_id_1(), session.redis())
            .await
            .expect("failed to get tags");

        assert_eq!(tags_vec!["zzz", "xxx"], tags);
    }

    #[actix_rt::test]
    async fn get_ordered_tags_with_redis_error() {
        let mut session = test_redis::Session::new().await;
        let mut conn = session.conn();

        let _: () = redis::cmd("set")
            .arg("user_tags_1")
            .arg("foo")
            .query_async(&mut conn)
            .await
            .unwrap();

        let result = read_redis_tags(user_id_1(), session.redis()).await;
        let error = result.unwrap_err().to_string();

        assert!(
            error.contains("WRONGTYPE: Operation against a key holding the wrong kind of value")
        );
    }

    #[actix_rt::test]
    async fn sort_tags_with_redis_data() {
        use crate::apps::helpers::sort_tags;

        let mut session = test_redis::Session::new().await;
        let user_id = "1";

        session.zadd(user_id, "2", "xxx").await;
        session.zadd(user_id, "1", "foo").await;
        session.zadd(user_id, "3", "zzz").await;

        let redis_tags = read_redis_tags(user_id_1(), session.redis())
            .await
            .expect("failed to get tags");
        let user_tags = tags_vec!["foo", "xxx", "zzz"];
        let sorted = sort_tags(redis_tags, user_tags);

        assert_eq!(vec!["zzz", "xxx", "foo"], sorted);
    }

    #[actix_rt::test]
    async fn increment_tags_happy_path() {
        let mut session = test_redis::Session::new().await;
        let user_id = "1";

        // prepare sort order for tags:
        session.zadd(user_id, "2", "xxx").await;
        session.zadd(user_id, "1", "foo").await;
        session.zadd(user_id, "3", "zzz").await;

        // check result BEFORE incrementing
        let redis_tags = read_redis_tags(user_id_1(), session.redis())
            .await
            .expect("failed to get tags");
        assert_eq!(vec!["zzz", "xxx", "foo"], redis_tags);

        for _ in 0..3 {
            increment_tags(user_id_1(), tags_vec!["foo"], session.redis())
                .await
                .expect("failed to increment");
        }

        // check result AFTER incrementing
        let redis_tags = read_redis_tags(user_id_1(), session.redis())
            .await
            .expect("failed to get tags");
        assert_eq!(vec!["foo", "zzz", "xxx"], redis_tags);
    }

    #[actix_rt::test]
    async fn decrement_tags_happy_path() {
        let mut session = test_redis::Session::new().await;
        let user_id = "1";

        session.zadd(user_id, "5", "xxx").await;
        session.zadd(user_id, "4", "foo").await;
        session.zadd(user_id, "6", "zzz").await;

        // first, let's check initial state
        let tags = read_redis_tags(user_id_1(), session.redis())
            .await
            .expect("failed to get tags");
        assert_eq!(vec!["zzz", "xxx", "foo"], tags);

        // now let's decrement zzz
        decrement_tags(user_id_1(), tags_vec!["zzz"], session.redis())
            .await
            .expect("failed to decrement");
        // and decrement zzz again
        decrement_tags(user_id_1(), tags_vec!["zzz"], session.redis())
            .await
            .expect("failed to decrement");
        // let's check tags order again
        let tags = read_redis_tags(user_id_1(), session.redis())
            .await
            .expect("failed to get tags");
        // zzz is no longer the first one
        assert_eq!(vec!["xxx", "zzz", "foo"], tags);
    }

    #[actix_rt::test]
    async fn decrement_tags_and_delete_zeros_happy_path() {
        let mut session = test_redis::Session::new().await;
        let user_id = "1";

        // prepare sort order for tags:
        session.zadd(user_id, "2", "xxx").await;
        session.zadd(user_id, "1", "foo").await;

        // first, let's check initial state
        let tags = read_redis_tags(user_id_1(), session.redis())
            .await
            .expect("failed to get tags");
        assert_eq!(vec!["xxx", "foo"], tags);

        decrement_tags(user_id_1(), tags_vec!["xxx", "foo"], session.redis())
            .await
            .expect("failed to decrement");

        // let's check tags order again
        let tags = read_redis_tags(user_id_1(), session.redis())
            .await
            .expect("failed to get tags");
        assert_eq!(vec!["xxx"], tags);
    }

    fn user_id_1() -> UserId {
        1.into()
    }
}
