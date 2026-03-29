use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    /// High-level category (trade / arb / fee / tmpl / sub / gov / sys / ins / oracle)
    pub category: String,
    /// Schema version from the contract event payload (v field)
    pub schema_version: i32,
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
    /// Filter by event category (e.g. "trade", "arb", "fee", "gov", "sys", "ins", "oracle")
    pub category: Option<String>,
    pub trade_id: Option<u64>,
    pub from_ledger: Option<i64>,
    pub to_ledger: Option<i64>,
    /// ISO-8601 lower bound on event timestamp
    pub from_time: Option<chrono::DateTime<Utc>>,
    /// ISO-8601 upper bound on event timestamp
    pub to_time: Option<chrono::DateTime<Utc>>,
    /// Filter by contract address
    pub contract_id: Option<String>,
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

/// Result of a batch event insert operation.
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchInsertResult {
    pub inserted: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRequest {
    pub from_ledger: i64,
    pub to_ledger: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub event_type: String,
    /// Event category (e.g. "trade", "arb", "fee") — enables client-side filtering
    pub category: String,
    /// Schema version from the contract event payload
    pub version: u32,
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
        Self {
            data,
            total,
            limit,
            offset,
            has_more,
        }
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

// =============================================================================
// Audit Log Models
// =============================================================================

/// A single audit log entry stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub actor: String,
    pub category: String,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub outcome: String,
    pub ledger: Option<i64>,
    pub tx_hash: Option<String>,
    pub metadata: serde_json::Value,
    pub severity: String,
    pub created_at: DateTime<Utc>,
}

/// Payload used to insert a new audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAuditLog {
    pub actor: String,
    pub category: AuditCategory,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub outcome: AuditOutcome,
    pub ledger: Option<i64>,
    pub tx_hash: Option<String>,
    pub metadata: serde_json::Value,
    pub severity: AuditSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategory {
    Security,
    Trade,
    Admin,
    Governance,
    System,
}

impl AuditCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditCategory::Security => "security",
            AuditCategory::Trade => "trade",
            AuditCategory::Admin => "admin",
            AuditCategory::Governance => "governance",
            AuditCategory::System => "system",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
}

impl AuditOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditOutcome::Success => "success",
            AuditOutcome::Failure => "failure",
            AuditOutcome::Denied => "denied",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Info,
    Warn,
    Error,
    Critical,
}

impl AuditSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditSeverity::Info => "info",
            AuditSeverity::Warn => "warn",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        }
    }
}

/// Query parameters for filtering audit logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub actor: Option<String>,
    pub category: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub outcome: Option<String>,
    pub severity: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Aggregated statistics for the analysis endpoint.
#[derive(Debug, Serialize)]
pub struct AuditStats {
    pub total: i64,
    pub by_category: Vec<AuditBucket>,
    pub by_outcome: Vec<AuditBucket>,
    pub by_severity: Vec<AuditBucket>,
    pub top_actors: Vec<AuditBucket>,
    pub top_actions: Vec<AuditBucket>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AuditBucket {
    pub label: String,
    pub count: i64,
}

// =============================================================================
// Search Analytics Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchAnalyticsRow {
    pub date: chrono::NaiveDate,
    pub search_type: String,
    pub query_count: i64,
    pub unique_terms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnalyticsQuery {
    /// ISO date lower bound (inclusive), e.g. "2026-01-01"
    pub from: Option<chrono::NaiveDate>,
    /// ISO date upper bound (inclusive)
    pub to: Option<chrono::NaiveDate>,
    /// Filter to a specific search type (global / trades / discovery)
    pub search_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchAnalyticsResponse {
    pub rows: Vec<SearchAnalyticsRow>,
    /// Top 10 most-searched terms across the requested window
    pub top_terms: Vec<SearchSuggestion>,
    /// Total queries in the window
    pub total_queries: i64,
}

/// Request body for the retention purge endpoint.
#[derive(Debug, Deserialize)]
pub struct RetentionRequest {
    /// Delete logs older than this many days (default 90, max 365).
    pub older_than_days: Option<i64>,
}

/// Response from the retention purge endpoint.
#[derive(Debug, Serialize)]
pub struct RetentionResponse {
    pub deleted: u64,
    pub older_than_days: i64,
}

// =============================================================================
// Notification Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationPreferences {
    pub address: String,
    pub email_enabled: bool,
    pub email_address: Option<String>,
    pub sms_enabled: bool,
    pub phone_number: Option<String>,
    pub push_enabled: bool,
    pub push_token: Option<String>,
    pub on_trade_created: bool,
    pub on_trade_funded: bool,
    pub on_trade_completed: bool,
    pub on_trade_confirmed: bool,
    pub on_dispute_raised: bool,
    pub on_dispute_resolved: bool,
    pub on_trade_cancelled: bool,
    pub updated_at: DateTime<Utc>,
}

/// Upsert payload — all fields optional so callers only send what they want to change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotificationPreferences {
    pub email_enabled: Option<bool>,
    pub email_address: Option<String>,
    pub sms_enabled: Option<bool>,
    pub phone_number: Option<String>,
    pub push_enabled: Option<bool>,
    pub push_token: Option<String>,
    pub on_trade_created: Option<bool>,
    pub on_trade_funded: Option<bool>,
    pub on_trade_completed: Option<bool>,
    pub on_trade_confirmed: Option<bool>,
    pub on_dispute_raised: Option<bool>,
    pub on_dispute_resolved: Option<bool>,
    pub on_trade_cancelled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationLogEntry {
    pub id: i64,
    pub address: String,
    pub channel: String,
    pub template_id: String,
    pub subject: Option<String>,
    pub body: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// User Management Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub address: String,
    pub username_hash: String,
    pub contact_hash: String,
    pub avatar_hash: Option<String>,
    pub verification: String,
    pub two_fa_enabled: bool,
    pub registered_at: i64,
    pub updated_at: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreference {
    pub address: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAnalyticsRow {
    pub address: String,
    pub total_trades: i32,
    pub trades_as_seller: i32,
    pub trades_as_buyer: i32,
    pub total_volume: i64,
    pub completed_trades: i32,
    pub disputed_trades: i32,
    pub cancelled_trades: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserRequest {
    pub address: String,
    pub username_hash: String,
    pub contact_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub username_hash: Option<String>,
    pub contact_hash: Option<String>,
    pub avatar_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPreferenceRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetVerificationRequest {
    pub status: String,
}
