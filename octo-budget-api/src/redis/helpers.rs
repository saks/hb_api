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

    let pipeline_res = pipeline
        .send::<Vec<bool>>(redis.get_ref().to_owned())
        .await?;
    dbg!(pipeline_res);

    Ok(())
}

pub async fn decrement_tags(user_id: UserId, tags: Vec<String>, redis: Redis) -> Result<(), Error> {
    let key = user_tags_redis_key(user_id);

    let mut pipeline = Pipeline::new();

    for tag in &tags {
        pipeline.add_command(&cmd("zincrby").arg(&key).arg("-1").arg(tag));
    }

    pipeline.add_command(&cmd("zremrangebyscore").arg(&key).arg("0").arg("0"));

    let pipeline_res = pipeline
        .send::<Vec<bool>>(redis.get_ref().to_owned())
        .await?;
    dbg!(pipeline_res);

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
    use crate::apps2::helpers::sort_tags;
    use actix::prelude::*;
    use actix_service::ServiceExt;
    use actix_web::web::Data;
    use futures::future;
    use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
    //    use futures_util::try_stream::TryStreamExt;
    use redis::{self, Commands as _};

    mod test_redis {
        use serde::export::fmt::Display;

        pub struct Session(redis::Connection);

        impl Session {
            pub fn new() -> Self {
                let url = crate::config::redis_url();
                let client = redis::Client::open(url.as_str()).expect("failed to create client");
                let conn = client.get_connection().expect("failed to connect");

                redis::cmd("flushall").execute(&conn);

                Self(conn)
            }

            pub fn zadd<T: redis::ToRedisArgs + Display>(&self, user_id: T, score: T, tag: T) {
                let key = format!("user_tags_{}", user_id);
                redis::cmd("zadd")
                    .arg(key)
                    .arg(score)
                    .arg(tag)
                    .execute(&self.0);
            }

            pub fn conn(&self) -> &redis::Connection {
                &self.0
            }
        }

        impl Drop for Session {
            fn drop(&mut self) {
                redis::cmd("flushall").execute(&self.0);
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
        let session = test_redis::Session::new();

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
        let session = test_redis::Session::new();

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
                let tags: Vec<String> = res.unwrap().unwrap();
                //                assert_eq!(vec!["zzz", "xxx"], tags);

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

    //    #[test]
    //    fn decrement_tags_happy_path() {
    //        use futures::future::Future;
    //
    //        let session = test_redis::Session::new();
    //        let user_id = "1";
    //
    //        session.zadd(user_id, "5", "xxx");
    //        session.zadd(user_id, "4", "foo");
    //        session.zadd(user_id, "6", "zzz");
    //
    //        System::run(|| {
    //            let addr = Data::new(crate::redis::start());
    //            let fut = read_redis_tags(1.into(), addr)
    //                .unit_error()
    //                .boxed()
    //                .compat();
    //
    //            let fut = fut.and_then(|res| {
    //                increment_tags(1.into(), tags_vec!["zzz"], addr)
    //                    .unit_err()
    //                    .boxed()
    //                    .compat()
    //            });
    //
    //            //            let fut = fut.map_err(|e| dbg!(e)).map(move |res| {
    //            //                let tags: Vec<String> = res.unwrap().unwrap();
    //            //                dbg!(tags);
    //            ////                assert_eq!(vec!["zzz", "xxx", "foo"], tags);
    //            //            }).and_then(move |res| {
    //            //                increment_tags(1.into(), tags_vec!["zzz"], addr)
    //            //
    //            //                System::current().stop();
    //            //                future::result(Ok(()))
    //            //            });
    //
    //            //            use actix::prelude::ContextFutureSpawner;
    //
    //            // check result BEFORE decrementing
    //            actix::spawn(fut);
    //        })
    //        .expect("failed to run system");
    //
    //        //             tests::run_future(
    //        //                 Compat::new(read_redis_tags(1, redis::get_connection())),
    //        //                 |result: Result<Vec<String>, Error>| {
    //        //                     assert_eq!(vec!["zzz", "xxx", "foo"], result.unwrap());
    //        //                 },
    //        //             );
    //        //
    //        //             for _ in 0..3 {
    //        //                 let fut = decrement_tags(1, crate::tags_vec!["zzz"], redis::get_connection());
    //        //                 tests::run_future(Compat::new(fut), |res| assert!(res.is_ok()));
    //        //             }
    //        //
    //        //             // check result AFTER decrementing
    //        //             tests::run_future(
    //        //                 Compat::new(read_redis_tags(1, redis::get_connection())),
    //        //                 |result: Result<Vec<String>, Error>| {
    //        //                     assert_eq!(vec!["xxx", "foo", "zzz"], result.unwrap());
    //        //                 },
    //        //             );
    //    }

    //     #[test]
    //     fn decrement_tags_and_delete_zeros_happy_path() {
    //         redis::flushall();
    //
    //         // prepare sort order for tags:
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "1", "foo"]);
    //
    //         // check result BEFORE decrementing
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 assert_eq!(vec!["xxx", "foo"], result.unwrap());
    //             },
    //         );
    //
    //         let fut = decrement_tags(1, crate::tags_vec!["xxx", "foo"], redis::get_connection());
    //         tests::run_future(Compat::new(fut), |res| assert!(res.is_ok()));
    //
    //         // check result AFTER decrementing
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 assert_eq!(vec!["xxx"], result.unwrap());
    //             },
    //         );
    //     }
}
