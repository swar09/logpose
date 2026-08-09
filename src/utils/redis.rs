use redis::{
    AsyncCommands, AsyncConnectionConfig, Client, RedisError, SetOptions,
    aio::MultiplexedConnection,
};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct RedisStore {
    conn: MultiplexedConnection,
}

impl RedisStore {
    pub async fn new(redis_addr: &str) -> Result<Self, RedisError> {
        let config = AsyncConnectionConfig::new()
            .set_connection_timeout(Some(Duration::from_secs(3)))
            .set_response_timeout(Some(Duration::from_secs(2)));
        let client = Client::open(redis_addr)?;
        let conn = client
            .get_multiplexed_async_connection_with_config(&config)
            .await?;

        Ok(RedisStore { conn })
    }

    pub async fn blacklist(&self, jti: Uuid, exp_rsec: u64) -> Result<bool, RedisError> {
        let mut conn = self.conn.clone();
        let options = SetOptions::default().with_expiration(redis::SetExpiry::EX(exp_rsec));
        let key = format!("blacklist:{jti}");
        let value = format!("{jti}");
        let result: Option<String> = conn.set_options(key, value, options).await?;

        Ok(result.as_deref() == Some("OK"))
    }

    pub async fn is_blacklist(&self, jti: Uuid) -> Result<bool, RedisError> {
        let mut conn = self.conn.clone();

        let key = format!("blacklist:{jti}");
        let result: bool = conn.exists(key).await?;

        Ok(result)
    }

    pub async fn set_url(
        &self,
        exp: u64,
        long_url: String,
        short_code: String,
    ) -> Result<bool, RedisError> {
        let mut conn = self.conn.clone();

        let options = SetOptions::default().with_expiration(redis::SetExpiry::EX(exp));
        let key = format!("url:{short_code}");
        let value = long_url.to_string();
        let result: Option<String> = conn.set_options(key, value, options).await?;

        Ok(result.as_deref() == Some("OK"))
    }

    pub async fn get_url(&self, short_code: String) -> Result<Option<String>, RedisError> {
        let mut conn = self.conn.clone();

        let key = format!("url:{short_code}");
        let result: Option<String> = conn.get(key).await?;

        Ok(result)
    }

    pub async fn delete_url(&self, short_code: String) -> Result<bool, RedisError> {
        let mut conn = self.conn.clone();

        let key = format!("url:{short_code}");
        let result: u32 = conn.del(key).await?;

        Ok(result > 0)
    }
}
