// this file is abstraction over url repository and redis caching okay

use std::sync::Arc;

use aes::Aes256;
use diesel::r2d2::Pool;
use diesel::{PgConnection, r2d2::ConnectionManager};
use fpe::ff1::FF1;

use crate::{
    error::AppError,
    repository::url::get_long_url_by_id,
    utils::{base62::decode, redis::RedisStore},
};

#[derive(Clone)]
pub struct UrlService {
    redis: RedisStore,
    pg_pool: Pool<ConnectionManager<PgConnection>>,
    ff: Arc<FF1<Aes256>>,
}

impl UrlService {
    pub fn new(
        redis: RedisStore,
        pg_pool: Pool<ConnectionManager<PgConnection>>,
        ff: Arc<FF1<Aes256>>,
    ) -> Self {
        UrlService { redis, pg_pool, ff }
    }

    pub async fn get_url(&self, short_code: String) -> Result<String, AppError> {
        let cached_long_url = match self.redis.get_url(short_code.clone()).await {
            Ok(Some(url)) => Some(url),
            Ok(None) => None,
            Err(e) => {
                tracing::warn!("Redis lookup failed, falling back to database: {e}");
                None
            }
        };
        if let Some(url) = cached_long_url {
            // cache hit
            return Ok(url);
        }
        let mut pg_conn = self.pg_pool.get()?;
        // cache miss -> Database call
        let id = decode(&short_code, &self.ff)?;
        let long_url = get_long_url_by_id(id as i32, &mut pg_conn)?;
        let cache_long_url = long_url.clone();
        let redis = self.redis.clone();
        tokio::spawn(async move {
            let result = redis.set_url(120, cache_long_url, short_code).await;
            match result {
                Ok(i) => {
                    tracing::info!("cached long url got {i}");
                }
                Err(e) => {
                    tracing::error!("{e}");
                }
            }
        });
        Ok(long_url)
    }
}
