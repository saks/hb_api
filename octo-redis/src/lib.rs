#![feature(async_await)]

// use futures03::{compat::Future01CompatExt as _, FutureExt as _, TryFutureExt as _};
// use futures03::compat::Future01CompatExt as _;

mod actor;
mod command;
mod errors;

pub type Addr = actix::Addr<RedisActor>;

pub use actor::RedisActor;
pub use command::cmd;
pub use errors::Error;

#[cfg(test)]
mod tests;
