use lazy_static::lazy_static;
use std::env::{self, var};
use std::fmt::Display;

const REDIS_URL_ENV_VAR: &str = "REDIS_URL";
const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
const REDIS_PORT: &str = "6379";
const PG_PORT: &str = "5432";
const PG_DEFAULT_PASSWORD: &str = "mysecretpassword";
const PG_DEFAULT_USER: &str = "rustapp";
const PG_DEFAULT_DB: &str = "test";
const REDIS_KEY_USER_TAGS_PREFIX: &str = "user_tags_";
const FORCE_HTTPS_VAR_NAME: &str = "FORCE_HTTPS";

lazy_static! {
    pub static ref REDIS_URL: String = get_redis_url();
    pub static ref DATABASE_URL: String = get_database_url();
    pub static ref DATABASE_POOL_SIZE: usize = env::var("DATABASE_POOL_SIZE")
        .expect("DATABASE_POOL_SIZE env var is not set")
        .parse()
        .expect("DATABASE_POOL_SIZE should be a number");
}

mod helpers {
    use std::process::{Command, Stdio};

    pub(super) fn docker_compose_service_port(name: &str, port: &str) -> String {
        let output = Command::new("docker-compose")
            .arg("port")
            .arg(name)
            .arg(port)
            .output()
            .expect("failed to run docker-compose to get service port");

        String::from_utf8(output.stdout).expect("failed to parse docker-compose output")
    }

    pub(super) fn docker_compose_start(name: &str) {
        Command::new("docker-compose")
            .arg("up")
            .arg("-d")
            .arg(name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("failed to start docker-compose service");
    }
}

fn get_database_url() -> String {
    var(DATABASE_URL_ENV_VAR).unwrap_or_else(|_| {
        helpers::docker_compose_start("db");

        let host = helpers::docker_compose_service_port("db", PG_PORT);
        format!(
            "postgres://{user}:{password}@{host}/{db}",
            user = PG_DEFAULT_USER,
            password = PG_DEFAULT_PASSWORD,
            host = host.trim(),
            db = PG_DEFAULT_DB
        )
    })
}

fn get_redis_url() -> String {
    var(REDIS_URL_ENV_VAR).unwrap_or_else(|_| {
        helpers::docker_compose_start("redis");

        let redis_host = helpers::docker_compose_service_port("redis", REDIS_PORT);
        format!("redis://{}", redis_host.trim())
    })
}

macro_rules! config_env_var {
    ($var:ident) => {
        lazy_static! {
            #[derive(Copy, Clone, Debug)]
            pub static ref $var: String = env::var(stringify!($var))
                .expect(&format!("`{}' env var is not set", stringify!($var)));
        }
    };
}

config_env_var!(AUTH_TOKEN_SECRET);
config_env_var!(LISTEN_IP);
config_env_var!(PORT);

pub fn user_tags_redis_key(user_id: impl Display) -> String {
    format!(
        "{prefix}{user_id}",
        prefix = REDIS_KEY_USER_TAGS_PREFIX,
        user_id = user_id
    )
}

pub fn is_force_https() -> bool {
    std::env::var(FORCE_HTTPS_VAR_NAME).is_ok()
}

pub fn redis_url() -> String {
    REDIS_URL.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_redis_url_from_env_var() {
        let url = "redis://redis:password@127.0.0.1:6379";
        env::set_var(REDIS_URL_ENV_VAR, url);

        assert_eq!(get_redis_url(), url);

        env::remove_var(REDIS_URL_ENV_VAR);
    }

    #[test]
    fn get_redis_url_from_docker_compose() {
        if env::var("CI").is_ok() {
            // don't run this test on CI
            return;
        }

        env::remove_var(REDIS_URL_ENV_VAR);

        assert!(get_redis_url().starts_with("redis://0.0.0.0:"));
    }

    #[test]
    fn get_database_url_from_env_var() {
        let url = "postgres://user:pass@127.0.0.1:5432/test_db";
        env::set_var(DATABASE_URL_ENV_VAR, url);

        assert_eq!(get_database_url(), url);

        env::remove_var(DATABASE_URL_ENV_VAR);
    }

    #[test]
    fn get_database_url_from_docker_compose() {
        if env::var("CI").is_ok() {
            // don't run this test on CI
            return;
        }

        env::remove_var(DATABASE_URL_ENV_VAR);

        assert!(get_database_url().starts_with("postgres://rustapp:mysecretpassword@0.0.0.0:"));
    }
}
