use soroban_sdk::{contracttype, Address, String, Vec};

/// Maximum byte length for a single metadata value string
pub const METADATA_MAX_VALUE_LEN: u32 = 256;
/// Maximum number of key-value pairs in metadata
pub const METADATA_MAX_ENTRIES: u32 = 10;

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
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
    AwaitingBridge, // cross-chain: waiting for bridge oracle confirmation
    BridgeFailed,   // cross-chain: bridge attestation failed
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
    /// Split the net payout (after fee) between buyer and seller.
    /// `buyer_bps` is the buyer's share in basis points (0–10000).
    /// 0 = all to seller, 10000 = all to buyer.
    Partial { buyer_bps: u32 },
}

// ---------------------------------------------------------------------------
// Multi-Signature Arbitration
// ---------------------------------------------------------------------------

/// Configuration for multi-signature arbitration on high-value trades
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    /// List of arbitrators assigned to this trade
    pub arbitrators: Vec<Address>,
    /// Number of votes required to reach consensus (threshold)
    pub threshold: u32,
    /// Timeout in seconds after dispute is raised when voting expires
    pub voting_timeout_seconds: u64,
    /// Ledger timestamp when voting started (set when dispute is raised)
    pub voting_started_at: Option<u64>,
}

/// Individual arbitrator vote on a dispute resolution
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratorVote {
    pub arbitrator: Address,
    pub resolution: DisputeResolution,
    pub timestamp: u64,
}

/// Summary of current voting state for a multi-sig dispute
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VotingSummary {
    pub total_arbitrators: u32,
    pub votes_cast: u32,
    pub threshold: u32,
    pub consensus_resolution: Option<DisputeResolution>,
    pub has_consensus: bool,
    pub voting_expired: bool,
}

/// Arbitration configuration for a trade (single or multi-signature)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArbitrationConfig {
    /// Single arbitrator (existing behavior)
    Single(Address),
    /// Multi-signature arbitration for high-value trades
    MultiSig(MultiSigConfig),
}


#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trade {
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub arbitrator: Option<ArbitrationConfig>,
    pub status: TradeStatus,
    /// Optional Unix timestamp (seconds) after which funds auto-release to seller
    /// if no dispute has been raised. Uses Stellar ledger time (UTC).
    pub expiry_time: Option<u64>,
    /// Token address used for this trade (e.g. USDC, EURC, or any SAC token)
    pub currency: Address,
    /// Optional JSON-like string metadata (product info, shipping details, etc.)
    pub metadata: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KycStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserCompliance {
    pub kyc_status: KycStatus,
    pub aml_cleared: bool,
    pub jurisdiction: String,
}

// ---------------------------------------------------------------------------
// Arbitrator Reputation
// ---------------------------------------------------------------------------

/// Reputation record stored per registered arbitrator.
/// Ratings are 1–5 stars; stored as cumulative sum + count for an average.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratorReputation {
    /// Total disputes assigned to this arbitrator
    pub total_disputes: u32,
    /// Disputes fully resolved (any outcome)
    pub resolved_count: u32,
    /// Resolutions that released funds to the buyer
    pub buyer_wins: u32,
    /// Resolutions that released funds to the seller
    pub seller_wins: u32,
    /// Sum of all star ratings received (1–5 per rating)
    pub rating_sum: u32,
    /// Number of ratings received
    pub rating_count: u32,
}

// ---------------------------------------------------------------------------
// Subscription Model
// ---------------------------------------------------------------------------

/// Duration of a subscription in ledgers (~1 ledger ≈ 5 s; 30 days ≈ 518_400 ledgers)
pub const SUBSCRIPTION_DURATION_LEDGERS: u32 = 518_400;

/// Monthly price in stroops (USDC micro-units) per tier
pub const SUB_PRICE_BASIC: u64 = 5_000_000;   // 5 USDC
pub const SUB_PRICE_PRO: u64 = 15_000_000;    // 15 USDC
pub const SUB_PRICE_ENTERPRISE: u64 = 50_000_000; // 50 USDC

