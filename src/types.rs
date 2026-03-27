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
    pub avatar_hash: Option<soroban_sdk::Bytes>,
    pub verification: VerificationStatus,
    pub two_fa_enabled: bool,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DashboardStats {
    pub platform: PlatformAnalytics,
    pub success_rate_bps: u32,
    pub dispute_rate_bps: u32,
    pub avg_trade_volume: u64,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemConfig {
    pub fee_bps: u32,
    pub is_paused: bool,
    pub trade_counter: u64,
    pub accumulated_fees: u64,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Currency {
    Usdc,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeFormInput {
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub currency: Currency,
    pub arbitrator: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradePreview {
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub currency: Currency,
    pub arbitrator: Option<Address>,
    pub estimated_fee: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundingPreview {
    pub trade_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: u64,
    pub fee: u64,
    pub buyer_balance: u64,
    pub allowance_sufficient: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeSortField {
    CreatedAt,
    UpdatedAt,
    Amount,
    Fee,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortCriterion {
    pub field: TradeSortField,
    pub order: SortOrder,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeFilter {
    pub status: Option<TradeStatus>,
    pub min_amount: Option<u64>,
    pub max_amount: Option<u64>,
    pub from_ledger: Option<u32>,
    pub to_ledger: Option<u32>,
    pub seller: Option<Address>,
    pub buyer: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeSearchPage {
    pub records: Vec<TransactionRecord>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

pub const MAX_PRESETS_PER_USER: u32 = 20;
pub const PRESET_NAME_MAX_LEN: u32 = 64;

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
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChartPoint {
    pub ledger: u32,
    pub value: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeChartData {
    pub points: Vec<ChartPoint>,
    pub total_volume: u64,
    pub total_trades: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuccessRateData {
    pub completed: u64,
    pub disputed: u64,
    pub cancelled: u64,
    pub success_rate_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusDistribution {
    pub created: u64,
    pub funded: u64,
    pub completed: u64,
    pub disputed: u64,
    pub cancelled: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeChartData {
    pub points: Vec<ChartPoint>,
    pub total_fees: u64,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsFilter {
    pub from_ledger: Option<u32>,
    pub to_ledger: Option<u32>,
    pub bucket_size: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OnboardingStep {
    RegisterProfile,
    AcknowledgeFees,
    CreateFirstTemplate,
    CreateFirstTrade,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StepStatus {
    Pending,
    Done,
    Skipped,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardingProgress {
    pub address: Address,
    pub current_step: OnboardingStep,
    pub step_statuses: Vec<StepStatus>,
    pub started_at: u32,
    pub updated_at: u32,
    pub finished: bool,
}
