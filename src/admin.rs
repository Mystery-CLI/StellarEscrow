use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::storage::{
    get_accumulated_fees, get_fee_bps, get_platform_analytics, get_trade_counter,
    is_paused, save_platform_analytics, set_paused,
};
use crate::types::{PlatformAnalytics, SystemConfig};

/// Transfer admin role to a new address (current admin only).
pub fn transfer_admin(
    env: &Env,
    current_admin: Address,
    new_admin: Address,
) -> Result<(), ContractError> {
    current_admin.require_auth();
    crate::storage::set_admin(env, &new_admin);
    Ok(())
}

/// Pause the contract — blocks new trades (admin only, auth checked by caller).
pub fn pause_contract(env: &Env) {
    set_paused(env, true);
}

/// Unpause the contract (admin only, auth checked by caller).
pub fn unpause_contract(env: &Env) {
    set_paused(env, false);
}

pub fn check_not_paused(env: &Env) -> Result<(), ContractError> {
    if is_paused(env) {
        Err(ContractError::ContractPaused)
    } else {
        Ok(())
    }
}

/// Return a snapshot of current system configuration.
pub fn get_system_config(env: &Env) -> Result<SystemConfig, ContractError> {
    Ok(SystemConfig {
        fee_bps: get_fee_bps(env)?,
        is_paused: is_paused(env),
        trade_counter: get_trade_counter(env)?,
        accumulated_fees: get_accumulated_fees(env)?,
    })
}

/// Return platform-wide analytics.
pub fn get_analytics(env: &Env) -> PlatformAnalytics {
    get_platform_analytics(env)
}

/// Called when a trade is created — updates platform stats.
pub fn on_trade_created(env: &Env, amount: u64) {
    let mut s = get_platform_analytics(env);
    s.total_trades += 1;
    s.total_volume = s.total_volume.saturating_add(amount);
    s.active_trades += 1;
    save_platform_analytics(env, &s);
}

/// Called when a trade is confirmed/completed — updates platform stats.
pub fn on_trade_completed(env: &Env, fee: u64) {
    let mut s = get_platform_analytics(env);
    s.completed_trades += 1;
    s.active_trades = s.active_trades.saturating_sub(1);
    s.total_fees_collected = s.total_fees_collected.saturating_add(fee);
    save_platform_analytics(env, &s);
}

/// Called when a dispute is raised.
pub fn on_trade_disputed(env: &Env) {
    let mut s = get_platform_analytics(env);
    s.disputed_trades += 1;
    save_platform_analytics(env, &s);
}

/// Called when a trade is cancelled.
pub fn on_trade_cancelled(env: &Env) {
    let mut s = get_platform_analytics(env);
    s.cancelled_trades += 1;
    s.active_trades = s.active_trades.saturating_sub(1);
    save_platform_analytics(env, &s);
}