/// Fee discounts in bps applied on top of the tier/base fee
pub const SUB_DISCOUNT_BASIC_BPS: u32 = 20;       // −0.20 %
pub const SUB_DISCOUNT_PRO_BPS: u32 = 50;          // −0.50 %
pub const SUB_DISCOUNT_ENTERPRISE_BPS: u32 = 100;  // −1.00 %

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionTier {
    Basic,
    Pro,
    Enterprise,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub subscriber: Address,
    pub tier: SubscriptionTier,
    /// Ledger sequence at which the subscription expires
    pub expires_at: u32,
    /// Ledger sequence of the last renewal / purchase
    pub renewed_at: u32,
}

// ---------------------------------------------------------------------------
// Governance
// ---------------------------------------------------------------------------

/// Total supply of governance tokens minted at initialization
pub const GOV_TOTAL_SUPPLY: i128 = 1_000_000_000_000_000; // 1 billion (7 decimals)

/// Voting period in ledgers (~7 days)
pub const GOV_VOTING_PERIOD: u32 = 1_209_600;

/// Minimum tokens required to create a proposal
pub const GOV_PROPOSAL_THRESHOLD: i128 = 10_000_000_000; // 10,000 tokens

/// Minimum quorum (% of total supply * 100, i.e. 400 = 4%)
pub const GOV_QUORUM_BPS: u32 = 400;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalAction {
    UpdateFeeBps(u32),
    UpdateTierConfig(TierConfig),
    DistributeFees(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub action: ProposalAction,
    pub votes_for: i128,
    pub votes_against: i128,
    pub status: ProposalStatus,
    pub created_at: u32,
    pub ends_at: u32,
}

// ---------------------------------------------------------------------------
// Privacy Features
// ---------------------------------------------------------------------------

/// Max length for an encrypted data pointer (e.g. IPFS CID or URL)
pub const PRIVACY_DATA_PTR_MAX_LEN: u32 = 256;

/// Commitment to sensitive trade data stored off-chain (e.g. SHA-256 hex)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradePrivacy {
    /// Hash of the plaintext trade details (commitment scheme)
    pub data_hash: String,
    /// Encrypted data pointer (e.g. IPFS CID) — only parties can decrypt
    pub encrypted_ptr: Option<String>,
    /// Whether arbitration is private (arbitrator identity hidden from public)
    pub private_arbitration: bool,
}

/// A selective disclosure grant — allows `grantee` to access private trade data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisclosureGrant {
    pub trade_id: u64,
    pub grantee: Address,
    /// Encrypted decryption key for the grantee (encrypted with grantee's public key off-chain)
    pub encrypted_key: String,
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
    pub default_metadata: Option<String>,
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
// Cross-Chain Bridge Support
// ---------------------------------------------------------------------------

/// Metadata for a cross-chain trade, stored alongside the base Trade.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainInfo {
    /// Identifier of the source chain (e.g. "ethereum", "polygon")
    pub source_chain: String,
    /// The deposit transaction hash on the source chain (as reported by the oracle)
    pub source_tx_hash: String,
    /// Ledger sequence after which the trade can be expired and funds reclaimed
    pub expires_at_ledger: u32,
}

// ---------------------------------------------------------------------------
// Trade Insurance
// ---------------------------------------------------------------------------

/// Maximum insurance premium in basis points (10% of trade amount)
pub const MAX_INSURANCE_PREMIUM_BPS: u32 = 1000;

/// Insurance policy attached to a trade.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePolicy {
    /// Registered provider that underwrites this policy
    pub provider: Address,
    /// Premium paid by the buyer, in USDC stroops (already deducted from escrow)
    pub premium: u64,
    /// Maximum payout the provider will cover beyond the escrowed amount
    pub coverage: u64,
    /// Whether a claim has been paid out
    pub claimed: bool,
}
