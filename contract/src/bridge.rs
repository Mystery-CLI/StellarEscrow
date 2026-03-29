//! Cross-chain bridge integration for StellarEscrow.
//!
//! # Design
//! - Supports multiple bridge providers (Wormhole, IBC, Axelar, etc.)
//! - Bridge communication via oracle attestations
//! - Cross-chain validation with timeout and retry logic
//! - Graceful failure handling with automatic rollback
//! - Per-trade bridge metadata tracking
//!
//! # Trade Flow
//! 1. User initiates cross-chain trade on source chain
//! 2. Bridge provider locks/burns tokens on source chain
//! 3. Bridge oracle submits attestation to Stellar contract
//! 4. Contract validates attestation and confirms deposit
//! 5. Trade proceeds normally (funded state)
//! 6. On completion, funds released on destination chain

use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

use crate::errors::ContractError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const MAX_BRIDGE_PROVIDERS: u32 = 10;
pub const BRIDGE_ATTESTATION_TIMEOUT_SECS: u64 = 3600; // 1 hour
pub const BRIDGE_RETRY_LIMIT: u32 = 3;
pub const BRIDGE_CONFIRMATION_BLOCKS: u32 = 12; // Finality threshold

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Supported bridge protocols
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeProvider {
    Wormhole,
    IBC,
    Axelar,
    LayerZero,
    Custom(String), // For extensibility
}

/// Bridge attestation status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttestationStatus {
    Pending,
    Confirmed,
    Failed,
    Expired,
}

/// Cross-chain trade metadata
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainTrade {
    /// Trade ID on Stellar (destination chain)
    pub trade_id: u64,
    /// Source blockchain identifier (e.g., "ethereum", "polygon", "avalanche")
    pub source_chain: String,
    /// Destination chain (always "stellar" for this contract)
    pub dest_chain: String,
    /// Source chain transaction hash
    pub source_tx_hash: String,
    /// Bridge provider used
    pub bridge_provider: BridgeProvider,
    /// Attestation ID from bridge
    pub attestation_id: String,
    /// Current attestation status
    pub attestation_status: AttestationStatus,
    /// Timestamp when attestation was submitted
    pub attestation_timestamp: u64,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Minimum block confirmations required on source chain
    pub min_confirmations: u32,
    /// Current block confirmations on source chain
    pub current_confirmations: u32,
    /// Bridge fee paid (in stroops)
    pub bridge_fee: u64,
    /// Metadata about the bridge transaction
    pub bridge_metadata: Option<String>,
}

/// Bridge provider configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeProviderConfig {
    pub provider: BridgeProvider,
    /// Oracle address that submits attestations
    pub oracle_address: Address,
    /// Whether this provider is active
    pub is_active: bool,
    /// Fee percentage in basis points
    pub fee_bps: u32,
    /// Supported source chains (comma-separated)
    pub supported_chains: String,
    /// Maximum trade amount allowed via this bridge
    pub max_trade_amount: u64,
    /// Minimum trade amount allowed via this bridge
    pub min_trade_amount: u64,
}

/// Bridge attestation from oracle
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeAttestation {
    pub attestation_id: String,
    pub trade_id: u64,
    pub source_chain: String,
    pub source_tx_hash: String,
    pub amount: u64,
    pub recipient: Address,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub provider: BridgeProvider,
}

/// Bridge validation result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeValidation {
    pub valid: bool,
    pub error_code: Option<u32>,
    pub error_message: Option<String>,
    pub confirmations: u32,
}

// ---------------------------------------------------------------------------
// Storage Keys
// ---------------------------------------------------------------------------

fn key_bridge_providers() -> Symbol {
    symbol_short!("BPROV")
}

fn key_bridge_trade(trade_id: u64) -> (Symbol, u64) {
    (symbol_short!("BTRD"), trade_id)
}

fn key_attestation(attestation_id: &String) -> (Symbol, String) {
    (symbol_short!("ATST"), attestation_id.clone())
}

fn key_bridge_nonce() -> Symbol {
    symbol_short!("BNONCE")
}

fn key_bridge_paused() -> Symbol {
    symbol_short!("BPAUSE")
}

// ---------------------------------------------------------------------------
// Bridge Provider Management
// ---------------------------------------------------------------------------

