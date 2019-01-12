use actix::{Arbiter, System};
use actix_redis::{Command, RedisActor};
use futures::Future;
use redis_async::{
    resp::{RespValue, RespValue::SimpleString},
    resp_array,
};

pub fn handle_message<F: 'static>(msg: Command, callback: F)
where
    F: Fn(RespValue),
{
    let system = System::new("test");

    let addr = RedisActor::start(crate::config::redis_url());
    let result = addr.send(msg);

    Arbiter::spawn(
        result
            .map(move |result| {
                callback(result.expect("unexpected redis response"));
                System::current().stop();
            })
            .map_err(|_| ()),
    );

    system.run();
}

// use actix::{Arbiter, System};
// use actix_redis::RedisActor;
// use futures::Future;
// use std::sync::Arc;
// use tokio_async_await::compat::backward::Compat;
pub fn run_future<F: 'static, Fut: 'static>(fut: Fut, callback: F)
where
    Fut: Future,
    F: Fn(Result<Fut::Item, Fut::Error>),
{
    let system = System::new("test");

    Arbiter::spawn({
        fut.then(move |res| {
            callback(res);
            System::current().stop();
            futures::future::ok(())
        })
    });

    system.run();
}

// use failure::Fallible;
// pub fn handle_message2<F: 'static>(fut: Future<Item = Fallible<Vec<String>>>, callback: F)
// where
//     F: Fn(RespValue),
// {
//     let system = System::new("test");
//
//     let addr = RedisActor::start(crate::config::redis_url());
//     let result = addr.send(msg);
//
//     Arbiter::spawn(
//         result
//             .map(move |result| {
//                 callback(result.expect("unexpected redis response"));
//                 System::current().stop();
//             })
//             .map_err(|_| ()),
//     );
//
//     system.run();
// }

pub fn exec_cmd(cmd: Vec<&str>) {
    let msg = Command(RespValue::Array(
        cmd.into_iter().map(|e| e.into()).collect(),
    ));
    handle_message(msg, |_| {});
}

pub fn get_connection() -> crate::apps::Redis {
    use std::sync::Arc;
    Arc::new(RedisActor::start(crate::config::redis_url()))
}

pub fn flushall() {
    handle_message(Command(resp_array!["flushall"]), |result| {
        assert_eq!(
            SimpleString("OK".to_string()),
            result,
            "not OK response from redis"
        );
    });
}
