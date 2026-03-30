use crate::cache::Cache;
use crate::config::CacheConfig;
use crate::database::Database;
use crate::error::AppError;
use crate::models::{EventQuery, EventStats, IndexerStatus, StatsResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub const KEY_EVENTS_LIST_PREFIX: &str = "events:list:";
pub const KEY_STATS: &str = "stats:platform";
pub const KEY_INDEXER_STATUS: &str = "status:indexer";
pub const KEY_SEARCH_TRADES_PREFIX: &str = "search:trades:";
pub const KEY_COMPLIANCE_PREFIX: &str = "compliance:";
pub const KEY_TRADE_PREFIX: &str = "trade:";

// Soroban RPC read-only call cache keys
/// Prefix for `get_funding_preview` results: `rpc:funding_preview:<trade_id>:<buyer>`
pub const KEY_RPC_FUNDING_PREVIEW_PREFIX: &str = "rpc:funding_preview:";
/// Prefix for `get_trade_detail` results: `rpc:trade_detail:<trade_id>`
pub const KEY_RPC_TRADE_DETAIL_PREFIX: &str = "rpc:trade_detail:";
/// Prefix for `get_platform_stats` results: `rpc:platform_stats`
pub const KEY_RPC_PLATFORM_STATS: &str = "rpc:platform_stats";
/// Prefix for `get_fee_info` results: `rpc:fee_info`
pub const KEY_RPC_FEE_INFO: &str = "rpc:fee_info";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheStrategy {
    ReadThrough,
    WriteAround,
    Warmed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub writes: u64,
    pub invalidations: u64,
    pub warm_runs: u64,
    pub warm_failures: u64,
    pub hit_rate: f64,
    pub connected: bool,
    pub configured: bool,
    pub tracked_keys: usize,
    pub last_warm_started_at: Option<DateTime<Utc>>,
    pub last_warm_completed_at: Option<DateTime<Utc>>,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            writes: 0,
            invalidations: 0,
            warm_runs: 0,
            warm_failures: 0,
            hit_rate: 0.0,
            connected: false,
            configured: false,
            tracked_keys: 0,
            last_warm_started_at: None,
            last_warm_completed_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheWarmReport {
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSnapshot {
    pub stats: CacheStats,
    pub keys: Vec<String>,
    pub strategies: Vec<CacheStrategyRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStrategyRule {
    pub key_pattern: String,
    pub strategy: CacheStrategy,
    pub ttl_secs: u64,
    pub description: String,
}

pub struct CacheService {
    pub cache: Cache,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    tracked_keys: Arc<RwLock<Vec<String>>>,
}

impl CacheService {
    pub async fn new(config: CacheConfig) -> Self {
        let cache = Cache::new(&config.redis_url).await;
        let connected = cache.is_connected();
        let configured = !config.redis_url.trim().is_empty();

        Self {
            cache,
            config,
            stats: Arc::new(RwLock::new(CacheStats {
                connected,
                configured,
                ..Default::default()
            })),
            tracked_keys: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn strategies(&self) -> Vec<CacheStrategyRule> {
        vec![
            CacheStrategyRule {
                key_pattern: format!("{}*", KEY_EVENTS_LIST_PREFIX),
                strategy: CacheStrategy::ReadThrough,
                ttl_secs: self.config.events_ttl_secs,
                description: "Paginated event list responses".to_string(),
            },
            CacheStrategyRule {
                key_pattern: KEY_STATS.to_string(),
                strategy: CacheStrategy::Warmed,
                ttl_secs: self.config.default_ttl_secs,
                description: "Platform stats dashboard summary".to_string(),
            },
            CacheStrategyRule {
                key_pattern: KEY_INDEXER_STATUS.to_string(),
                strategy: CacheStrategy::Warmed,
                ttl_secs: self.config.default_ttl_secs,
                description: "Indexer sync status".to_string(),
            },
            CacheStrategyRule {
                key_pattern: format!("{}*", KEY_SEARCH_TRADES_PREFIX),
                strategy: CacheStrategy::ReadThrough,
                ttl_secs: self.config.default_ttl_secs,
                description: "Trade search responses".to_string(),
            },
            CacheStrategyRule {
                key_pattern: format!("{}*", KEY_COMPLIANCE_PREFIX),
                strategy: CacheStrategy::ReadThrough,
                ttl_secs: self.config.default_ttl_secs,
                description: "Latest compliance check by address".to_string(),
            },
            CacheStrategyRule {
                key_pattern: format!("{}*", KEY_TRADE_PREFIX),
                strategy: CacheStrategy::WriteAround,
                ttl_secs: self.config.default_ttl_secs,
                description: "Trade-specific entries invalidated on lifecycle changes".to_string(),
            },
            // Soroban RPC read-only call caching
            CacheStrategyRule {
                key_pattern: format!("{}*", KEY_RPC_FUNDING_PREVIEW_PREFIX),
                strategy: CacheStrategy::ReadThrough,
                ttl_secs: self.config.default_ttl_secs,
                description: "get_funding_preview RPC results; invalidated on trade lifecycle events".to_string(),
            },
            CacheStrategyRule {
                key_pattern: format!("{}*", KEY_RPC_TRADE_DETAIL_PREFIX),
                strategy: CacheStrategy::ReadThrough,
                ttl_secs: self.config.default_ttl_secs,
                description: "get_trade_detail RPC results; invalidated on trade lifecycle events".to_string(),
            },
            CacheStrategyRule {
                key_pattern: KEY_RPC_PLATFORM_STATS.to_string(),
                strategy: CacheStrategy::ReadThrough,
                ttl_secs: self.config.default_ttl_secs,
                description: "get_platform_stats RPC results; invalidated on fee/trade events".to_string(),
            },
            CacheStrategyRule {
                key_pattern: KEY_RPC_FEE_INFO.to_string(),
                strategy: CacheStrategy::ReadThrough,
                ttl_secs: self.config.default_ttl_secs,
                description: "get_fee_info RPC results; invalidated on fee_updated events".to_string(),
            },
        ]
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let result = self.cache.get(key).await;
        let mut stats = self.stats.write().await;
        if result.is_some() {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
        update_hit_rate(&mut stats);
        result
    }

    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T, ttl: Duration) {
        self.cache.set(key, value, ttl).await;
        self.track_key(key).await;
        let mut stats = self.stats.write().await;
        stats.writes += 1;
        stats.tracked_keys = self.tracked_keys.read().await.len();
    }

    pub async fn get_or_load<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        loader: F,
    ) -> Result<T, AppError>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, AppError>>,
    {
        if let Some(cached) = self.get(key).await {
            return Ok(cached);
        }

        let value = loader().await?;
        self.set(key, &value, ttl).await;
        Ok(value)
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
        self.untrack_key(key).await;
        let mut stats = self.stats.write().await;
        stats.invalidations += 1;
        stats.tracked_keys = self.tracked_keys.read().await.len();
    }

    pub async fn invalidate_pattern(&self, pattern: &str) {
        let tracked = self.tracked_keys.read().await.clone();
        self.cache.invalidate_pattern(pattern).await;

        let mut keys = self.tracked_keys.write().await;
        if !tracked.is_empty() {
            keys.retain(|key| !pattern_matches(pattern, key));
        }

        let mut stats = self.stats.write().await;
        stats.invalidations += 1;
        stats.tracked_keys = keys.len();
    }

    pub async fn invalidate_trade(&self, trade_id: u64) {
        let key = format!("{}{}", KEY_TRADE_PREFIX, trade_id);
        self.invalidate(&key).await;
        self.invalidate_pattern(&format!("{}*", KEY_EVENTS_LIST_PREFIX)).await;
        self.invalidate(KEY_STATS).await;
        self.invalidate(KEY_INDEXER_STATUS).await;
        self.invalidate_pattern(&format!("{}*", KEY_SEARCH_TRADES_PREFIX)).await;
    }

    pub async fn invalidate_compliance(&self, address: &str) {
        self.invalidate(&Self::compliance_status_key(address)).await;
    }

    pub async fn warm_core(&self, database: &Arc<Database>, contract_id: &str) -> Result<CacheWarmReport, AppError> {
        let started_at = Utc::now();
        {
            let mut stats = self.stats.write().await;
            stats.warm_runs += 1;
            stats.last_warm_started_at = Some(started_at);
        }

        let result = async {
            let status = Self::build_indexer_status(database, contract_id).await?;
            let stats_response = Self::build_stats_response(database).await?;
            let recent_events = database
                .get_events(&EventQuery {
                    limit: Some(20),
                    offset: Some(0),
                    event_type: None,
                    category: None,
                    trade_id: None,
                    from_ledger: None,
                    to_ledger: None,
                    from_time: None,
                    to_time: None,
                    contract_id: None,
                })
                .await?;

            let ttl = Duration::from_secs(self.config.default_ttl_secs);
            let events_ttl = Duration::from_secs(self.config.events_ttl_secs);

            self.set(KEY_INDEXER_STATUS, &status, ttl).await;
            self.set(KEY_STATS, &stats_response, ttl).await;
            self.set(&Self::events_query_key(&EventQuery {
                limit: Some(20),
                offset: Some(0),
                event_type: None,
                category: None,
                trade_id: None,
                from_ledger: None,
                to_ledger: None,
                from_time: None,
                to_time: None,
                contract_id: None,
            }), &recent_events, events_ttl).await;

            Ok::<_, AppError>(vec![
                KEY_INDEXER_STATUS.to_string(),
                KEY_STATS.to_string(),
                Self::events_query_key(&EventQuery {
                    limit: Some(20),
                    offset: Some(0),
                    event_type: None,
                    category: None,
                    trade_id: None,
                    from_ledger: None,
                    to_ledger: None,
                    from_time: None,
                    to_time: None,
                    contract_id: None,
                }),
            ])
        }
        .await;

        match result {
            Ok(keys) => {
                let completed_at = Utc::now();
                let mut stats = self.stats.write().await;
                stats.last_warm_completed_at = Some(completed_at);
                stats.tracked_keys = self.tracked_keys.read().await.len();
                Ok(CacheWarmReport {
                    started_at,
                    completed_at,
                    keys,
                })
            }
            Err(err) => {
                let mut stats = self.stats.write().await;
                stats.warm_failures += 1;
                Err(err)
            }
        }
    }

    pub async fn get_stats_snapshot(&self) -> CacheStats {
        let mut snapshot = self.stats.read().await.clone();
        snapshot.connected = self.cache.is_connected();
        snapshot.tracked_keys = self.tracked_keys.read().await.len();
        snapshot
    }

    pub async fn snapshot(&self) -> CacheSnapshot {
        CacheSnapshot {
            stats: self.get_stats_snapshot().await,
            keys: self.tracked_keys.read().await.clone(),
            strategies: self.strategies(),
        }
    }

    pub fn ttl_default(&self) -> Duration {
        Duration::from_secs(self.config.default_ttl_secs)
    }

    pub fn ttl_events(&self) -> Duration {
        Duration::from_secs(self.config.events_ttl_secs)
    }

    pub fn events_query_key(query: &EventQuery) -> String {
        format!(
            "{}{}",
            KEY_EVENTS_LIST_PREFIX,
            compact_json(query).unwrap_or_else(|_| "default".to_string())
        )
    }

    pub fn search_trades_key(query: &crate::models::TradeSearchQuery) -> String {
        format!(
            "{}{}",
            KEY_SEARCH_TRADES_PREFIX,
            compact_json(query).unwrap_or_else(|_| "default".to_string())
        )
    }

    pub fn compliance_status_key(address: &str) -> String {
        format!("{}{}", KEY_COMPLIANCE_PREFIX, address.trim())
    }

    // -----------------------------------------------------------------------
    // Soroban RPC read-only call cache helpers
    // -----------------------------------------------------------------------

    /// Cache key for `get_funding_preview(trade_id, buyer)`.
    /// TTL: 30 s — preview data changes only when the trade is funded or cancelled.
    pub fn rpc_funding_preview_key(trade_id: u64, buyer: &str) -> String {
        format!("{}{}:{}", KEY_RPC_FUNDING_PREVIEW_PREFIX, trade_id, buyer.trim())
    }

    /// Cache key for `get_trade_detail(trade_id)`.
    /// TTL: 30 s — invalidated on any trade lifecycle event.
    pub fn rpc_trade_detail_key(trade_id: u64) -> String {
        format!("{}{}", KEY_RPC_TRADE_DETAIL_PREFIX, trade_id)
    }

    /// TTL for Soroban RPC read-only responses (funding preview, trade detail).
    pub fn ttl_rpc(&self) -> Duration {
        Duration::from_secs(self.config.rpc_ttl_secs)
    }

    /// Invalidate all RPC cache entries related to a specific trade.
    /// Called whenever a trade lifecycle event is observed (funded, completed, cancelled, etc.).
    pub async fn invalidate_rpc_trade(&self, trade_id: u64) {
        // Invalidate trade detail
        self.invalidate(&Self::rpc_trade_detail_key(trade_id)).await;
        // Invalidate all funding previews for this trade (any buyer)
        self.invalidate_pattern(&format!("{}{}:*", KEY_RPC_FUNDING_PREVIEW_PREFIX, trade_id)).await;
        // Platform stats may have changed
        self.invalidate(KEY_RPC_PLATFORM_STATS).await;
    }

    /// Invalidate global RPC stats (fee info, platform stats).
    /// Called on fee_updated or fees_withdrawn events.
    pub async fn invalidate_rpc_global(&self) {
        self.invalidate(KEY_RPC_FEE_INFO).await;
        self.invalidate(KEY_RPC_PLATFORM_STATS).await;
    }

    async fn track_key(&self, key: &str) {
        let mut keys = self.tracked_keys.write().await;
        if !keys.iter().any(|existing| existing == key) {
            keys.push(key.to_string());
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

    async fn untrack_key(&self, key: &str) {
        let mut keys = self.tracked_keys.write().await;
        keys.retain(|existing| existing != key);
    }

    async fn build_indexer_status(
        database: &Arc<Database>,
        contract_id: &str,
    ) -> Result<IndexerStatus, AppError> {
        let (total_events, latest) = tokio::try_join!(
            database.get_event_count(None),
            database.get_latest_ledger_global(),
        )?;

        let _ = contract_id;
        let (latest_ledger, latest_ledger_time) = latest
            .map(|(ledger, time)| (Some(ledger), Some(time)))
            .unwrap_or((None, None));

        Ok(IndexerStatus {
            syncing: true,
            latest_ledger,
            latest_ledger_time,
            total_events,
            server_time: Utc::now(),
        })
    }

    async fn build_stats_response(database: &Arc<Database>) -> Result<StatsResponse, AppError> {
        let (total_events, type_counts) = tokio::try_join!(
            database.get_event_count(None),
            database.get_event_type_counts(),
        )?;

        let by_type = type_counts
            .into_iter()
            .map(|(event_type, count)| EventStats { event_type, count })
            .collect();

        Ok(StatsResponse {
            total_events,
            by_type,
        })
    }
}

fn update_hit_rate(stats: &mut CacheStats) {
    let total = stats.hits + stats.misses;
    stats.hit_rate = if total > 0 {
        stats.hits as f64 / total as f64
    } else {
        0.0
    };
}

fn compact_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value).map(|json| {
        json.chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>()
    })
}

fn pattern_matches(pattern: &str, value: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        value.starts_with(prefix)
    } else {
        value == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::{pattern_matches, CacheService, KEY_COMPLIANCE_PREFIX, KEY_EVENTS_LIST_PREFIX, KEY_SEARCH_TRADES_PREFIX};
    use crate::models::{EventQuery, TradeSearchQuery};

    #[test]
    fn builds_deterministic_event_query_keys() {
        let key = CacheService::events_query_key(&EventQuery {
            limit: Some(20),
            offset: Some(0),
            event_type: Some("trade_created".to_string()),
            category: None,
            trade_id: Some(10),
            from_ledger: None,
            to_ledger: None,
            from_time: None,
            to_time: None,
            contract_id: None,
        });

        assert!(key.starts_with(KEY_EVENTS_LIST_PREFIX));
        assert!(key.contains("trade_created"));
        assert!(key.contains("10"));
    }

    #[test]
    fn builds_trade_search_keys() {
        let key = CacheService::search_trades_key(&TradeSearchQuery {
            q: Some("seller".to_string()),
            status: Some("trade_funded".to_string()),
            seller: None,
            buyer: None,
            min_amount: None,
            max_amount: None,
            limit: Some(10),
            offset: Some(0),
        });

        assert!(key.starts_with(KEY_SEARCH_TRADES_PREFIX));
        assert!(key.contains("seller"));
    }

    #[test]
    fn compliance_key_uses_address() {
        let key = CacheService::compliance_status_key("GABC");
        assert_eq!(key, format!("{}GABC", KEY_COMPLIANCE_PREFIX));
    }

    #[test]
    fn pattern_matching_supports_prefix_glob() {
        assert!(pattern_matches("events:*", "events:list:abc"));
        assert!(!pattern_matches("events:*", "stats:platform"));
    }
}
