// this file is abstraction over url reposiotory and redis caching okay

use base62::decode;
use diesel::r2d2::Pool;
use diesel::{PgConnection, r2d2::ConnectionManager};
use std::error::Error;

use crate::{repository::url::get_long_url_by_id, utils::redis::RedisStore};

pub struct UrlService {
    redis: RedisStore,
    pg_pool: Pool<ConnectionManager<PgConnection>>,
}

impl UrlService {
    pub fn new(
        redis: RedisStore,
        pg_pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Result<Self, axum::Error> {
        Ok(UrlService { redis, pg_pool })
    }

    pub async fn get_url(&self, short_code: String) -> Result<String, Box<dyn Error>> {
        if let Some(cached_long_url) = self.redis.get_url(short_code.clone()).await? {
            // cache hit
            return Ok(cached_long_url);
        }
        let mut pg_conn = self.pg_pool.get()?;
        // cache miss -> Database call
        let id = decode(&short_code)?;
        let long_url = get_long_url_by_id(id as i32, &mut pg_conn)?;
        let cache_long_url = long_url.clone();
        let redis = self.redis.clone();
        tokio::spawn(async move {
            let result = redis.set_url(120, cache_long_url, short_code).await;
            match result {
                Ok(i) => {
                    println!("cached long url got {i}");
                }
                Err(e) => {
                    eprintln!("{e}");
                }
            }
        });
        Ok(long_url)
    }
}
