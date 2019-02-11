use actix_redis::{Command, Error as RedisError, RespValue};
use actix_web_async_await::await;
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

    await!(execute_redis_commands(commands, redis))
}

pub async fn decrement_tags(user_id: i32, tags: Vec<String>, redis: Redis) -> Result<(), Error> {
    let key = crate::config::user_tags_redis_key(user_id);

    let mut commands: Vec<_> = tags
        .iter()
        .map(|tag| resp_array!["zincrby", &key, "-1", tag])
        .collect();

    commands.push(resp_array!["zremrangebyscore", &key, "0", "0"]);

    await!(execute_redis_commands(commands, redis))
}

pub async fn read_redis_tags(user_id: i32, redis: Redis) -> Result<Vec<String>, Error> {
    use crate::errors::Error::BadRedisResponse;

    let redis_key = crate::config::user_tags_redis_key(user_id);

    let command = Command(resp_array!["zrevrange", redis_key, "0", "-1"]);
    let response = await!(redis.send(command))?.map_err(Error::Redis)?;

    let tags = match response {
        // Here we assume that if returned value is of Array type, then query has succeeded.
        res @ Array(..) => Vec::from_resp(res).map_err(|e| BadRedisResponse(format!("{:?}", e))),
        res => Err(BadRedisResponse(format!("{:?}", res))),
    }?;

    Ok(tags)
}

async fn execute_redis_commands(commands: Vec<RespValue>, redis: Redis) -> Result<(), Error> {
    let responses = commands
        .into_iter()
        .map(|cmd| redis.send(Command(cmd)))
        .collect::<Vec<_>>();

    let responses = await!(futures::future::join_all(responses))?;

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
    use crate::apps::helpers::sort_tags;
    use crate::tags_vec;
    use crate::tests::{self as tests, redis};
    use tokio_async_await::compat::backward::Compat;

    #[test]
    fn sorted_tags_if_no_data_stores() {
        redis::flushall();

        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                assert_eq!(Vec::<String>::new(), result.unwrap());
            },
        );
    }

    #[test]
    fn sorted_tags_if_data_exist() {
        redis::flushall();
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);

        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                assert_eq!(vec!["zzz", "xxx"], result.unwrap());
            },
        );
    }

    #[test]
    #[should_panic(expected = "WRONGTYPE Operation against a key holding the wrong kind of value")]
    fn get_ordered_tags_with_redis_error() {
        redis::flushall();
        redis::exec_cmd(vec!["SET", "user_tags_1", "foo"]);

        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                result.unwrap();
            },
        );
    }

    #[test]
    fn sort_tags_with_redis_data() {
        redis::flushall();

        // prepare sort order for tags:
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "1", "foo"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);

        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                let redis_tags = result.unwrap();
                let user_tags = tags_vec!["foo", "xxx", "zzz"];
                let sorted = sort_tags(redis_tags, user_tags);

                assert_eq!(tags_vec!["zzz", "xxx", "foo"], sorted);
            },
        );
    }

    #[test]
    fn increment_tags_happy_path() {
        redis::flushall();

        // prepare sort order for tags:
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "1", "foo"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "3", "zzz"]);

        // check result BEFORE incrementing
        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                assert_eq!(vec!["zzz", "xxx", "foo"], result.unwrap());
            },
        );

        for _ in 0..3 {
            let fut = increment_tags(1, crate::tags_vec!["foo"], redis::get_connection());
            tests::run_future(Compat::new(fut), |res| assert!(res.is_ok()));
        }

        // check result AFTER incrementing
        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                assert_eq!(vec!["foo", "zzz", "xxx"], result.unwrap());
            },
        );
    }

    #[test]
    fn decrement_tags_happy_path() {
        redis::flushall();

        // prepare sort order for tags:
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "5", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "4", "foo"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "6", "zzz"]);

        // check result BEFORE decrementing
        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                assert_eq!(vec!["zzz", "xxx", "foo"], result.unwrap());
            },
        );

        for _ in 0..3 {
            let fut = decrement_tags(1, crate::tags_vec!["zzz"], redis::get_connection());
            tests::run_future(Compat::new(fut), |res| assert!(res.is_ok()));
        }

        // check result AFTER decrementing
        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                assert_eq!(vec!["xxx", "foo", "zzz"], result.unwrap());
            },
        );
    }

    #[test]
    fn decrement_tags_and_delete_zeros_happy_path() {
        redis::flushall();

        // prepare sort order for tags:
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "2", "xxx"]);
        redis::exec_cmd(vec!["ZADD", "user_tags_1", "1", "foo"]);

        // check result BEFORE decrementing
        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                assert_eq!(vec!["xxx", "foo"], result.unwrap());
            },
        );

        let fut = decrement_tags(1, crate::tags_vec!["xxx", "foo"], redis::get_connection());
        tests::run_future(Compat::new(fut), |res| assert!(res.is_ok()));

        // check result AFTER decrementing
        tests::run_future(
            Compat::new(read_redis_tags(1, redis::get_connection())),
            |result: Result<Vec<String>, Error>| {
                assert_eq!(vec!["xxx"], result.unwrap());
            },
        );
    }
}
