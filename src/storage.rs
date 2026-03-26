use soroban_sdk::{Address, Env, Vec};

use crate::errors::ContractError;
use crate::types::{
    FilterPreset, OnboardingProgress, PlatformAnalytics, TierConfig, TimelineEntry, Trade,
    TradeTemplate, UserAnalytics, UserPreference, UserProfile, UserTierInfo,
};

// ---------------------------------------------------------------------------
// Instance-storage keys
// Instance-storage keys (contract-wide singletons)
// ---------------------------------------------------------------------------
const INITIALIZED: &str = "INIT";
const ADMIN: &str = "ADMIN";
const USDC_TOKEN: &str = "USDC";
const FEE_BPS: &str = "FEE_BPS";
const TRADE_COUNTER: &str = "COUNTER";
const ACCUMULATED_FEES: &str = "ACC_FEES";
const PAUSED: &str = "PAUSED";
const TIER_CONFIG: &str = "TIER_CFG";
const TEMPLATE_COUNTER: &str = "TMPL_CTR";
const PLATFORM_STATS: &str = "PSTATS";
const PRESET_COUNTER: &str = "PST_CTR";

// ---------------------------------------------------------------------------
// Persistent-storage key prefixes

// ---------------------------------------------------------------------------
// Persistent-storage key prefixes (per-entity)
// ---------------------------------------------------------------------------
const TRADE_PREFIX: &str = "TRADE";
const ARBITRATOR_PREFIX: &str = "ARB";
const ADDR_TRADES_PREFIX: &str = "ADDR_T";
const USER_TIER_PREFIX: &str = "UTIER";
const TEMPLATE_PREFIX: &str = "TMPL";
const USER_PREFIX: &str = "USER";
const USER_PREF_PREFIX: &str = "UPREF";
const USER_ANALYTICS_PREFIX: &str = "USTAT";
const TIMELINE_PREFIX: &str = "TLINE";
const PRESET_PREFIX: &str = "PST";
const USER_PRESETS_PREFIX: &str = "UPST";
const ONBOARDING_PREFIX: &str = "ONBOARD";

// =============================================================================
// Initialization
// =============================================================================

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

pub fn has_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INITIALIZED)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&INITIALIZED, &true);
}

// =============================================================================
// Admin
// =============================================================================

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN, admin);
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage().instance().get(&ADMIN).ok_or(ContractError::NotInitialized)
}

// =============================================================================
// USDC token
// =============================================================================

pub fn set_usdc_token(env: &Env, token: &Address) {
    env.storage().instance().set(&USDC_TOKEN, token);
}

pub fn get_usdc_token(env: &Env) -> Result<Address, ContractError> {
    env.storage().instance().get(&USDC_TOKEN).ok_or(ContractError::NotInitialized)
}

// =============================================================================
// Fee
// =============================================================================

pub fn set_fee_bps(env: &Env, fee_bps: u32) {
    env.storage().instance().set(&FEE_BPS, &fee_bps);
}

pub fn get_fee_bps(env: &Env) -> Result<u32, ContractError> {
    env.storage().instance().get(&FEE_BPS).ok_or(ContractError::NotInitialized)
}

// =============================================================================
// Trade counter
// =============================================================================

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

// =============================================================================
// Accumulated fees
// =============================================================================

pub fn set_accumulated_fees(env: &Env, fees: u64) {
    env.storage().instance().set(&ACCUMULATED_FEES, &fees);
}

pub fn get_accumulated_fees(env: &Env) -> Result<u64, ContractError> {
    env.storage().instance().get(&ACCUMULATED_FEES).ok_or(ContractError::NotInitialized)
}

// =============================================================================
// Pause state
// =============================================================================

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&PAUSED, &paused);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&PAUSED).unwrap_or(false)
}

// =============================================================================
// Trades
// =============================================================================

pub fn save_trade(env: &Env, trade_id: u64, trade: &Trade) {
    let key = (TRADE_PREFIX, trade_id);
    env.storage().persistent().set(&key, trade);
}

