use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::errors::ContractError;
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
