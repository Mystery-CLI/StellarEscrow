use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Maximum byte length for a single metadata value string
pub const METADATA_MAX_VALUE_LEN: u32 = 256;
/// Maximum number of key-value pairs in metadata
pub const METADATA_MAX_ENTRIES: u32 = 10;

/// A single metadata key-value entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

/// Structured metadata attached to a trade (e.g. product description, shipping info)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeMetadata {
    pub entries: Vec<MetadataEntry>,
}

/// Optional wrapper for TradeMetadata (Soroban SDK requires enum wrappers for Option<contracttype>)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionalTradeMetadata {
    None,
    Some(TradeMetadata),
}

/// Optional wrapper for TradeStatus
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionalTradeStatus {
    None,
    Some(TradeStatus),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trade {
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub arbitrator: Option<Address>,
    pub status: TradeStatus,
    pub created_at: u32,
    pub updated_at: u32,
    pub metadata: Option<TradeMetadata>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionRecord {
    pub trade_id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub status: TradeStatus,
    pub created_at: u32,
    pub updated_at: u32,
    pub metadata: Option<TradeMetadata>,
}

pub const METADATA_MAX_VALUE_LEN: u32 = 256;
pub const METADATA_MAX_ENTRIES: u32 = 10;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeMetadata {
    pub entries: Vec<MetadataEntry>,
}

// ---------------------------------------------------------------------------
// Fee Tier System
// ---------------------------------------------------------------------------

pub const TIER_SILVER_THRESHOLD: u64 = 10_000_000_000;
pub const TIER_GOLD_THRESHOLD: u64 = 100_000_000_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserTier {
    Bronze,
    Silver,
    Gold,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserTierInfo {
    pub tier: UserTier,
    pub total_volume: u64,
    pub custom_fee_bps: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    pub bronze_fee_bps: u32,
    pub silver_fee_bps: u32,
    pub gold_fee_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryFilter {
    pub status: Option<TradeStatus>,
    pub from_ledger: Option<u32>,
    pub to_ledger: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryPage {
    pub records: Vec<TransactionRecord>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

// ---------------------------------------------------------------------------
// Trade Templates
// ---------------------------------------------------------------------------

pub const TEMPLATE_NAME_MAX_LEN: u32 = 64;
pub const TEMPLATE_MAX_VERSIONS: u32 = 10;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateTerms {
    pub description: String,
    pub default_arbitrator: Option<Address>,
    pub fixed_amount: Option<u64>,
    pub default_metadata: Option<TradeMetadata>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateVersion {
    pub version: u32,
    pub terms: TemplateTerms,
    pub created_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeTemplate {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub current_version: u32,
    pub versions: Vec<TemplateVersion>,
    pub active: bool,
    pub created_at: u32,
    pub updated_at: u32,
}

// ---------------------------------------------------------------------------
// User Management
// ---------------------------------------------------------------------------
// =============================================================================
// User Management
// =============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    pub username_hash: soroban_sdk::Bytes,
    pub contact_hash: soroban_sdk::Bytes,
    /// SHA-256 hash of off-chain avatar image (None if not set)
    pub avatar_hash: Option<soroban_sdk::Bytes>,
    pub verification: VerificationStatus,
    /// Whether two-factor authentication is enabled
    pub two_fa_enabled: bool,
    /// Preferred session timeout in seconds (0 = platform default)
    pub session_timeout_secs: u32,
    pub registered_at: u32,
    pub updated_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPreference {
    pub key: soroban_sdk::String,
    pub value: soroban_sdk::String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserAnalytics {
    pub address: Address,
    pub total_trades: u32,
    pub trades_as_seller: u32,
    pub trades_as_buyer: u32,
    pub total_volume: u64,
    pub completed_trades: u32,
    pub disputed_trades: u32,
    pub cancelled_trades: u32,
}

// ---------------------------------------------------------------------------
// Admin Panel
// ---------------------------------------------------------------------------
// =============================================================================
// Admin Panel
// =============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformAnalytics {
    pub total_trades: u64,
    pub total_volume: u64,
    pub total_fees_collected: u64,
    pub active_trades: u64,
    pub completed_trades: u64,
    pub disputed_trades: u64,
    pub cancelled_trades: u64,
}

/// Full analytics dashboard snapshot — single call for all dashboard metrics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DashboardStats {
    /// Platform-wide counters
    pub platform: PlatformAnalytics,
    /// Success rate in basis points (completed * 10000 / total_trades), 0 if no trades
    pub success_rate_bps: u32,
    /// Dispute rate in basis points (disputed * 10000 / total_trades), 0 if no trades
    pub dispute_rate_bps: u32,
    /// Average trade volume (total_volume / total_trades), 0 if no trades
    pub avg_trade_volume: u64,
}

/// Trade volume aggregated over a ledger range — for date-range charts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeInRange {
    pub from_ledger: u32,
    pub to_ledger: u32,
    pub trade_count: u64,
    pub total_volume: u64,
    pub completed_count: u64,
    pub disputed_count: u64,
    pub cancelled_count: u64,
}

/// System configuration snapshot returned to admin
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemConfig {
    pub fee_bps: u32,
    pub is_paused: bool,
    pub trade_counter: u64,
    pub accumulated_fees: u64,
}

// ---------------------------------------------------------------------------
// Trade Detail View
// ---------------------------------------------------------------------------
// =============================================================================
// Trade Detail View
// =============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineEntry {
    pub status: TradeStatus,
    pub ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeAction {
    Fund,
    Complete,
    ConfirmReceipt,
    RaiseDispute,
    Cancel,
    ResolveDispute,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeDetail {
    pub trade: Trade,
    pub timeline: Vec<TimelineEntry>,
    pub available_actions: Vec<TradeAction>,
    pub seller_payout: u64,
}

// ---------------------------------------------------------------------------
// Advanced Filtering & Sorting
// ---------------------------------------------------------------------------

/// Field to sort trades by.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeSortField {
    CreatedAt,
    UpdatedAt,
    Amount,
    Fee,
}

/// A single sort criterion: field + direction.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortCriterion {
    pub field: TradeSortField,
    pub order: SortOrder,
}

/// Multi-criteria filter for advanced trade search.
/// All set fields are ANDed together.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeFilter {
    /// Filter by trade status
    pub status: Option<TradeStatus>,
    /// Minimum trade amount (inclusive)
    pub min_amount: Option<u64>,
    /// Maximum trade amount (inclusive)
    pub max_amount: Option<u64>,
    /// Minimum created_at ledger (inclusive)
    pub from_ledger: Option<u32>,
    /// Maximum created_at ledger (inclusive)
    pub to_ledger: Option<u32>,
    /// Only return trades where this address is seller
    pub seller: Option<Address>,
    /// Only return trades where this address is buyer
    pub buyer: Option<Address>,
}

/// Paginated result for advanced trade search.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeSearchPage {
    pub records: Vec<TransactionRecord>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

/// Maximum number of presets a user can save.
pub const MAX_PRESETS_PER_USER: u32 = 20;
/// Maximum length of a preset name.
pub const PRESET_NAME_MAX_LEN: u32 = 64;

/// A saved filter preset owned by a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilterPreset {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub filter: TradeFilter,
    pub sort: SortCriterion,
    pub created_at: u32,
    pub updated_at: u32,
// Analytics Charts & Graphs
// ---------------------------------------------------------------------------

/// A single data point for a time-series chart (ledger bucket → value).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChartPoint {
    /// Ledger sequence representing the start of this bucket
    pub ledger: u32,
    /// Aggregated value for this bucket (volume, count, etc.)
    pub value: u64,
}

/// Trade volume chart data — bucketed by ledger range.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeChartData {
    /// Ordered data points (ascending ledger)
    pub points: Vec<ChartPoint>,
    /// Total volume across all points
    pub total_volume: u64,
    /// Total number of trades across all points
    pub total_trades: u64,
}

/// Success rate snapshot — completed vs total settled trades.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessRateData {
    pub completed: u64,
    pub disputed: u64,
    pub cancelled: u64,
    /// Basis points (0–10000): completed / (completed + disputed + cancelled) * 10000
    pub success_rate_bps: u32,
}

/// Per-status trade count breakdown for a status distribution chart.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusDistribution {
    pub created: u64,
    pub funded: u64,
    pub completed: u64,
    pub disputed: u64,
    pub cancelled: u64,
}

