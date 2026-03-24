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
    /// Ledger sequence number when the trade was created
    pub created_at: u32,
    /// Ledger sequence number of the last status update
    pub updated_at: u32,
    /// Optional structured metadata (product info, shipping details, etc.)
    pub metadata: Option<TradeMetadata>,
}

/// A richer view of a trade used for history queries
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

// ---------------------------------------------------------------------------
// Fee Tier System
// ---------------------------------------------------------------------------

/// Volume thresholds (in USDC micro-units) for automatic tier upgrades.
/// Bronze: 0+, Silver: 10_000_000_000 (10k USDC), Gold: 100_000_000_000 (100k USDC)
pub const TIER_SILVER_THRESHOLD: u64 = 10_000_000_000;
pub const TIER_GOLD_THRESHOLD: u64 = 100_000_000_000;

/// User membership tier — determines the fee rate applied to their trades.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserTier {
    /// Default tier — uses the platform base fee
    Bronze,
    /// Mid tier — reduced fee rate
    Silver,
    /// Top tier — lowest fee rate
    Gold,
    /// Manually assigned custom fee rate (overrides volume-based tier)
    Custom,
}

/// Per-user tier record stored on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserTierInfo {
    /// Current tier
    pub tier: UserTier,
    /// Cumulative completed trade volume (sum of trade amounts)
    pub total_volume: u64,
    /// Custom fee in basis points — only used when tier == Custom
    pub custom_fee_bps: Option<u32>,
}

/// Tier configuration set by admin — defines fee bps per tier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    /// Fee bps for Bronze (default: platform base fee)
    pub bronze_fee_bps: u32,
    /// Fee bps for Silver
    pub silver_fee_bps: u32,
    /// Fee bps for Gold
    pub gold_fee_bps: u32,
}

/// Filter options for history queries
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryFilter {
    /// Optional status filter
    pub status: Option<TradeStatus>,
    /// Minimum ledger sequence (inclusive)
    pub from_ledger: Option<u32>,
    /// Maximum ledger sequence (inclusive)
    pub to_ledger: Option<u32>,
}

/// Paginated history result
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

/// Maximum length of a template name string
pub const TEMPLATE_NAME_MAX_LEN: u32 = 64;
/// Maximum number of active versions retained per template
pub const TEMPLATE_MAX_VERSIONS: u32 = 10;

/// Predefined terms attached to a template
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateTerms {
    /// Human-readable description of the trade terms
    pub description: String,
    /// Optional default arbitrator for trades from this template
    pub default_arbitrator: Option<Address>,
    /// Optional fixed amount — if set, trades must use this amount
    pub fixed_amount: Option<u64>,
    /// Optional metadata defaults applied to every trade from this template
    pub default_metadata: Option<TradeMetadata>,
}

/// A single versioned snapshot of a template
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateVersion {
    pub version: u32,
    pub terms: TemplateTerms,
    /// Ledger sequence when this version was created
    pub created_at: u32,
}

/// A reusable trade template owned by a seller
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeTemplate {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    /// Current (latest) version number
    pub current_version: u32,
    /// All retained versions (newest last)
    pub versions: Vec<TemplateVersion>,
    /// Whether the template is active and can be used to create trades
    pub active: bool,
    pub created_at: u32,
    pub updated_at: u32,
// =============================================================================
// User Management (Issue #64)
// =============================================================================

/// Verification status of a user, set by admin
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

/// On-chain user profile
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    /// SHA-256 hash of off-chain display name (stored as 32 bytes)
    pub username_hash: soroban_sdk::Bytes,
    /// SHA-256 hash of off-chain contact info
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

/// Per-user preferences stored as a key→value map entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPreference {
    pub key: soroban_sdk::String,
    pub value: soroban_sdk::String,
}

/// Aggregated analytics for a user
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

// =============================================================================
// Admin Panel (Issue #35)
// =============================================================================

/// Aggregated platform-wide analytics for the admin panel
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

// =============================================================================
// Trade Detail View (Issue #31)
// =============================================================================

/// A single entry in the trade status timeline
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineEntry {
    pub status: TradeStatus,
    pub ledger: u32,
}

/// Context-sensitive actions available to a given viewer address
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

/// Complete trade detail view returned by get_trade_detail
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeDetail {
    /// Full trade data
    pub trade: Trade,
    /// Ordered status timeline (Created → current status)
    pub timeline: Vec<TimelineEntry>,
    /// Actions available to the viewer (empty if viewer is not a party)
    pub available_actions: Vec<TradeAction>,
    /// Net payout to seller after fee deduction
    pub seller_payout: u64,
}
