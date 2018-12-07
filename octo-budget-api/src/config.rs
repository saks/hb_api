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

config_env_var!(REDIS_URL);
config_env_var!(DATABASE_URL);
config_env_var!(AUTH_TOKEN_SECRET);
config_env_var!(LISTEN_IP);
config_env_var!(LISTEN_PORT);

lazy_static! {
    pub static ref DATABASE_POOL_SIZE: usize = env::var("DATABASE_POOL_SIZE")
        .expect("DATABASE_POOL_SIZE env var is not set")
        .parse()
        .expect("DATABASE_POOL_SIZE should be a number");
}
