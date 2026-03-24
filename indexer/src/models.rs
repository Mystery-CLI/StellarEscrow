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

// ---- File storage models ----

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FileRecord {
    pub id: Uuid,
    pub owner_id: String,
    pub file_type: String,
    pub original_name: String,
    pub stored_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum: String,
    pub trade_id: Option<i64>,
    pub is_compressed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FileListQuery {
    pub owner_id: String,
    pub file_type: Option<String>,
}