pub fn get_trade(env: &Env, trade_id: u64) -> Result<Trade, ContractError> {
    let key = (TRADE_PREFIX, trade_id);
    env.storage().persistent().get(&key).ok_or(ContractError::TradeNotFound)
}

// =============================================================================
// Arbitrators
// =============================================================================

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

// =============================================================================
// Address → trade index
// =============================================================================

/// Append a trade ID to the address's trade index (call for both seller and buyer).
pub fn index_trade_for_address(env: &Env, address: &Address, trade_id: u64) {
    let key = (ADDR_TRADES_PREFIX, address);
    let mut ids: Vec<u64> = env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));
    ids.push_back(trade_id);
    env.storage().persistent().set(&key, &ids);
}

/// Return all trade IDs associated with an address.
pub fn get_trade_ids_for_address(env: &Env, address: &Address) -> Vec<u64> {
    let key = (ADDR_TRADES_PREFIX, address);
    env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env))
}

// =============================================================================
// Tier config
// =============================================================================

pub fn save_tier_config(env: &Env, config: &TierConfig) {
    env.storage().instance().set(&TIER_CONFIG, config);
}

pub fn get_tier_config(env: &Env) -> Option<TierConfig> {
    env.storage().instance().get(&TIER_CONFIG)
}

// =============================================================================
// Per-user tier
// =============================================================================

pub fn save_user_tier(env: &Env, user: &Address, info: &UserTierInfo) {
    let key = (USER_TIER_PREFIX, user);
    env.storage().persistent().set(&key, info);
}

pub fn get_user_tier(env: &Env, user: &Address) -> Option<UserTierInfo> {
    let key = (USER_TIER_PREFIX, user);
    env.storage().persistent().get(&key)
}

// =============================================================================
// Templates
// =============================================================================

pub fn get_template_counter(env: &Env) -> u64 {
    env.storage().instance().get(&TEMPLATE_COUNTER).unwrap_or(0)
}

pub fn increment_template_counter(env: &Env) -> Result<u64, ContractError> {
    let next = get_template_counter(env).checked_add(1).ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&TEMPLATE_COUNTER, &next);
    Ok(next)
}

pub fn save_template(env: &Env, template_id: u64, template: &TradeTemplate) {
    let key = (TEMPLATE_PREFIX, template_id);
    env.storage().persistent().set(&key, template);
}

pub fn get_template(env: &Env, template_id: u64) -> Result<TradeTemplate, ContractError> {
    let key = (TEMPLATE_PREFIX, template_id);
    env.storage().persistent().get(&key).ok_or(ContractError::TemplateNotFound)
}

// =============================================================================
// User profiles
// =============================================================================

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

// =============================================================================
// User preferences
// =============================================================================

pub fn save_preference(env: &Env, address: &Address, pref: &UserPreference) {
    // Store each preference as a separate entry keyed by (prefix, address, key_bytes)
    // Use a soroban Map stored under (prefix, address) to avoid 3-tuple key issues
    let map_key = (USER_PREF_PREFIX, address);
    let mut map: soroban_sdk::Map<soroban_sdk::String, soroban_sdk::String> =
        env.storage().persistent().get(&map_key).unwrap_or(soroban_sdk::Map::new(env));
    map.set(pref.key.clone(), pref.value.clone());
    env.storage().persistent().set(&map_key, &map);
}

pub fn get_preference(
    env: &Env,
    address: &Address,
    pref_key: &soroban_sdk::String,
) -> Option<UserPreference> {
    let map_key = (USER_PREF_PREFIX, address);
    let map: soroban_sdk::Map<soroban_sdk::String, soroban_sdk::String> =
        env.storage().persistent().get(&map_key).unwrap_or(soroban_sdk::Map::new(env));

    map.get(pref_key.clone())
        .map(|value| UserPreference {
            key: pref_key.clone(),
            value,
        })
}

