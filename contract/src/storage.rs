use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::errors::ContractError;
use crate::types::{TierConfig, Trade, TradeTemplate, UserTierInfo, Subscription, Proposal};
use crate::types::{ArbitratorReputation, TierConfig, Trade, TradeTemplate, UserTierInfo};

const INITIALIZED: &str = "INIT";
const ADMIN: &str = "ADMIN";
const USDC_TOKEN: &str = "USDC";
const FEE_BPS: &str = "FEE_BPS";
const TRADE_COUNTER: &str = "COUNTER";
const ACCUMULATED_FEES: &str = "ACC_FEES";
const TRADE_PREFIX: &str = "TRADE";
const ARBITRATOR_PREFIX: &str = "ARB";
const PAUSED: &str = "PAUSED";
const TIER_CONFIG: &str = "TIER_CFG";
const USER_TIER_PREFIX: &str = "UTIER";
const TEMPLATE_COUNTER: &str = "TMPL_CTR";
const TEMPLATE_PREFIX: &str = "TMPL";
const SUBSCRIPTION_PREFIX: &str = "SUB";
const GOV_TOKEN: &str = "GOV_TKN";
const PROPOSAL_COUNTER: &str = "PROP_CTR";
const PROPOSAL_PREFIX: &str = "PROP";
const VOTE_PREFIX: &str = "VOTE";
const DELEGATE_PREFIX: &str = "DELEG";
const ARB_REP_PREFIX: &str = "ARB_REP";
const ARB_RATED_PREFIX: &str = "ARB_RTD";
const CURRENCY_FEES_PREFIX: &str = "CFEES";
use crate::types::{CrossChainInfo, InsurancePolicy, TierConfig, Trade, TradeTemplate, UserTierInfo};

// Instance storage keys (short symbols, cheapest encoding)
fn key_init() -> Symbol { symbol_short!("INIT") }
fn key_admin() -> Symbol { symbol_short!("ADMIN") }
fn key_usdc() -> Symbol { symbol_short!("USDC") }
fn key_fee_bps() -> Symbol { symbol_short!("FEE_BPS") }
fn key_counter() -> Symbol { symbol_short!("COUNTER") }
fn key_acc_fees() -> Symbol { symbol_short!("ACC_FEES") }
fn key_paused() -> Symbol { symbol_short!("PAUSED") }
fn key_tier_cfg() -> Symbol { symbol_short!("TIER_CFG") }
fn key_tmpl_ctr() -> Symbol { symbol_short!("TMPL_CTR") }
fn key_version() -> Symbol { symbol_short!("VERSION") }
fn key_bridge() -> Symbol { symbol_short!("BRIDGE") }

// Persistent storage key prefixes
const TRADE_PREFIX: &str = "T";
const ARB_PREFIX: &str = "A";
const USER_TIER_PREFIX: &str = "U";
const TEMPLATE_PREFIX: &str = "P";
const XCHAIN_PREFIX: &str = "X";
const INS_PROVIDER_PREFIX: &str = "IP";
const INS_POLICY_PREFIX: &str = "IPL";

// Initialization
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&key_init())
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&key_init(), &true);
}

// Admin
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&key_admin(), admin);
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&key_admin())
        .ok_or(ContractError::NotInitialized)
}

// USDC Token
pub fn set_usdc_token(env: &Env, token: &Address) {
    env.storage().instance().set(&key_usdc(), token);
}

pub fn get_usdc_token(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&key_usdc())
        .ok_or(ContractError::NotInitialized)
}

// Fee BPS
pub fn set_fee_bps(env: &Env, fee_bps: u32) {
    env.storage().instance().set(&key_fee_bps(), &fee_bps);
}

pub fn get_fee_bps(env: &Env) -> Result<u32, ContractError> {
    env.storage()
        .instance()
        .get(&key_fee_bps())
        .ok_or(ContractError::NotInitialized)
}

// Trade Counter
pub fn set_trade_counter(env: &Env, counter: u64) {
    env.storage().instance().set(&key_counter(), &counter);
}

pub fn get_trade_counter(env: &Env) -> Result<u64, ContractError> {
    env.storage()
        .instance()
        .get(&key_counter())
        .ok_or(ContractError::NotInitialized)
}

pub fn increment_trade_counter(env: &Env) -> Result<u64, ContractError> {
    let next = get_trade_counter(env)?
        .checked_add(1)
        .ok_or(ContractError::Overflow)?;
    set_trade_counter(env, next);
    Ok(next)
}

// Accumulated Fees
pub fn set_accumulated_fees(env: &Env, fees: u64) {
    env.storage().instance().set(&key_acc_fees(), &fees);
}

pub fn get_accumulated_fees(env: &Env) -> Result<u64, ContractError> {
    env.storage()
        .instance()
        .get(&key_acc_fees())
        .ok_or(ContractError::NotInitialized)
}

