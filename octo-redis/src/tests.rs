use super::*;
use actix::prelude::*;
use futures03::future::{FutureExt as _, TryFutureExt as _};
use redis::Commands as _;

fn setup() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1/").expect("Failed build client");
    client.get_connection().expect("Failed to connect")
}

#[test]
fn get_text_value() {
    let conn = setup();
    let expected_value = "ZZZ!!!";
    let _: () = conn
        .set("foo", expected_value)
        .expect("cannot set expected value");

    let sys = System::new("test");
    let addr = RedisActor::start("redis://127.0.0.1/");
    let msg = crate::cmd("GET").arg("foo").send::<String>(addr);

    Arbiter::spawn(
        msg.unit_error()
            .boxed()
            .compat()
            .map(move |res| {
                assert_eq!(expected_value, res.expect("redis error"));
                System::current().stop();
            })
            .map_err(|e| {
                panic!("Error: {:?}", e);
            }),
    );

    sys.run().expect("failed to run system");
}
