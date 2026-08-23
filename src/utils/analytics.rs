use std::{net::SocketAddr, time::Duration};

use axum::http::{HeaderMap, header};
use chrono::Utc;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use tracing::{error, warn};
use woothee::parser::Parser;

use crate::{
    models::url_analytics::{NewEntry, RawAnalyticsEvent},
    repository::url_analytics::create_batch,
    utils::redis::RedisStore,
};

pub const DEFAULT_ANALYTICS_BATCH_SIZE: usize = 500;
pub const DEFAULT_ANALYTICS_FLUSH_INTERVAL: Duration = Duration::from_millis(500);

pub fn extract_raw_analytics(addr: SocketAddr, headers: &HeaderMap, short_code: String) -> RawAnalyticsEvent {
    let get_header = |name: &str| -> Option<String> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|xff| {
            xff.split(',')
                .next_back()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| addr.ip().to_string());

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    let referer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty());

    let country_code = get_header("x-country-code");

    RawAnalyticsEvent {
        short_code,
        ip_address,
        user_agent,
        referer,
        country_code,
        clicked_at: Utc::now(),
    }
}

pub async fn run_analytics_worker(
    redis: RedisStore,
    pool: Pool<ConnectionManager<PgConnection>>,
    batch_size: usize,
    flush_interval: Duration,
) {
    let parser = Parser::new();

    loop {
        let raw_items = match redis.pop_analytics_batch(batch_size).await {
            Ok(items) => items,
            Err(e) => {
                error!("Error popping analytics batch from Redis: {e}");
                tokio::time::sleep(flush_interval).await;
                continue;
            },
        };

        let count = raw_items.len();
        if raw_items.is_empty() {
            tokio::time::sleep(flush_interval).await;
            continue;
        }

        let mut entries: Vec<NewEntry> = Vec::with_capacity(count);
        for item in raw_items {
            match serde_json::from_str::<RawAnalyticsEvent>(&item) {
                Ok(event) => {
                    let (browser, device) = if let Some(ref ua) = event.user_agent {
                        match parser.parse(ua) {
                            Some(res) => (
                                if res.name.is_empty() {
                                    None
                                } else {
                                    Some(res.name.to_string())
                                },
                                if res.category.is_empty() {
                                    None
                                } else {
                                    Some(res.category.to_string())
                                },
                            ),
                            None => (None, None),
                        }
                    } else {
                        (None, None)
                    };

                    entries.push(NewEntry {
                        device,
                        browser,
                        referer: event.referer,
                        short_code: Some(event.short_code),
                        clicked_at: event.clicked_at,
                        ip_address: event.ip_address,
                        user_agent: event.user_agent,
                        country_code: event.country_code,
                    });
                },
                Err(e) => {
                    warn!("Failed to parse raw analytics event JSON: {e}");
                },
            }
        }

        if !entries.is_empty() {
            let pool_clone = pool.clone();
            let insert_res = tokio::task::spawn_blocking(move || {
                let mut conn = pool_clone.get().map_err(|e| e.to_string())?;
                create_batch(&entries, &mut conn).map_err(|e| e.to_string())
            })
            .await;

            match insert_res {
                Ok(Ok(num_inserted)) => {
                    tracing::debug!("Successfully inserted {num_inserted} analytics events");
                },
                Ok(Err(e)) => {
                    error!("Database error inserting analytics batch: {e}");
                },
                Err(e) => {
                    error!("Join error in analytics batch insert task: {e}");
                },
            }
        }

        if count < batch_size {
            tokio::time::sleep(flush_interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::*;

    #[test]
    fn test_extract_raw_analytics_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64)".parse().unwrap(),
        );
        headers.insert(header::REFERER, "https://google.com".parse().unwrap());
        headers.insert("x-forwarded-for", "203.0.113.195, 70.41.3.18".parse().unwrap());
        headers.insert("x-country-code", "US".parse().unwrap());

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let event = extract_raw_analytics(addr, &headers, "xyz123".to_string());

        assert_eq!(event.short_code, "xyz123");
        assert_eq!(event.ip_address, "70.41.3.18");
        assert_eq!(
            event.user_agent.as_deref(),
            Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        );
        assert_eq!(event.referer.as_deref(), Some("https://google.com"));
        assert_eq!(event.country_code.as_deref(), Some("US"));
    }

    #[test]
    fn test_raw_analytics_serialization_roundtrip() {
        let event = RawAnalyticsEvent {
            short_code: "abc456".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: Some("curl/7.68.0".to_string()),
            referer: None,
            country_code: Some("IN".to_string()),
            clicked_at: Utc::now(),
        };

        let json = serde_json::to_string(&event).expect("Serialization failed");
        let parsed: RawAnalyticsEvent = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(parsed.short_code, event.short_code);
        assert_eq!(parsed.ip_address, event.ip_address);
        assert_eq!(parsed.user_agent, event.user_agent);
        assert_eq!(parsed.referer, event.referer);
        assert_eq!(parsed.country_code, event.country_code);
    }
}
