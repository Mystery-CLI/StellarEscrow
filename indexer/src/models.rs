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

/// Paginated response wrapper for list endpoints.
#[derive(Debug, Serialize)]
pub struct PagedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSearchQuery {
    pub q: Option<String>,
    pub status: Option<String>,
    pub seller: Option<String>,
    pub buyer: Option<String>,
    pub min_amount: Option<u64>,
    pub max_amount: Option<u64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryQuery {
    pub q: Option<String>,
    pub role: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TradeSearchResult {
    pub trade_id: i64,
    pub seller: String,
    pub buyer: String,
    pub amount: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DiscoveryResult {
    pub address: String,
    pub role: String,
    pub seen_count: i64,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchSuggestion {
    pub term: String,
    pub hits: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchHistoryEntry {
    pub id: i64,
    pub query_text: String,
    pub search_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSearchResponse {
    pub trades: Vec<TradeSearchResult>,
    pub users: Vec<DiscoveryResult>,
    pub arbitrators: Vec<DiscoveryResult>,
    pub suggestions: Vec<SearchSuggestion>,
}