// Per-currency accumulated fees
pub fn get_currency_fees(env: &Env, currency: &Address) -> u64 {
    let key = (CURRENCY_FEES_PREFIX, currency);
    env.storage().persistent().get(&key).unwrap_or(0)
}

pub fn set_currency_fees(env: &Env, currency: &Address, fees: u64) {
    let key = (CURRENCY_FEES_PREFIX, currency);
    env.storage().persistent().set(&key, &fees);
/// Add `delta` to accumulated fees in a single read-modify-write.
pub fn add_accumulated_fees(env: &Env, delta: u64) -> Result<(), ContractError> {
    let current: u64 = env.storage().instance().get(&key_acc_fees()).unwrap_or(0);
    let new_fees = current.checked_add(delta).ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&key_acc_fees(), &new_fees);
    Ok(())
}

// Trades
pub fn save_trade(env: &Env, trade_id: u64, trade: &Trade) {
    let key = (TRADE_PREFIX, trade_id);
    env.storage().persistent().set(&key, trade);
}

pub fn get_trade(env: &Env, trade_id: u64) -> Result<Trade, ContractError> {
    let key = (TRADE_PREFIX, trade_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::TradeNotFound)
}

// Arbitrators
pub fn save_arbitrator(env: &Env, arbitrator: &Address) {
    let key = (ARB_PREFIX, arbitrator);
    env.storage().persistent().set(&key, &true);
}

pub fn remove_arbitrator(env: &Env, arbitrator: &Address) {
    let key = (ARB_PREFIX, arbitrator);
    env.storage().persistent().remove(&key);
}

pub fn has_arbitrator(env: &Env, arbitrator: &Address) -> bool {
    let key = (ARB_PREFIX, arbitrator);
    env.storage().persistent().has(&key)
}

// Pause state
pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&key_paused(), &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&PAUSED).unwrap_or(false)
    env.storage().instance().get(&key_paused()).unwrap_or(false)
}

// Tier config
pub fn save_tier_config(env: &Env, config: &TierConfig) {
    env.storage().instance().set(&key_tier_cfg(), config);
}

pub fn get_tier_config(env: &Env) -> Option<TierConfig> {
    env.storage().instance().get(&key_tier_cfg())
}

// Per-user tier
pub fn save_user_tier(env: &Env, user: &Address, info: &UserTierInfo) {
    let key = (USER_TIER_PREFIX, user);
    env.storage().persistent().set(&key, info);
}

pub fn get_user_tier(env: &Env, user: &Address) -> Option<UserTierInfo> {
    let key = (USER_TIER_PREFIX, user);
    env.storage().persistent().get(&key)
}

// Template storage
pub fn get_template_counter(env: &Env) -> u64 {
    env.storage().instance().get(&key_tmpl_ctr()).unwrap_or(0)
}

pub fn increment_template_counter(env: &Env) -> Result<u64, ContractError> {
    let next = get_template_counter(env)
        .checked_add(1)
        .ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&key_tmpl_ctr(), &next);
    Ok(next)
}

pub fn save_template(env: &Env, template_id: u64, template: &TradeTemplate) {
    let key = (TEMPLATE_PREFIX, template_id);
    env.storage().persistent().set(&key, template);
}

