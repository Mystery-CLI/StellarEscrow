//! On-chain analytics and metrics collection for StellarEscrow.
//!
//! All counters are stored in instance storage (cheap reads) and updated
//! atomically on every state-changing trade operation.
//!
//! # Metrics collected
//! - Trade volume (total USDC moved through the contract)
//! - Trade counts by status (created, funded, completed, disputed, cancelled)
//! - Success rate (completed / (completed + cancelled + disputed))
//! - Arbitrator performance (disputes handled, resolution breakdown)
//! - Platform usage (active trades, total fees collected, unique trade count)

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

use crate::errors::ContractError;

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

fn key_metrics() -> Symbol { symbol_short!("METRICS") }
fn key_arb_stats(env: &Env, arb: &Address) -> (Symbol, Address) {
    (symbol_short!("ARB_STAT"), arb.clone())
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Aggregate platform metrics stored on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformMetrics {
    /// Total USDC volume that has passed through escrow (in stroops).
    pub total_volume: u64,
    /// Number of trades ever created.
    pub trades_created: u64,
    /// Number of trades that reached Funded state.
    pub trades_funded: u64,
    /// Number of trades successfully completed (confirmed by buyer).
    pub trades_completed: u64,
    /// Number of trades that were disputed.
    pub trades_disputed: u64,
    /// Number of trades cancelled.
    pub trades_cancelled: u64,
    /// Total platform fees accumulated (in stroops).
    pub total_fees_collected: u64,
}

/// Per-arbitrator performance metrics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratorMetrics {
    /// Total disputes this arbitrator has resolved.
    pub disputes_resolved: u64,
    /// Resolutions in favour of the buyer.
    pub resolved_to_buyer: u64,
    /// Resolutions in favour of the seller.
    pub resolved_to_seller: u64,
    /// Partial / split resolutions.
    pub resolved_partial: u64,
}

/// Derived statistics computed from `PlatformMetrics`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformStats {
    pub metrics: PlatformMetrics,
    /// Success rate in basis points: completed / (completed + cancelled + disputed) * 10000.
    /// Returns 0 when no terminal trades exist yet.
    pub success_rate_bps: u32,
    /// Dispute rate in basis points: disputed / trades_funded * 10000.
    pub dispute_rate_bps: u32,
    /// Currently active trades (created + funded).
    pub active_trades: u64,
}

// ---------------------------------------------------------------------------
// Storage helpers
// ---------------------------------------------------------------------------

pub fn load_metrics(env: &Env) -> PlatformMetrics {
    env.storage()
        .instance()
        .get(&key_metrics())
        .unwrap_or(PlatformMetrics {
            total_volume: 0,
            trades_created: 0,
            trades_funded: 0,
            trades_completed: 0,
            trades_disputed: 0,
            trades_cancelled: 0,
            total_fees_collected: 0,
        })
}

fn save_metrics(env: &Env, m: &PlatformMetrics) {
    env.storage().instance().set(&key_metrics(), m);
}

pub fn load_arb_metrics(env: &Env, arb: &Address) -> ArbitratorMetrics {
    env.storage()
        .persistent()
        .get(&key_arb_stats(env, arb))
        .unwrap_or(ArbitratorMetrics {
            disputes_resolved: 0,
            resolved_to_buyer: 0,
            resolved_to_seller: 0,
            resolved_partial: 0,
        })
}

fn save_arb_metrics(env: &Env, arb: &Address, m: &ArbitratorMetrics) {
    env.storage().persistent().set(&key_arb_stats(env, arb), m);
}

// ---------------------------------------------------------------------------
// Update hooks — called from lib.rs on each state transition
// ---------------------------------------------------------------------------

pub fn on_trade_created(env: &Env, amount: u64) {
    let mut m = load_metrics(env);
    m.trades_created = m.trades_created.saturating_add(1);
    m.total_volume = m.total_volume.saturating_add(amount);
    save_metrics(env, &m);
}

pub fn on_trade_funded(env: &Env) {
    let mut m = load_metrics(env);
    m.trades_funded = m.trades_funded.saturating_add(1);
    save_metrics(env, &m);
}

pub fn on_trade_completed(env: &Env, fee: u64) {
    let mut m = load_metrics(env);
    m.trades_completed = m.trades_completed.saturating_add(1);
    m.total_fees_collected = m.total_fees_collected.saturating_add(fee);
    save_metrics(env, &m);
}

pub fn on_trade_disputed(env: &Env) {
    let mut m = load_metrics(env);
    m.trades_disputed = m.trades_disputed.saturating_add(1);
    save_metrics(env, &m);
}

pub fn on_trade_cancelled(env: &Env) {
    let mut m = load_metrics(env);
    m.trades_cancelled = m.trades_cancelled.saturating_add(1);
    save_metrics(env, &m);
}

/// Called when an arbitrator resolves a dispute.
/// `resolution` is 0 = buyer, 1 = seller, 2 = partial.
pub fn on_dispute_resolved(env: &Env, arb: &Address, resolution: u8) {
    let mut m = load_arb_metrics(env, arb);
    m.disputes_resolved = m.disputes_resolved.saturating_add(1);
    match resolution {
        0 => m.resolved_to_buyer = m.resolved_to_buyer.saturating_add(1),
        1 => m.resolved_to_seller = m.resolved_to_seller.saturating_add(1),
        _ => m.resolved_partial = m.resolved_partial.saturating_add(1),
    }
    save_arb_metrics(env, arb, &m);
}

// ---------------------------------------------------------------------------
// Query functions
// ---------------------------------------------------------------------------

/// Return raw platform metrics.
pub fn get_metrics(env: &Env) -> PlatformMetrics {
    load_metrics(env)
}

/// Return derived platform statistics including success rate and dispute rate.
pub fn get_stats(env: &Env) -> PlatformStats {
    let m = load_metrics(env);

    let terminal = m.trades_completed
        .saturating_add(m.trades_cancelled)
        .saturating_add(m.trades_disputed);

    let success_rate_bps = if terminal == 0 {
        0u32
    } else {
        ((m.trades_completed as u128 * 10_000) / terminal as u128) as u32
    };

    let dispute_rate_bps = if m.trades_funded == 0 {
        0u32
    } else {
        ((m.trades_disputed as u128 * 10_000) / m.trades_funded as u128) as u32
    };

    let active_trades = m.trades_created
        .saturating_sub(m.trades_completed)
        .saturating_sub(m.trades_cancelled)
        .saturating_sub(m.trades_disputed);

    PlatformStats { metrics: m, success_rate_bps, dispute_rate_bps, active_trades }
}

/// Return performance metrics for a specific arbitrator.
pub fn get_arb_metrics(env: &Env, arb: &Address) -> ArbitratorMetrics {
    load_arb_metrics(env, arb)
}
