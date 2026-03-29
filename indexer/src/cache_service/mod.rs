use crate::cache::Cache;
use crate::config::CacheConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

// ---------------------------------------------------------------------------
// Cache key constants
// ---------------------------------------------------------------------------

pub const KEY_EVENTS_LIST: &str = "events:list";
pub const KEY_STATS: &str = "stats:platform";
pub const KEY_TRADE_PREFIX: &str = "trade:";
pub const KEY_COMPLIANCE_PREFIX: &str = "compliance:";
pub const KEY_ANALYTICS_DASHBOARD: &str = "analytics:dashboard";

// ---------------------------------------------------------------------------
// Monitoring
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
    pub hit_rate: f64,
    pub connected: bool,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CacheService {
    pub cache: Cache,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl CacheService {
    pub async fn new(config: CacheConfig) -> Self {
        let cache = Cache::new(&config.redis_url).await;
        let connected = !config.redis_url.is_empty();
        Self {
            cache,
            config,
            stats: Arc::new(RwLock::new(CacheStats {
                connected,
                ..Default::default()
            })),
        }
    }

    // ── Generic get/set with hit tracking ────────────────────────────────────

    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let result = self.cache.get(key).await;
        let mut s = self.stats.write().await;
        if result.is_some() {
            s.hits += 1;
        } else {
            s.misses += 1;
        }
        let total = s.hits + s.misses;
        s.hit_rate = if total > 0 { s.hits as f64 / total as f64 } else { 0.0 };
        result
    }

    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T, ttl: Duration) {
        self.cache.set(key, value, ttl).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
        self.stats.write().await.invalidations += 1;
    }

    pub async fn invalidate_pattern(&self, pattern: &str) {
        self.cache.invalidate_pattern(pattern).await;
        self.stats.write().await.invalidations += 1;
    }

    // ── Domain-specific helpers ───────────────────────────────────────────────

    pub async fn get_events_list<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        self.get(KEY_EVENTS_LIST).await
    }

    pub async fn set_events_list<T: serde::Serialize>(&self, value: &T) {
        let ttl = Duration::from_secs(self.config.events_ttl_secs);
        self.set(KEY_EVENTS_LIST, value, ttl).await;
    }

    pub async fn get_stats<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        self.get(KEY_STATS).await
    }

    pub async fn set_stats<T: serde::Serialize>(&self, value: &T) {
        let ttl = Duration::from_secs(self.config.default_ttl_secs);
        self.set(KEY_STATS, value, ttl).await;
    }

    pub async fn invalidate_trade(&self, trade_id: u64) {
        let key = format!("{}{}", KEY_TRADE_PREFIX, trade_id);
        self.invalidate(&key).await;
        // Also bust the events list since it may contain this trade
        self.invalidate(KEY_EVENTS_LIST).await;
    }

    pub async fn invalidate_compliance(&self, address: &str) {
        let key = format!("{}{}", KEY_COMPLIANCE_PREFIX, address);
        self.invalidate(&key).await;
    }

    pub async fn get_search<T: serde::de::DeserializeOwned>(&self, cache_key: &str) -> Option<T> {
        self.get(cache_key).await
    }

    pub async fn set_search<T: serde::Serialize>(&self, cache_key: &str, value: &T) {
        let ttl = Duration::from_secs(self.config.search_ttl_secs);
        self.set(cache_key, value, ttl).await;
    }

    pub async fn get_analytics<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        self.get(KEY_ANALYTICS_DASHBOARD).await
    }

    pub async fn set_analytics<T: serde::Serialize>(&self, value: &T) {
        let ttl = Duration::from_secs(self.config.analytics_ttl_secs);
        self.set(KEY_ANALYTICS_DASHBOARD, value, ttl).await;
    }

    /// Warm the cache by pre-loading frequently accessed keys.
    /// Called at startup and periodically.
    pub async fn warm<F, Fut>(&self, loader: F)
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        tracing::info!("Cache warming started");
        loader().await;
        tracing::info!("Cache warming complete");
    }

    pub async fn get_stats_snapshot(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}
