use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Thin Redis cache wrapper used by API handlers.
/// Falls back gracefully (cache miss) if Redis is unavailable.
#[derive(Clone)]
pub struct Cache {
    client: Option<redis::aio::ConnectionManager>,
}

impl Cache {
    /// Connect to Redis. Returns a no-op cache if `redis_url` is empty or connection fails.
    pub async fn new(redis_url: &str) -> Self {
        if redis_url.is_empty() {
            return Self { client: None };
        }
        match redis::Client::open(redis_url)
            .and_then(|c| Ok(c))
            .map_err(|e| e)
        {
            Ok(client) => match redis::aio::ConnectionManager::new(client).await {
                Ok(mgr) => {
                    tracing::info!("Redis cache connected: {}", redis_url);
                    Self { client: Some(mgr) }
                }
                Err(e) => {
                    tracing::warn!("Redis unavailable, caching disabled: {}", e);
                    Self { client: None }
                }
            },
            Err(e) => {
                tracing::warn!("Redis client error, caching disabled: {}", e);
                Self { client: None }
            }
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.client.clone()?;
        let raw: String = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .ok()?;
        serde_json::from_str(&raw).ok()
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) {
        let Some(mut conn) = self.client.clone() else {
            return;
        };
        let Ok(raw) = serde_json::to_string(value) else {
            return;
        };
        let _: Result<(), _> = redis::cmd("SET")
            .arg(key)
            .arg(raw)
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await;
    }

    pub async fn invalidate(&self, key: &str) {
        let Some(mut conn) = self.client.clone() else {
            return;
        };
        let _: Result<(), _> = redis::cmd("DEL").arg(key).query_async(&mut conn).await;
    }

    pub async fn invalidate_pattern(&self, pattern: &str) {
        let Some(mut conn) = self.client.clone() else {
            return;
        };
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();
        if !keys.is_empty() {
            let _: Result<(), _> = redis::cmd("DEL").arg(&keys).query_async(&mut conn).await;
        }
    }
}
