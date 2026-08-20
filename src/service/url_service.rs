use std::sync::Arc;

use aes::Aes256;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use fpe::ff1::FF1;

use crate::{
    error::AppError,
    repository::url::{get_long_url_by_id, get_long_url_by_short_code},
    utils::{
        alias::{get_cache_ttl_for_code, is_custom_alias},
        base62::decode,
        redis::RedisStore,
    },
};

#[derive(Clone)]
pub struct UrlService {
    redis: RedisStore,
    pg_pool: Pool<ConnectionManager<PgConnection>>,
    ff: Arc<FF1<Aes256>>,
}

impl UrlService {
    pub fn new(redis: RedisStore, pg_pool: Pool<ConnectionManager<PgConnection>>, ff: Arc<FF1<Aes256>>) -> Self {
        UrlService { redis, pg_pool, ff }
    }

    pub async fn get_url(&self, short_code: String) -> Result<String, AppError> {
        let cached_long_url = match self.redis.get_url(short_code.clone()).await {
            Ok(Some(url)) => Some(url),
            Ok(None) => None,
            Err(e) => {
                tracing::warn!("Redis lookup failed, falling back to database: {e}");
                None
            },
        };
        if let Some(url) = cached_long_url {
            return Ok(url);
        }

        let mut pg_conn = self.pg_pool.get()?;
        let ttl_seconds = get_cache_ttl_for_code(&short_code);

        let long_url = if is_custom_alias(&short_code) {
            get_long_url_by_short_code(short_code.clone(), &mut pg_conn)?
        } else {
            let id = decode(&short_code, &self.ff)?;
            get_long_url_by_id(id as i32, &mut pg_conn)?
        };

        let cache_long_url = long_url.clone();
        let redis = self.redis.clone();
        let sc = short_code.clone();
        tokio::spawn(async move {
            let _ = redis.set_url(ttl_seconds, cache_long_url, sc).await;
        });

        Ok(long_url)
    }
}
