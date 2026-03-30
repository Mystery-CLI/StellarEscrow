use soroban_sdk::{contracttype, Address, String};

pub const MAX_METADATA_SIZE: u32 = 1024;
pub const MAX_INSURANCE_PREMIUM_BPS: u32 = 1000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OptionalMetadata {
    None,
    Some(String),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
    AwaitingBridge,
    AwaitingBridge, // cross-chain: waiting for bridge oracle confirmation
    BridgeFailed,   // cross-chain: bridge attestation failed
    Triggered,      // price-based trigger executed
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
    Partial(u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArbitrationConfig {
    Single(Address),
}

// ---------------------------------------------------------------------------
// Price Triggers
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriggerAction {
    Cancel,
    Release,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceTrigger {
    pub base: Address,
    pub quote: Address,
    pub target_price: i128,
    /// If true, trigger when price >= target_price. If false, trigger when price <= target_price.
    pub trigger_above: bool,
    pub action: TriggerAction,
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
    pub expiry_time: Option<u64>,
    pub currency: Address,
    pub metadata: OptionalMetadata,
    /// Optional JSON-like string metadata (product info, shipping details, etc.)
    pub metadata: Option<String>,
    /// Optional price-based trigger
    pub trigger: Option<PriceTrigger>,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainInfo {
    pub source_chain: String,
    pub source_tx_hash: String,
    pub expires_at_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePolicy {
    pub provider: Address,
    pub premium: u64,
    pub coverage: u64,
    pub claimed: bool,
}
