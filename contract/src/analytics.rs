//! On-chain analytics and metrics collection for StellarEscrow.
//!
//! # Metrics collected
//! - Trade volume (total USDC moved through the contract)
//! - Trade counts by status (created, funded, completed, disputed, cancelled)
//! - Success rate in basis points: completed / (completed + cancelled + disputed) * 10_000
//! - Dispute rate in basis points: disputed / funded * 10_000
//! - Unique address count (buyers + sellers that have ever interacted)
//! - Time-windowed snapshots (24 h, 7 d, 30 d) via `TimeWindow`
//! - Arbitrator performance (disputes handled, resolution breakdown)

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

fn key_metrics() -> Symbol { symbol_short!("METRICS") }
fn key_window(tag: u32) -> (Symbol, u32) { (symbol_short!("WIN"), tag) }
fn key_unique_count() -> Symbol { symbol_short!("UNIQ_CNT") }
fn key_unique_addr(addr: &Address) -> (Symbol, Address) { (symbol_short!("UNIQ"), addr.clone()) }
fn key_arb_stats(arb: &Address) -> (Symbol, Address) { (symbol_short!("ARB_STAT"), arb.clone()) }

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

/// Metrics accumulated within a rolling time window.
/// Windows are keyed by a `TimeWindow` tag and reset when the window expires.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowMetrics {
    /// Ledger timestamp (UTC seconds) when this window started.
    pub window_start: u64,
    /// Duration of the window in seconds.
    pub window_seconds: u64,
    pub volume: u64,
    pub trades_created: u64,
    pub trades_completed: u64,
    pub trades_cancelled: u64,
    pub trades_disputed: u64,
}

/// Time window selector for `analytics_query`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeWindow {
    /// Last 24 hours (86 400 s).
    Last24h,
    /// Last 7 days (604 800 s).
    Last7d,
    /// Last 30 days (2 592 000 s).
    Last30d,
    /// All-time (no windowing).
    AllTime,
}

/// Per-arbitrator performance metrics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratorMetrics {
    pub disputes_resolved: u64,
    pub resolved_to_buyer: u64,
    pub resolved_to_seller: u64,
    pub resolved_partial: u64,
}

/// Derived statistics computed from `PlatformMetrics`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformStats {
    pub metrics: PlatformMetrics,
    /// Success rate in basis points: completed / (completed + cancelled + disputed) * 10_000.
    pub success_rate_bps: u32,
    /// Dispute rate in basis points: disputed / trades_funded * 10_000.
    pub dispute_rate_bps: u32,
    /// Currently active trades (created + funded − terminal).
    pub active_trades: u64,
}

/// Structured result returned by `analytics_query`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsResult {
    /// All-time aggregate metrics.
    pub all_time: PlatformStats,
    /// Metrics for the requested time window (same as all_time when AllTime is passed).
    pub window: WindowMetrics,
    /// Number of unique addresses (buyers + sellers) that have ever interacted.
    pub unique_addresses: u64,
}

// ---------------------------------------------------------------------------
// Storage helpers
// ---------------------------------------------------------------------------

