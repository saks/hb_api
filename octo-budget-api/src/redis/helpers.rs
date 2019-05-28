use actix_redis::{Command, Error as RedisError, RespValue};
// use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
use futures03::{compat::Future01CompatExt as _, FutureExt as _};
use redis_async::{
    resp::{FromResp, RespValue::Array},
    resp_array,
};

use super::Redis;
use crate::errors::Error;

pub async fn increment_tags(user_id: i32, tags: Vec<String>, redis: Redis) -> Result<(), Error> {
    let key = crate::config::user_tags_redis_key(user_id);

    let commands = tags
        .iter()
        .map(|tag| resp_array!["zincrby", &key, "1", tag])
        .collect::<Vec<_>>();

    execute_redis_commands(commands, redis).await
}

pub async fn increment_tags2(user_id: i32, tags: Vec<String>, redis: Redis) -> Result<(), Error> {
    let key = crate::config::user_tags_redis_key(user_id);

    let commands = tags
        .iter()
        .map(|tag| vec!["zincrby", &key, "1", &tag])
        .collect::<Vec<_>>();

    execute_redis_commands2(commands, redis).await
}
// pub async fn decrement_tags(user_id: i32, tags: Vec<String>, redis: Redis) -> Result<(), Error> {
//     let key = crate::config::user_tags_redis_key(user_id);
//
//     let mut commands: Vec<_> = tags
//         .iter()
//         .map(|tag| resp_array!["zincrby", &key, "-1", tag])
//         .collect();
//
//     commands.push(resp_array!["zremrangebyscore", &key, "0", "0"]);
//
//     execute_redis_commands(commands, redis).await
// }

pub async fn read_redis_tags(user_id: i32, redis: Redis) -> Result<Vec<String>, Error> {
    use crate::errors::Error::BadRedisResponse;

    let redis_key = crate::config::user_tags_redis_key(user_id);

    let command = Command(resp_array!["zrevrange", redis_key, "0", "-1"]);
    let response = Box::new(redis.send(command))
        .compat()
        .await?
        .map_err(Error::Redis)?;

    let tags = match response {
        // Here we assume that if returned value is of Array type, then query has succeeded.
        res @ Array(..) => Vec::from_resp(res).map_err(|e| BadRedisResponse(format!("{:?}", e))),
        res => Err(BadRedisResponse(format!("{:?}", res))),
    }?;

    Ok(tags)
}

async fn execute_redis_commands2(commands: Vec<Vec<&str>>, redis: Redis) -> Result<(), Error> {
    let responses = commands
        .into_iter()
        .map(|vec| RespValue::Array(vec.into_iter().map(|e| e.into()).collect::<Vec<_>>()))
        .map(|cmd| redis.send(Command(cmd)))
        .collect::<Vec<_>>();

    let responses = Box::new(futures::future::join_all(responses))
        .compat()
        .await?;

    let results = responses
        .into_iter()
        .collect::<Result<Vec<RespValue>, RedisError>>()
        .map_err(Error::Redis)?;

    results
        .into_iter()
        .map(|resp| match resp {
            e @ RespValue::Error(..) => Err(Error::RedisCommandFailed(e)),
            _ => Ok(()),
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(())
}

async fn execute_redis_commands(commands: Vec<RespValue>, redis: Redis) -> Result<(), Error> {
    let responses = commands
        .into_iter()
        .map(|cmd| redis.send(Command(cmd)))
        .collect::<Vec<_>>();

    let responses = Box::new(futures::future::join_all(responses))
        .compat()
        .await?;

    let results = responses
        .into_iter()
        .collect::<Result<Vec<RespValue>, RedisError>>()
        .map_err(Error::Redis)?;

    results
        .into_iter()
        .map(|resp| match resp {
            e @ RespValue::Error(..) => Err(Error::RedisCommandFailed(e)),
            _ => Ok(()),
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps2::helpers::sort_tags;
    use crate::tags_vec;
    use crate::tests::{self as tests, redis};
    use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
    // use futures03::FutureExt as _;

    #[test]
    fn my_test() {
        redis::flushall();

        let c = redis::get_connection();
        // let fut = async {
        //     let x = read_redis_tags(1, c).await;
        //
        //     Ok(())
        // };

        async fn xxx() -> Result<usize, String> {
            Ok(123)
        }

        let cmds = vec![vec!["info"]];
        let fut = execute_redis_commands2(cmds, c);

        // fut.then(|x| {
        //     dbg!(&x);
        //     Ok(())
        // });

        // let fut = xxx();

        // let x = actix_web::test::block_on(fut.boxed().compat());
        // dbg!(x);
    }

    // #[test]
    // fn sorted_tags_if_no_data_stores() {
    //     redis::flushall();
    //
    //     let c = redis::get_connection();
    //     let x = read_redis_tags(1, c);
    //
    //     // let z = tokio::run(Box::new(x).compat());
    //
    //     let future03 = async {
    //         dbg!();
    //         Ok(())
    //     };
    //
    //     use std::future::Future;
    //     async fn xxx() -> Result<usize, Error> {
    //         dbg!();
    //         Ok(123)
    //     }
    //
    //     let fut = xxx();
    //     let x = Box::pin(fut);
    //     let x1 = x.compat();
    //     let _ = tokio::run(x1);
    //     // let _ = tokio::run(Box::pin(xxx()).compat());
    //
    //     //
    //     // let z = actix::run(|| x.boxed().compat().map_err(|e| ()));
    //     // tests::run_future(
    //     //     Compat::new(read_redis_tags(1, redis::get_connection())),
    //     //     |result: Result<Vec<String>, Error>| {
    //     //         assert_eq!(Vec::<String>::new(), result.unwrap());
    //     //     },
    //     // );
    // }

    //     #[test]
    //     fn sorted_tags_if_data_exist() {
    //         redis::flushall();
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);
    //
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 assert_eq!(vec!["zzz", "xxx"], result.unwrap());
    //             },
    //         );
    //     }
    //
    //     #[test]
    //     #[should_panic(expected = "WRONGTYPE Operation against a key holding the wrong kind of value")]
    //     fn get_ordered_tags_with_redis_error() {
    //         redis::flushall();
    //         redis::exec_cmd(vec!["SET", "user_tags_1", "foo"]);
    //
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 result.unwrap();
    //             },
    //         );
    //     }
    //
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
    //
    //     #[test]
    //     fn decrement_tags_happy_path() {
    //         redis::flushall();
    //
    //         // prepare sort order for tags:
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "5", "xxx"]);
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "4", "foo"]);
    //         redis::exec_cmd(vec!["ZADD", "user_tags_1", "6", "zzz"]);
    //
    //         // check result BEFORE decrementing
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 assert_eq!(vec!["zzz", "xxx", "foo"], result.unwrap());
    //             },
    //         );
    //
    //         for _ in 0..3 {
    //             let fut = decrement_tags(1, crate::tags_vec!["zzz"], redis::get_connection());
    //             tests::run_future(Compat::new(fut), |res| assert!(res.is_ok()));
    //         }
    //
    //         // check result AFTER decrementing
    //         tests::run_future(
    //             Compat::new(read_redis_tags(1, redis::get_connection())),
    //             |result: Result<Vec<String>, Error>| {
    //                 assert_eq!(vec!["xxx", "foo", "zzz"], result.unwrap());
    //             },
    //         );
    //     }
    //
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
