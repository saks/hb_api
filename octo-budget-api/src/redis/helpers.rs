use super::Redis;
use crate::{config::user_tags_redis_key, errors::Error};
use octo_budget_lib::auth_token::UserId;
use octo_redis::{cmd, Pipeline};

pub async fn increment_tags(user_id: UserId, tags: Vec<String>, redis: Redis) -> Result<(), Error> {
    let key = user_tags_redis_key(user_id);

    let mut pipeline = Pipeline::new();

    for tag in &tags {
        pipeline.add_command(&cmd("zincrby").arg(&key).arg("1").arg(tag));
    }

    let _ = pipeline
        .send::<Vec<String>>(redis.get_ref().to_owned())
        .await?;

    Ok(())
}

pub async fn decrement_tags(user_id: UserId, tags: Vec<String>, redis: Redis) -> Result<(), Error> {
    let key = user_tags_redis_key(user_id);

    let mut pipeline = Pipeline::new();

    for tag in &tags {
        pipeline.add_command(&cmd("zincrby").arg(&key).arg("-1").arg(tag));
    }

    pipeline.add_command(&cmd("zremrangebyscore").arg(&key).arg("0").arg("0"));

    let _ = pipeline
        .send::<Vec<String>>(redis.get_ref().to_owned())
        .await?;

    Ok(())
}