pub fn get_template(env: &Env, template_id: u64) -> Result<TradeTemplate, ContractError> {
    let key = (TEMPLATE_PREFIX, template_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(crate::errors::ContractError::TemplateNotFound)
}

// Subscriptions
pub fn save_subscription(env: &Env, subscriber: &Address, sub: &Subscription) {
    let key = (SUBSCRIPTION_PREFIX, subscriber);
    env.storage().persistent().set(&key, sub);
}

pub fn get_subscription(env: &Env, subscriber: &Address) -> Option<Subscription> {
    let key = (SUBSCRIPTION_PREFIX, subscriber);
    env.storage().persistent().get(&key)
}

pub fn remove_subscription(env: &Env, subscriber: &Address) {
    let key = (SUBSCRIPTION_PREFIX, subscriber);
    env.storage().persistent().remove(&key);
}


// Governance
pub fn set_gov_token(env: &Env, token: &Address) {
    env.storage().instance().set(&GOV_TOKEN, token);
}
pub fn get_gov_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&GOV_TOKEN)
}
pub fn get_proposal_counter(env: &Env) -> u64 {
    env.storage().instance().get(&PROPOSAL_COUNTER).unwrap_or(0)
}
pub fn increment_proposal_counter(env: &Env) -> Result<u64, crate::errors::ContractError> {
    let next = get_proposal_counter(env).checked_add(1).ok_or(crate::errors::ContractError::Overflow)?;
    env.storage().instance().set(&PROPOSAL_COUNTER, &next);
    Ok(next)
}
pub fn save_proposal(env: &Env, id: u64, proposal: &Proposal) {
    let key = (PROPOSAL_PREFIX, id);
    env.storage().persistent().set(&key, proposal);
}
pub fn get_proposal(env: &Env, id: u64) -> Result<Proposal, crate::errors::ContractError> {
    let key = (PROPOSAL_PREFIX, id);
    env.storage().persistent().get(&key).ok_or(crate::errors::ContractError::ProposalNotFound)
}
pub fn has_voted(env: &Env, proposal_id: u64, voter: &Address) -> bool {
    let key = (VOTE_PREFIX, proposal_id, voter);
    env.storage().persistent().has(&key)
}
pub fn mark_voted(env: &Env, proposal_id: u64, voter: &Address) {
    let key = (VOTE_PREFIX, proposal_id, voter);
    env.storage().persistent().set(&key, &true);
}
pub fn set_delegate(env: &Env, delegator: &Address, delegatee: &Address) {
    let key = (DELEGATE_PREFIX, delegator);
    env.storage().persistent().set(&key, delegatee);
}
pub fn get_delegate(env: &Env, delegator: &Address) -> Option<Address> {
    let key = (DELEGATE_PREFIX, delegator);
    env.storage().persistent().get(&key)
}
pub fn remove_delegate(env: &Env, delegator: &Address) {
    let key = (DELEGATE_PREFIX, delegator);
    env.storage().persistent().remove(&key);
// ---------------------------------------------------------------------------
// Arbitrator Reputation
// ---------------------------------------------------------------------------

pub fn get_arbitrator_reputation(env: &Env, arbitrator: &Address) -> ArbitratorReputation {
    let key = (ARB_REP_PREFIX, arbitrator);
    env.storage().persistent().get(&key).unwrap_or(ArbitratorReputation {
        total_disputes: 0,
        resolved_count: 0,
        buyer_wins: 0,
        seller_wins: 0,
        rating_sum: 0,
        rating_count: 0,
    })
}

pub fn save_arbitrator_reputation(env: &Env, arbitrator: &Address, rep: &ArbitratorReputation) {
    let key = (ARB_REP_PREFIX, arbitrator);
    env.storage().persistent().set(&key, rep);
}

/// Returns true if `rater` has already submitted a rating for this trade's arbitrator.
pub fn has_rated(env: &Env, trade_id: u64, rater: &Address) -> bool {
    let key = (ARB_RATED_PREFIX, trade_id, rater);
    env.storage().persistent().has(&key)
}

pub fn mark_rated(env: &Env, trade_id: u64, rater: &Address) {
    let key = (ARB_RATED_PREFIX, trade_id, rater);
    env.storage().persistent().set(&key, &true);
}
        .ok_or(ContractError::TemplateNotFound)
}

// Contract version
pub fn get_version(env: &Env) -> u32 {
    env.storage().instance().get(&key_version()).unwrap_or(1)
}

pub fn set_version(env: &Env, version: u32) {
    env.storage().instance().set(&key_version(), &version);
}

// Bridge oracle
pub fn set_bridge_oracle(env: &Env, oracle: &Address) {
    env.storage().instance().set(&key_bridge(), oracle);
}

pub fn get_bridge_oracle(env: &Env) -> Option<Address> {
    env.storage().instance().get(&key_bridge())
}

// Cross-chain info (keyed by trade_id)
pub fn save_cross_chain_info(env: &Env, trade_id: u64, info: &CrossChainInfo) {
    let key = (XCHAIN_PREFIX, trade_id);
    env.storage().persistent().set(&key, info);
}

pub fn get_cross_chain_info(env: &Env, trade_id: u64) -> Option<CrossChainInfo> {
    let key = (XCHAIN_PREFIX, trade_id);
    env.storage().persistent().get(&key)
}

// Insurance providers (registered by admin, mirrors arbitrator pattern)
pub fn save_insurance_provider(env: &Env, provider: &Address) {
    let key = (INS_PROVIDER_PREFIX, provider);
    env.storage().persistent().set(&key, &true);
}

pub fn remove_insurance_provider(env: &Env, provider: &Address) {
    let key = (INS_PROVIDER_PREFIX, provider);
    env.storage().persistent().remove(&key);
}

pub fn has_insurance_provider(env: &Env, provider: &Address) -> bool {
    let key = (INS_PROVIDER_PREFIX, provider);
    env.storage().persistent().has(&key)
}

// Insurance policies (keyed by trade_id)
pub fn save_insurance_policy(env: &Env, trade_id: u64, policy: &InsurancePolicy) {
    let key = (INS_POLICY_PREFIX, trade_id);
    env.storage().persistent().set(&key, policy);
}

pub fn get_insurance_policy(env: &Env, trade_id: u64) -> Option<InsurancePolicy> {
    let key = (INS_POLICY_PREFIX, trade_id);
    env.storage().persistent().get(&key)
}