// =============================================================================
// User analytics
// =============================================================================

pub fn save_analytics(env: &Env, analytics: &UserAnalytics) {
    let key = (USER_ANALYTICS_PREFIX, &analytics.address);
    env.storage().persistent().set(&key, analytics);
}

pub fn get_analytics(env: &Env, address: &Address) -> UserAnalytics {
    let key = (USER_ANALYTICS_PREFIX, address);
    env.storage().persistent().get(&key).unwrap_or(UserAnalytics {
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
// Platform analytics
// =============================================================================

pub fn get_platform_analytics(env: &Env) -> PlatformAnalytics {
    env.storage().instance().get(&PLATFORM_STATS).unwrap_or(PlatformAnalytics {
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
// Trade timeline
// =============================================================================

pub fn append_timeline_entry(env: &Env, trade_id: u64, entry: TimelineEntry) {
    let key = (TIMELINE_PREFIX, trade_id);
    let mut entries: Vec<TimelineEntry> = env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));
    entries.push_back(entry);
    env.storage().persistent().set(&key, &entries);
}

pub fn get_timeline(env: &Env, trade_id: u64) -> Vec<TimelineEntry> {
    let key = (TIMELINE_PREFIX, trade_id);
    env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env))
}

// =============================================================================
// Filter presets
// =============================================================================

fn get_preset_counter(env: &Env) -> u64 {
    env.storage().instance().get(&PRESET_COUNTER).unwrap_or(0)
}

pub fn increment_preset_counter(env: &Env) -> Result<u64, ContractError> {
    let next = get_preset_counter(env).checked_add(1).ok_or(ContractError::Overflow)?;
    env.storage().instance().set(&PRESET_COUNTER, &next);
    Ok(next)
}

pub fn save_preset(env: &Env, preset: &FilterPreset) {
    let key = (PRESET_PREFIX, preset.id);
    env.storage().persistent().set(&key, preset);
}

pub fn get_preset(env: &Env, preset_id: u64) -> Result<FilterPreset, ContractError> {
    let key = (PRESET_PREFIX, preset_id);
    env.storage().persistent().get(&key).ok_or(ContractError::PresetNotFound)
}

pub fn delete_preset(env: &Env, preset_id: u64) {
    let key = (PRESET_PREFIX, preset_id);
    env.storage().persistent().remove(&key);
}

/// Append a preset ID to the user's preset index.
pub fn index_preset_for_user(env: &Env, owner: &Address, preset_id: u64) {
    let key = (USER_PRESETS_PREFIX, owner);
    let mut ids: Vec<u64> = env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));
    ids.push_back(preset_id);
    env.storage().persistent().set(&key, &ids);
}

/// Remove a preset ID from the user's preset index.
pub fn remove_preset_from_index(env: &Env, owner: &Address, preset_id: u64) {
    let key = (USER_PRESETS_PREFIX, owner);
    let ids: Vec<u64> = env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));
    let mut updated: Vec<u64> = Vec::new(env);
    for id in ids.iter() {
        if id != preset_id {
            updated.push_back(id);
        }
    }
    env.storage().persistent().set(&key, &updated);
}

/// Return all preset IDs for a user.
pub fn get_preset_ids_for_user(env: &Env, owner: &Address) -> Vec<u64> {
    let key = (USER_PRESETS_PREFIX, owner);
    env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env))
}

// =============================================================================
// Onboarding
// =============================================================================

pub fn save_onboarding(env: &Env, progress: &OnboardingProgress) {
    let key = (ONBOARDING_PREFIX, &progress.address);
    env.storage().persistent().set(&key, progress);
}

pub fn get_onboarding(env: &Env, address: &Address) -> Option<OnboardingProgress> {
    let key = (ONBOARDING_PREFIX, address);
    env.storage().persistent().get(&key)
}

pub fn has_onboarding(env: &Env, address: &Address) -> bool {
    let key = (ONBOARDING_PREFIX, address);
    env.storage().persistent().has(&key)
}