pub async fn read_redis_tags(user_id: UserId, redis: Redis) -> Result<Vec<String>, Error> {
    let redis_key = user_tags_redis_key(user_id);

    cmd("zrevrange")
        .arg(redis_key)
        .arg("0")
        .arg("-1")
        .send::<Vec<String>>(redis.get_ref().to_owned())
        .await
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags_vec;
    use actix::prelude::*;
    use actix_web::web::Data;
    use futures::future;
    use futures03::{FutureExt as _, TryFutureExt as _};
    use redis;

    mod test_redis {
        use serde::export::fmt::Display;

        pub struct Session(redis::Connection);

        impl Session {
            pub fn new() -> Self {
                let url = crate::config::redis_url();
                let client = redis::Client::open(url.as_str()).expect("failed to create client");
                let mut conn = client.get_connection().expect("failed to connect");

                redis::cmd("flushall").execute(&mut conn);

                Self(conn)
            }

            pub fn zadd<T: redis::ToRedisArgs + Display>(&mut self, user_id: T, score: T, tag: T) {
                let key = format!("user_tags_{}", user_id);
                redis::cmd("zadd")
                    .arg(key)
                    .arg(score)
                    .arg(tag)
                    .execute(self.conn());
            }

            pub fn conn(&mut self) -> &mut redis::Connection {
                &mut self.0
            }
        }

        impl Drop for Session {
            fn drop(&mut self) {
                redis::cmd("flushall").execute(&mut self.0);
            }
        }
    }

    #[test]
    fn sorted_tags_if_no_data_stored() {
        test_redis::Session::new();

        System::run(|| {
            let addr = Data::new(crate::redis::start());
            let fut = read_redis_tags(1.into(), addr)
                .unit_error()
                .boxed()
                .compat();

            actix::spawn(fut.then(|res| {
                assert_eq!(Vec::<String>::new(), res.unwrap().unwrap());

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .expect("failed to run system");
    }

    #[test]
    fn sorted_tags_if_data_exist() {
        let mut session = test_redis::Session::new();

        redis::cmd("zadd")
            .arg("user_tags_1")
            .arg("2")
            .arg("xxx")
            .execute(session.conn());
        redis::cmd("zadd")
            .arg("user_tags_1")
            .arg("3")
            .arg("zzz")
            .execute(session.conn());

        System::run(|| {
            let addr = Data::new(crate::redis::start());
            let fut = read_redis_tags(1.into(), addr)
                .unit_error()
                .boxed()
                .compat();

            actix::spawn(fut.then(|res| {
                let tags: Vec<String> = res.unwrap().unwrap();
                assert_eq!(vec!["zzz", "xxx"], tags);

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .expect("failed to run system");
    }

    #[should_panic = "Redis(Redis(WRONGTYPE: Operation against a key holding the wrong kind of value))"]
    #[test]
    fn get_ordered_tags_with_redis_error() {
        let mut session = test_redis::Session::new();

        redis::cmd("set")
            .arg("user_tags_1")
            .arg("foo")
            .execute(session.conn());

        System::run(|| {
            let addr = Data::new(crate::redis::start());
            let fut = read_redis_tags(1.into(), addr)
                .unit_error()
                .boxed()
                .compat();

            actix::spawn(fut.then(|res| {
                let _: Vec<String> = res.unwrap().unwrap();

                System::current().stop();
                future::result(Ok(()))
            }));
        })
        .expect("failed to run system");
    }

    //     #[test]
    //     fn sort_tags_with_redis_data() {
    //         redis::flushall();
    //
    //         // prepare sort order for tags:
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "1", "foo"]);
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);
    //
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 let redis_tags = result.unwrap();
    //                 let user_tags = tags_vec!["foo", "xxx", "zzz"];
    //                 let sorted = sort_tags(redis_tags, user_tags);
    //
    //                 assert_eq!(tags_vec!["zzz", "xxx", "foo"], sorted);
    //             },
    //         );
    //     }
    //
    //     #[test]
    //     fn increment_tags_happy_path() {
    //         redis::flushall();
    //
    //         // prepare sort order for tags:
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "1", "foo"]);
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);
    //
    //         // check result BEFORE incrementing
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 assert_eq!(vec!["zzz", "xxx", "foo"], result.unwrap());
    //             },
    //         );
    //
    //         for _ in 0..3 {
    //             let fut = increment_tags(1, crate::tags_vec!["foo"], redis::get_connection());
    //             tests::run_future(Compat::new(fut), |res| assert!(res.is_ok()));
    //         }
    //
    //         // check result AFTER incrementing
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 assert_eq!(vec!["foo", "zzz", "xxx"], result.unwrap());
    //             },
    //         );
    //     }

    #[test]
    fn decrement_tags_happy_path() {
        let mut session = test_redis::Session::new();
        let user_id = "1";

        session.zadd(user_id, "5", "xxx");
        session.zadd(user_id, "4", "foo");
        session.zadd(user_id, "6", "zzz");

        System::run(|| {
            let addr = Data::new(crate::redis::start());

            // first, let's check initial state
            let fut = read_redis_tags(1.into(), addr)
                .unit_error()
                .boxed()
                .compat()
                .and_then(move |res| {
                    let addr = Data::new(crate::redis::start());
                    let tags: Vec<String> = res.unwrap();
                    assert_eq!(vec!["zzz", "xxx", "foo"], tags);

                    // now let's decrement zzz
                    decrement_tags(1.into(), tags_vec!["zzz"], addr)
                        .unit_error()
                        .boxed()
                        .compat()
                })
                .and_then(move |res| {
                    res.expect("failed to decrement tags");

                    let addr = Data::new(crate::redis::start());
                    // and decrement zzz again
                    decrement_tags(1.into(), tags_vec!["zzz"], addr)
                        .unit_error()
                        .boxed()
                        .compat()
                })
                .and_then(move |res| {
                    res.expect("failed to decrement tags");

                    let addr = Data::new(crate::redis::start());
                    // let's check tags order again
                    read_redis_tags(1.into(), addr)
                        .unit_error()
                        .boxed()
                        .compat()
                })
                .and_then(|res| {
                    let tags: Vec<String> = res.unwrap();
                    // zzz is no longer the first one
                    assert_eq!(vec!["xxx", "zzz", "foo"], tags);

                    System::current().stop();
                    future::result(Ok(()))
                });

            actix::spawn(fut);
        })
        .expect("failed to run system");
    }

    #[test]
    fn decrement_tags_and_delete_zeros_happy_path() {
        let mut session = test_redis::Session::new();
        let user_id = "1";

        // prepare sort order for tags:
        session.zadd(user_id, "2", "xxx");
        session.zadd(user_id, "1", "foo");

        System::run(|| {
            let addr = Data::new(crate::redis::start());

            // first, let's check initial state
            let fut = read_redis_tags(1.into(), addr)
                .unit_error()
                .boxed()
                .compat()
                .and_then(move |res| {
                    let addr = Data::new(crate::redis::start());
                    let tags: Vec<String> = res.unwrap();
                    assert_eq!(vec!["xxx", "foo"], tags);

                    decrement_tags(1.into(), tags_vec!["xxx", "foo"], addr)
                        .unit_error()
                        .boxed()
                        .compat()
                })
                .and_then(move |res| {
                    res.expect("failed to decrement tags");

                    let addr = Data::new(crate::redis::start());
                    // let's check tags order again
                    read_redis_tags(1.into(), addr)
                        .unit_error()
                        .boxed()
                        .compat()
                })
                .and_then(|res| {
                    let tags: Vec<String> = res.unwrap();
                    assert_eq!(vec!["xxx"], tags);

                    System::current().stop();
                    future::result(Ok(()))
                });

            actix::spawn(fut);
        })
        .expect("failed to run system");
    }
}
