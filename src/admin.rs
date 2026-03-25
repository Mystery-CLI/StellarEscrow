use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::storage::{
    get_accumulated_fees, get_fee_bps, get_platform_analytics, get_trade_counter,
    get_trade_ids_for_address, get_trade, is_paused, save_platform_analytics, set_paused,
};
use crate::types::{DashboardStats, PlatformAnalytics, SystemConfig, TradeStatus, VolumeInRange};

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

/// Return a full dashboard snapshot: platform stats + derived rates.
pub fn get_dashboard(env: &Env) -> DashboardStats {
    let p = get_platform_analytics(env);
    let (success_rate_bps, dispute_rate_bps, avg_trade_volume) = if p.total_trades > 0 {
        (
            (p.completed_trades.saturating_mul(10000) / p.total_trades) as u32,
            (p.disputed_trades.saturating_mul(10000) / p.total_trades) as u32,
            p.total_volume / p.total_trades,
        )
    } else {
        (0, 0, 0)
    };
    DashboardStats { platform: p, success_rate_bps, dispute_rate_bps, avg_trade_volume }
}

/// Aggregate trade volume and counts within a ledger range.
/// Scans all trades for `address` (pass admin address to get platform-wide data
/// by iterating all indexed trades, or a user address for per-user range stats).
pub fn get_volume_in_range(
    env: &Env,
    address: &Address,
    from_ledger: u32,
    to_ledger: u32,
) -> Result<VolumeInRange, ContractError> {
    let ids = get_trade_ids_for_address(env, address);
    let mut out = VolumeInRange {
        from_ledger,
        to_ledger,
        trade_count: 0,
        total_volume: 0,
        completed_count: 0,
        disputed_count: 0,
        cancelled_count: 0,
    };
    for id in ids.iter() {
        let trade = get_trade(env, id)?;
        if trade.created_at < from_ledger || trade.created_at > to_ledger {
            continue;
        }
        out.trade_count += 1;
        out.total_volume = out.total_volume.saturating_add(trade.amount);
        match trade.status {
            TradeStatus::Completed => out.completed_count += 1,
            TradeStatus::Disputed => out.disputed_count += 1,
            TradeStatus::Cancelled => out.cancelled_count += 1,
            _ => {}
        }
    }
    Ok(out)
}
