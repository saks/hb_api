use crate::config;
use actix_web::web::block;

pub type RedisConnection = redis::aio::MultiplexedConnection;

#[derive(Clone)]
pub struct Redis {
    client: redis::Client,
    connection: RedisConnection,
}

impl Redis {
    pub async fn new() -> Self {
        config::redis_url();
        let redis_url = config::REDIS_URL.to_string();
        let client =
            redis::Client::open(redis_url.as_str()).expect("Failed to create redis client");
        let (connection, driver) = client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to redis");

        actix_rt::spawn(driver);

        Self { client, connection }
    }

    pub async fn execute(&self, pipeline: redis::Pipeline) -> Result<(), crate::errors::Error> {
        let res = pipeline.query_async(&mut self.connection.clone()).await?;

        Ok(res)
    }
}

pub mod helpers;
