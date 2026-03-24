use soroban_sdk::{Address, Env, Vec};

use crate::errors::ContractError;
use crate::types::{TierConfig, Trade, TradeTemplate, UserTierInfo};

const INITIALIZED: &str = "INIT";
const ADMIN: &str = "ADMIN";
const USDC_TOKEN: &str = "USDC";
const FEE_BPS: &str = "FEE_BPS";
const TRADE_COUNTER: &str = "COUNTER";
const ACCUMULATED_FEES: &str = "ACC_FEES";
const TRADE_PREFIX: &str = "TRADE";
const ARBITRATOR_PREFIX: &str = "ARB";
const ADDR_TRADES_PREFIX: &str = "ADDR_T";
const PAUSED: &str = "PAUSED";
const TIER_CONFIG: &str = "TIER_CFG";
const USER_TIER_PREFIX: &str = "UTIER";
const TEMPLATE_COUNTER: &str = "TMPL_CTR";
const TEMPLATE_PREFIX: &str = "TMPL";

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

pub fn has_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&INITIALIZED, &true);
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage().instance().get(&ADMIN).ok_or(ContractError::NotInitialized)
}

pub fn set_usdc_token(env: &Env, token: &Address) {
    env.storage().instance().set(&USDC_TOKEN, token);
}

pub fn get_usdc_token(env: &Env) -> Result<Address, ContractError> {
    env.storage().instance().get(&USDC_TOKEN).ok_or(ContractError::NotInitialized)
}

pub fn set_fee_bps(env: &Env, fee_bps: u32) {
    env.storage().instance().set(&FEE_BPS, &fee_bps);
}

pub fn get_fee_bps(env: &Env) -> Result<u32, ContractError> {
    env.storage().instance().get(&FEE_BPS).ok_or(ContractError::NotInitialized)
}

pub fn set_trade_counter(env: &Env, counter: u64) {
    env.storage().instance().set(&TRADE_COUNTER, &counter);
}

pub fn get_trade_counter(env: &Env) -> Result<u64, ContractError> {
    env.storage().instance().get(&TRADE_COUNTER).ok_or(ContractError::NotInitialized)
}

pub fn increment_trade_counter(env: &Env) -> Result<u64, ContractError> {
    let current = get_trade_counter(env)?;
    let next = current.checked_add(1).ok_or(ContractError::Overflow)?;
    set_trade_counter(env, next);
    Ok(next)
}

pub fn set_accumulated_fees(env: &Env, fees: u64) {
    env.storage().instance().set(&ACCUMULATED_FEES, &fees);
}

pub fn get_accumulated_fees(env: &Env) -> Result<u64, ContractError> {
    env.storage().instance().get(&ACCUMULATED_FEES).ok_or(ContractError::NotInitialized)
}

pub fn save_trade(env: &Env, trade_id: u64, trade: &Trade) {
    let key = (TRADE_PREFIX, trade_id);
    env.storage().persistent().set(&key, trade);
}

pub fn get_trade(env: &Env, trade_id: u64) -> Result<Trade, ContractError> {
    let key = (TRADE_PREFIX, trade_id);
    env.storage().persistent().get(&key).ok_or(ContractError::TradeNotFound)
}

pub fn save_arbitrator(env: &Env, arbitrator: &Address) {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().set(&key, &true);
}

pub fn remove_arbitrator(env: &Env, arbitrator: &Address) {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().remove(&key);
}

pub fn has_arbitrator(env: &Env, arbitrator: &Address) -> bool {
    let key = (ARBITRATOR_PREFIX, arbitrator);
    env.storage().persistent().has(&key)
}

/// Append a trade ID to the address's trade index (call for both seller and buyer)
pub fn index_trade_for_address(env: &Env, address: &Address, trade_id: u64) {
    let key = (ADDR_TRADES_PREFIX, address);
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(trade_id);
    env.storage().persistent().set(&key, &ids);
}

/// Return all trade IDs associated with an address
pub fn get_trade_ids_for_address(env: &Env, address: &Address) -> Vec<u64> {
    let key = (ADDR_TRADES_PREFIX, address);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}

// ---------------------------------------------------------------------------
// Pause state
// ---------------------------------------------------------------------------

// Tier config storage
// ---------------------------------------------------------------------------

pub fn save_tier_config(env: &Env, config: &TierConfig) {
    env.storage().instance().set(&TIER_CONFIG, config);
}