pub fn load_metrics(env: &Env) -> PlatformMetrics {
    env.storage().instance().get(&key_metrics()).unwrap_or(PlatformMetrics {
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

fn window_tag(w: &TimeWindow) -> u32 {
    match w {
        TimeWindow::Last24h => 0,
        TimeWindow::Last7d  => 1,
        TimeWindow::Last30d => 2,
        TimeWindow::AllTime => 3,
    }
}

fn window_seconds(w: &TimeWindow) -> u64 {
    match w {
        TimeWindow::Last24h => 86_400,
        TimeWindow::Last7d  => 604_800,
        TimeWindow::Last30d => 2_592_000,
        TimeWindow::AllTime => u64::MAX,
    }
}

fn load_window(env: &Env, w: &TimeWindow) -> WindowMetrics {
    let now = env.ledger().timestamp();
    let secs = window_seconds(w);
    env.storage()
        .instance()
        .get(&key_window(window_tag(w)))
        .unwrap_or(WindowMetrics {
            window_start: now,
            window_seconds: secs,
            volume: 0,
            trades_created: 0,
            trades_completed: 0,
            trades_cancelled: 0,
            trades_disputed: 0,
        })
}

/// Load a window, resetting it if the window period has elapsed.
fn load_or_reset_window(env: &Env, w: &TimeWindow) -> WindowMetrics {
    let now = env.ledger().timestamp();
    let secs = window_seconds(w);
    let mut wm = load_window(env, w);
    // Reset if the window has expired (skip for AllTime which never expires)
    if secs != u64::MAX && now.saturating_sub(wm.window_start) >= secs {
        wm = WindowMetrics {
            window_start: now,
            window_seconds: secs,
            volume: 0,
            trades_created: 0,
            trades_completed: 0,
            trades_cancelled: 0,
            trades_disputed: 0,
        };
    }
    wm
}

fn save_window(env: &Env, w: &TimeWindow, wm: &WindowMetrics) {
    env.storage().instance().set(&key_window(window_tag(w)), wm);
}

fn update_all_windows<F: Fn(&mut WindowMetrics)>(env: &Env, f: F) {
    for w in &[TimeWindow::Last24h, TimeWindow::Last7d, TimeWindow::Last30d, TimeWindow::AllTime] {
        let mut wm = load_or_reset_window(env, w);
        f(&mut wm);
        save_window(env, w, &wm);
    }
}

fn load_unique_count(env: &Env) -> u64 {
    env.storage().instance().get(&key_unique_count()).unwrap_or(0u64)
}

fn record_address(env: &Env, addr: &Address) {
    let k = key_unique_addr(addr);
    if !env.storage().persistent().has(&k) {
        env.storage().persistent().set(&k, &true);
        let count = load_unique_count(env).saturating_add(1);
        env.storage().instance().set(&key_unique_count(), &count);
    }
}

pub fn load_arb_metrics(env: &Env, arb: &Address) -> ArbitratorMetrics {
    env.storage()
        .persistent()
        .get(&key_arb_stats(arb))
        .unwrap_or(ArbitratorMetrics {
            disputes_resolved: 0,
            resolved_to_buyer: 0,
            resolved_to_seller: 0,
            resolved_partial: 0,
        })
}

fn save_arb_metrics(env: &Env, arb: &Address, m: &ArbitratorMetrics) {
    env.storage().persistent().set(&key_arb_stats(arb), m);
}

// ---------------------------------------------------------------------------
// Update hooks — called from lib.rs on each state transition
// ---------------------------------------------------------------------------

pub fn on_trade_created(env: &Env, amount: u64, seller: &Address, buyer: &Address) {
    let mut m = load_metrics(env);
    m.trades_created = m.trades_created.saturating_add(1);
    m.total_volume = m.total_volume.saturating_add(amount);
    save_metrics(env, &m);

    update_all_windows(env, |wm| {
        wm.trades_created = wm.trades_created.saturating_add(1);
        wm.volume = wm.volume.saturating_add(amount);
    });

    record_address(env, seller);
    record_address(env, buyer);
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

    update_all_windows(env, |wm| {
        wm.trades_completed = wm.trades_completed.saturating_add(1);
    });
}

pub fn on_trade_disputed(env: &Env) {
    let mut m = load_metrics(env);
    m.trades_disputed = m.trades_disputed.saturating_add(1);
    save_metrics(env, &m);

    update_all_windows(env, |wm| {
        wm.trades_disputed = wm.trades_disputed.saturating_add(1);
    });
}

pub fn on_trade_cancelled(env: &Env) {
    let mut m = load_metrics(env);
    m.trades_cancelled = m.trades_cancelled.saturating_add(1);
    save_metrics(env, &m);

    update_all_windows(env, |wm| {
        wm.trades_cancelled = wm.trades_cancelled.saturating_add(1);
    });
}

/// Called when an arbitrator resolves a dispute.
/// `resolution`: 0 = buyer, 1 = seller, 2 = partial.
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
    compute_stats(m)
}

fn compute_stats(m: PlatformMetrics) -> PlatformStats {
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

/// Unified analytics query returning all-time stats, a time-windowed snapshot,
/// and the unique address count.
///
/// # Arguments
/// * `window` — which time window to include in the result.
///
/// # Returns
/// `AnalyticsResult` containing:
/// - `all_time`: aggregate `PlatformStats` (success rate, dispute rate, active trades)
/// - `window`: `WindowMetrics` for the requested period (resets automatically when expired)
/// - `unique_addresses`: count of distinct buyer/seller addresses ever seen
pub fn analytics_query(env: &Env, window: TimeWindow) -> AnalyticsResult {
    let all_time = compute_stats(load_metrics(env));
    let wm = load_or_reset_window(env, &window);
    let unique_addresses = load_unique_count(env);
    AnalyticsResult { all_time, window: wm, unique_addresses }
}