/// Register a new bridge provider
pub fn register_bridge_provider(
    env: &Env,
    config: BridgeProviderConfig,
) -> Result<(), ContractError> {
    let mut providers: Vec<BridgeProviderConfig> = env
        .storage()
        .persistent()
        .get(&key_bridge_providers())
        .unwrap_or(Vec::new(env));

    if providers.len() >= MAX_BRIDGE_PROVIDERS {
        return Err(ContractError::BridgeProviderLimitExceeded);
    }

    // Check for duplicate provider
    for provider in providers.iter() {
        if provider.provider == config.provider {
            return Err(ContractError::BridgeProviderAlreadyRegistered);
        }
    }

    providers.push_back(config);
    env.storage()
        .persistent()
        .set(&key_bridge_providers(), &providers);

    Ok(())
}

/// Get all registered bridge providers
pub fn get_bridge_providers(env: &Env) -> Vec<BridgeProviderConfig> {
    env.storage()
        .persistent()
        .get(&key_bridge_providers())
        .unwrap_or(Vec::new(env))
}

/// Get a specific bridge provider by type
pub fn get_bridge_provider(
    env: &Env,
    provider: &BridgeProvider,
) -> Result<BridgeProviderConfig, ContractError> {
    let providers = get_bridge_providers(env);
    for p in providers.iter() {
        if p.provider == *provider && p.is_active {
            return Ok(p.clone());
        }
    }
    Err(ContractError::BridgeProviderNotFound)
}

/// Deactivate a bridge provider
pub fn deactivate_bridge_provider(
    env: &Env,
    provider: &BridgeProvider,
) -> Result<(), ContractError> {
    let mut providers = get_bridge_providers(env);
    let mut found = false;

    for p in providers.iter_mut() {
        if p.provider == *provider {
            p.is_active = false;
            found = true;
            break;
        }
    }

    if !found {
        return Err(ContractError::BridgeProviderNotFound);
    }

    env.storage()
        .persistent()
        .set(&key_bridge_providers(), &providers);

    Ok(())
}

// ---------------------------------------------------------------------------
// Cross-Chain Trade Management
// ---------------------------------------------------------------------------

/// Create a cross-chain trade record
pub fn create_cross_chain_trade(
    env: &Env,
    trade_id: u64,
    source_chain: String,
    source_tx_hash: String,
    bridge_provider: BridgeProvider,
    attestation_id: String,
    bridge_fee: u64,
) -> Result<CrossChainTrade, ContractError> {
    // Validate provider is active
    let _provider_config = get_bridge_provider(env, &bridge_provider)?;

    let now = env.ledger().timestamp();

    let cross_chain_trade = CrossChainTrade {
        trade_id,
        source_chain,
        dest_chain: String::from_small_copy(env, "stellar"),
        source_tx_hash,
        bridge_provider,
        attestation_id,
        attestation_status: AttestationStatus::Pending,
        attestation_timestamp: now,
        retry_count: 0,
        min_confirmations: BRIDGE_CONFIRMATION_BLOCKS,
        current_confirmations: 0,
        bridge_fee,
        bridge_metadata: None,
    };

    env.storage()
        .persistent()
        .set(&key_bridge_trade(trade_id), &cross_chain_trade);

    Ok(cross_chain_trade)
}

/// Get cross-chain trade info
pub fn get_cross_chain_trade(
    env: &Env,
    trade_id: u64,
) -> Result<CrossChainTrade, ContractError> {
    env.storage()
        .persistent()
        .get(&key_bridge_trade(trade_id))
        .ok_or(ContractError::BridgeTradeNotFound)
}

/// Update attestation status
pub fn update_attestation_status(
    env: &Env,
    trade_id: u64,
    status: AttestationStatus,
    confirmations: u32,
) -> Result<(), ContractError> {
    let mut trade = get_cross_chain_trade(env, trade_id)?;

    trade.attestation_status = status;
    trade.current_confirmations = confirmations;

    // Check for expiry
    let now = env.ledger().timestamp();
    if now > trade.attestation_timestamp + BRIDGE_ATTESTATION_TIMEOUT_SECS {
        trade.attestation_status = AttestationStatus::Expired;
        return Err(ContractError::BridgeTradeExpired);
    }

    env.storage()
        .persistent()
        .set(&key_bridge_trade(trade_id), &trade);

    Ok(())
}