pub fn get_tier_config(env: &Env) -> Option<TierConfig> {
    env.storage().instance().get(&TIER_CONFIG)
}

// ---------------------------------------------------------------------------
// Per-user tier storage
// ---------------------------------------------------------------------------

pub fn save_user_tier(env: &Env, user: &Address, info: &UserTierInfo) {
    let key = (USER_TIER_PREFIX, user);
    env.storage().persistent().set(&key, info);
}

pub fn get_user_tier(env: &Env, user: &Address) -> Option<UserTierInfo> {
    let key = (USER_TIER_PREFIX, user);
    env.storage().persistent().get(&key)
}

// ---------------------------------------------------------------------------
// Template storage
// ---------------------------------------------------------------------------

pub fn get_template_counter(env: &Env) -> u64 {
    env.storage().instance().get(&TEMPLATE_COUNTER).unwrap_or(0)
}

pub fn increment_template_counter(env: &Env) -> Result<u64, ContractError> {
    let next = get_template_counter(env)
        .checked_add(1)
        .ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&TEMPLATE_COUNTER, &next);
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
// =============================================================================
// User Management storage (Issue #64)
// =============================================================================

use crate::types::{UserAnalytics, UserPreference, UserProfile};

const USER_PREFIX: &str = "USER";
const USER_PREF_PREFIX: &str = "UPREF";
const USER_ANALYTICS_PREFIX: &str = "USTAT";

pub fn save_user(env: &Env, profile: &UserProfile) {
    let key = (USER_PREFIX, &profile.address);
    env.storage().persistent().set(&key, profile);
}

pub fn get_user(env: &Env, address: &Address) -> Option<UserProfile> {
    let key = (USER_PREFIX, address);
    env.storage().persistent().get(&key)
}

pub fn has_user(env: &Env, address: &Address) -> bool {
    let key = (USER_PREFIX, address);
    env.storage().persistent().has(&key)
}

pub fn save_preference(env: &Env, address: &Address, pref: &UserPreference) {
    let key = (USER_PREF_PREFIX, address, &pref.key);
    env.storage().persistent().set(&key, pref);
}

pub fn get_preference(env: &Env, address: &Address, pref_key: &soroban_sdk::String) -> Option<UserPreference> {
    let key = (USER_PREF_PREFIX, address, pref_key);
    env.storage().persistent().get(&key)
}

pub fn save_analytics(env: &Env, analytics: &UserAnalytics) {
    let key = (USER_ANALYTICS_PREFIX, &analytics.address);
    env.storage().persistent().set(&key, analytics);
}

pub fn get_analytics(env: &Env, address: &Address) -> UserAnalytics {
    let key = (USER_ANALYTICS_PREFIX, address);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(UserAnalytics {
            address: address.clone(),
            total_trades: 0,
            trades_as_seller: 0,
            trades_as_buyer: 0,
            total_volume: 0,
            completed_trades: 0,
            disputed_trades: 0,
            cancelled_trades: 0,
        })
}

// =============================================================================
// Admin Panel storage (Issue #35)
// =============================================================================

use crate::types::PlatformAnalytics;

const PAUSED: &str = "PAUSED";
const PLATFORM_STATS: &str = "PSTATS";

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&PAUSED, &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&PAUSED).unwrap_or(false)
}

pub fn get_platform_analytics(env: &Env) -> PlatformAnalytics {
    env.storage()
        .instance()
        .get(&PLATFORM_STATS)
        .unwrap_or(PlatformAnalytics {
            total_trades: 0,
            total_volume: 0,
            total_fees_collected: 0,
            active_trades: 0,
            completed_trades: 0,
            disputed_trades: 0,
            cancelled_trades: 0,
        })
}

pub fn save_platform_analytics(env: &Env, stats: &PlatformAnalytics) {
    env.storage().instance().set(&PLATFORM_STATS, stats);
}

// =============================================================================
// Trade Detail storage (Issue #31)
// =============================================================================

use crate::types::TimelineEntry;

const TIMELINE_PREFIX: &str = "TLINE";

pub fn append_timeline_entry(env: &Env, trade_id: u64, entry: TimelineEntry) {
    let key = (TIMELINE_PREFIX, trade_id);
    let mut entries: Vec<TimelineEntry> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    entries.push_back(entry);
    env.storage().persistent().set(&key, &entries);
}

pub fn get_timeline(env: &Env, trade_id: u64) -> Vec<TimelineEntry> {
    let key = (TIMELINE_PREFIX, trade_id);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}
