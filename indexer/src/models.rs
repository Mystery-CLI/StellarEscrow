use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub contract_id: String,
    pub ledger: i64,
    pub transaction_hash: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCreatedData {
    pub trade_id: u64,
    pub seller: String,
    pub buyer: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeFundedData {
    pub trade_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCompletedData {
    pub trade_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeConfirmedData {
    pub trade_id: u64,
    pub payout: u64,
    pub fee: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeRaisedData {
    pub trade_id: u64,
    pub raised_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolvedData {
    pub trade_id: u64,
    pub resolution: String,
    pub recipient: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCancelledData {
    pub trade_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitratorRegisteredData {
    pub arbitrator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitratorRemovedData {
    pub arbitrator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeUpdatedData {
    pub fee_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeesWithdrawnData {
    pub amount: u64,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub event_type: Option<String>,
    pub trade_id: Option<u64>,
    pub from_ledger: Option<i64>,
    pub to_ledger: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRequest {
    pub from_ledger: i64,
    pub to_ledger: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

// ---- Loading state / status models ----

/// Wraps a paginated list response with metadata for progressive loading.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        let has_more = offset + limit < total;
        Self { data, total, limit, offset, has_more }
    }
}

/// Indexer sync / health status — drives loading indicators on the frontend.
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexerStatus {
    /// Whether the indexer is actively polling.
    pub syncing: bool,
    /// Latest ledger sequence number indexed.
    pub latest_ledger: Option<i64>,
    /// Timestamp of the latest indexed ledger.
    pub latest_ledger_time: Option<DateTime<Utc>>,
    /// Total events stored.
    pub total_events: i64,
    /// Server wall-clock time (UTC).
    pub server_time: DateTime<Utc>,
}

/// Per-event-type counts for dashboard skeleton/stats panels.
#[derive(Debug, Serialize, Deserialize)]
pub struct EventStats {
    pub event_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_events: i64,
    pub by_type: Vec<EventStats>,
}