/// Fee collection chart data — bucketed by ledger range.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeChartData {
    pub points: Vec<ChartPoint>,
    pub total_fees: u64,
}

/// Aggregated analytics snapshot for a single user — drives user stats charts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStatsSnapshot {
    pub address: Address,
    pub total_trades: u32,
    pub total_volume: u64,
    pub success_rate_bps: u32,
    pub trades_as_seller: u32,
    pub trades_as_buyer: u32,
}

/// Filter for analytics queries — ledger range only (no PII).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsFilter {
    /// Inclusive lower bound (ledger sequence). None = from genesis.
    pub from_ledger: Option<u32>,
    /// Inclusive upper bound (ledger sequence). None = current ledger.
    pub to_ledger: Option<u32>,
    /// Number of ledgers per bucket for time-series charts (0 = no bucketing).
    pub bucket_size: u32,
// =============================================================================
// Onboarding Flow
// =============================================================================

/// The ordered steps in the onboarding sequence.
/// Each variant maps to a discrete, skippable tutorial/setup stage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OnboardingStep {
    /// Step 1: User registers their profile (username + contact hashes)
    RegisterProfile,
    /// Step 2: User acknowledges platform fee structure and tier system
    AcknowledgeFees,
    /// Step 3: User creates their first trade template
    CreateFirstTemplate,
    /// Step 4: User creates their first trade
    CreateFirstTrade,
    /// Step 5: Onboarding complete
    Completed,
}

/// Status of a single onboarding step
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepStatus {
    /// Not yet started
    Pending,
    /// User completed this step
    Done,
    /// User explicitly skipped this step
    Skipped,
}

/// Persistent onboarding progress record for a user
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardingProgress {
    pub address: Address,
    /// The next step the user should take (or Completed)
    pub current_step: OnboardingStep,
    /// Status of each step in order: [RegisterProfile, AcknowledgeFees, CreateFirstTemplate, CreateFirstTrade]
    pub step_statuses: Vec<StepStatus>,
    /// Ledger sequence when onboarding was started
    pub started_at: u32,
    /// Ledger sequence of the last update (0 if never updated after start)
    pub updated_at: u32,
    /// Whether the user has fully completed or exited onboarding
    pub finished: bool,
}
