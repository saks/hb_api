use lazy_static::lazy_static;
use std::env;

macro_rules! config_env_var {
    ($var:ident) => {
        lazy_static! {
            #[derive(Copy, Clone, Debug)]
            pub static ref $var: String = env::var(stringify!($var))
                .expect(&format!("`{}' env var is not set", stringify!($var)));
        }
    };
}

const REDIS_KEY_USER_TAGS_PREFIX: &str = "user_tags_";

config_env_var!(REDIS_URL);
config_env_var!(DATABASE_URL);
config_env_var!(AUTH_TOKEN_SECRET);
config_env_var!(LISTEN_IP);
config_env_var!(LISTEN_PORT);

use std::fmt::Display;
pub fn user_tags_redis_key(user_id: impl Display) -> String {
    format!(
        "{prefix}{user_id}",
        prefix = REDIS_KEY_USER_TAGS_PREFIX,
        user_id = user_id
    )
}

pub fn redis_url() -> String {
    use url::Url;

    let redis_url = REDIS_URL.as_str();
    let url = Url::parse(REDIS_URL.as_str())
        .expect(format!("Cannot parse redis url: `{}'", redis_url).as_str());

    let host = url
        .host_str()
        .expect(format!("bad redis host: `{}'", redis_url).as_str());

    let port = url
        .port()
        .expect(format!("bad redis port: `{}'", redis_url).as_str());

    format!("{}:{}", host, port)
}

lazy_static! {
    pub static ref DATABASE_POOL_SIZE: usize = env::var("DATABASE_POOL_SIZE")
        .expect("DATABASE_POOL_SIZE env var is not set")
        .parse()
        .expect("DATABASE_POOL_SIZE should be a number");
}