// ---------------------------------------------------------------------------
// Attestation Validation
// ---------------------------------------------------------------------------

/// Validate a bridge attestation
pub fn validate_bridge_attestation(
    env: &Env,
    attestation: &BridgeAttestation,
) -> Result<BridgeValidation, ContractError> {
    // Get provider config
    let provider_config = get_bridge_provider(env, &attestation.provider)?;

    // Verify oracle signature (simplified - in production use cryptographic verification)
    if attestation.signature.len() == 0 {
        return Ok(BridgeValidation {
            valid: false,
            error_code: Some(1),
            error_message: Some(String::from_small_copy(env, "Invalid signature")),
            confirmations: 0,
        });
    }

    // Check amount limits
    if attestation.amount < provider_config.min_trade_amount
        || attestation.amount > provider_config.max_trade_amount
    {
        return Ok(BridgeValidation {
            valid: false,
            error_code: Some(2),
            error_message: Some(String::from_small_copy(env, "Amount out of range")),
            confirmations: 0,
        });
    }

    // Check if attestation already processed
    if let Ok(_existing) = env
        .storage()
        .persistent()
        .get::<(Symbol, String), BridgeAttestation>(&key_attestation(&attestation.attestation_id))
    {
        return Ok(BridgeValidation {
            valid: false,
            error_code: Some(3),
            error_message: Some(String::from_small_copy(env, "Attestation already processed")),
            confirmations: 0,
        });
    }

    // Store attestation
    env.storage()
        .persistent()
        .set(&key_attestation(&attestation.attestation_id), attestation);

    Ok(BridgeValidation {
        valid: true,
        error_code: None,
        error_message: None,
        confirmations: attestation.timestamp as u32,
    })
}

// ---------------------------------------------------------------------------
// Bridge State Management
// ---------------------------------------------------------------------------

/// Pause all bridge operations
pub fn pause_bridge(env: &Env) {
    env.storage().instance().set(&key_bridge_paused(), &true);
}

/// Resume bridge operations
pub fn resume_bridge(env: &Env) {
    env.storage().instance().set(&key_bridge_paused(), &false);
}

/// Check if bridge is paused
pub fn is_bridge_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&key_bridge_paused())
        .unwrap_or(false)
}

/// Get next bridge nonce for idempotency
pub fn get_next_bridge_nonce(env: &Env) -> Result<u64, ContractError> {
    let nonce = env
        .storage()
        .instance()
        .get(&key_bridge_nonce())
        .unwrap_or(0u64);

    let next = nonce.checked_add(1).ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&key_bridge_nonce(), &next);

    Ok(next)
}

// ---------------------------------------------------------------------------
// Bridge Failure Handling
// ---------------------------------------------------------------------------

/// Retry a failed bridge attestation
pub fn retry_bridge_attestation(
    env: &Env,
    trade_id: u64,
) -> Result<(), ContractError> {
    let mut trade = get_cross_chain_trade(env, trade_id)?;

    if trade.retry_count >= BRIDGE_RETRY_LIMIT {
        trade.attestation_status = AttestationStatus::Failed;
        env.storage()
            .persistent()
            .set(&key_bridge_trade(trade_id), &trade);
        return Err(ContractError::BridgeRetryLimitExceeded);
    }

    trade.retry_count += 1;
    trade.attestation_status = AttestationStatus::Pending;

    env.storage()
        .persistent()
        .set(&key_bridge_trade(trade_id), &trade);

    Ok(())
}

/// Rollback a failed cross-chain trade
pub fn rollback_cross_chain_trade(
    env: &Env,
    trade_id: u64,
) -> Result<CrossChainTrade, ContractError> {
    let mut trade = get_cross_chain_trade(env, trade_id)?;

    // Only rollback if attestation failed or expired
    if trade.attestation_status != AttestationStatus::Failed
        && trade.attestation_status != AttestationStatus::Expired
    {
        return Err(ContractError::InvalidStatus);
    }

    trade.attestation_status = AttestationStatus::Failed;

    env.storage()
        .persistent()
        .set(&key_bridge_trade(trade_id), &trade);

    Ok(trade)
}
