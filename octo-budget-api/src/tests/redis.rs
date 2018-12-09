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
    let addr = RedisActor::start(crate::config::redis_url());
    let result = addr.send(msg);

    System::run(move || {
        Arbiter::spawn(
            result
                .map(move |result| {
                    callback(result.unwrap());
                    System::current().stop();
                })
                .map_err(|_| ()),
        );
    });
}

pub fn exec_cmd(cmd: Vec<&str>) {
    let msg = Command(RespValue::Array(
        cmd.into_iter().map(|e| e.into()).collect(),
    ));
    handle_message(msg, |_| {});
}

pub fn flushall() {
    handle_message(Command(resp_array!["flushall"]), |result| {
        assert_eq!(SimpleString("OK".to_string()), result);
    });
}
