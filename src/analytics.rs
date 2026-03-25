/// Analytics charts and graphs for trade volume, platform metrics, and user statistics.
///
/// All queries are read-only and operate on existing on-chain data.
/// No PII (addresses) is included in exported data.
use soroban_sdk::{Address, Env, Vec};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    get_analytics, get_platform_analytics, get_trade, get_trade_counter,
    get_trade_ids_for_address,
};
use crate::types::{
    AnalyticsFilter, ChartPoint, FeeChartData, StatusDistribution, SuccessRateData,
    TradeStatus, UserStatsSnapshot, VolumeChartData,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Determine which ledger bucket a ledger sequence falls into.
/// Returns the bucket start ledger. If bucket_size == 0, every ledger is its own bucket.
fn bucket_for(ledger: u32, from_ledger: u32, bucket_size: u32) -> u32 {
    if bucket_size == 0 {
        return ledger;
    }
    let offset = ledger.saturating_sub(from_ledger);
    let bucket_index = offset / bucket_size;
    from_ledger + bucket_index * bucket_size
}

/// Check whether a ledger is within the filter range.
fn in_range(ledger: u32, filter: &AnalyticsFilter) -> bool {
    if let Some(from) = filter.from_ledger {
        if ledger < from {
            return false;
        }
    }
    if let Some(to) = filter.to_ledger {
        if ledger > to {
            return false;
        }
    }
    true
}

/// Validate that from_ledger <= to_ledger when both are set.
fn validate_filter(filter: &AnalyticsFilter) -> Result<(), ContractError> {
    if let (Some(from), Some(to)) = (filter.from_ledger, filter.to_ledger) {
        if from > to {
            return Err(ContractError::InvalidLedgerRange);
        }
    }
    Ok(())
}

/// Find or insert a ChartPoint for the given bucket, returning its index.
fn find_or_insert_bucket(points: &mut Vec<ChartPoint>, env: &Env, bucket: u32) -> u32 {
    let len = points.len();
    let mut i: u32 = 0;
    while i < len {
        if points.get(i).unwrap().ledger == bucket {
            return i;
        }
        i += 1;
    }
    points.push_back(ChartPoint { ledger: bucket, value: 0 });
    len
}

// ---------------------------------------------------------------------------
// Platform-level analytics
// ---------------------------------------------------------------------------

/// Build a trade volume chart bucketed by ledger range.
///
/// Iterates all trades up to `trade_counter`, filters by ledger range,
/// and accumulates volume per bucket. Returns ordered ChartPoints.
pub fn get_volume_chart(
    env: &Env,
    filter: AnalyticsFilter,
) -> Result<VolumeChartData, ContractError> {
    validate_filter(&filter)?;

    let from = filter.from_ledger.unwrap_or(0);
    let bucket_size = if filter.bucket_size == 0 { 1 } else { filter.bucket_size };

    let trade_count = get_trade_counter(env).unwrap_or(0);
    let mut points: Vec<ChartPoint> = Vec::new(env);
    let mut total_volume: u64 = 0;
    let mut total_trades: u64 = 0;

    let mut id: u64 = 1;
    while id <= trade_count {
        if let Ok(trade) = get_trade(env, id) {
            if in_range(trade.created_at, &filter) {
                let bucket = bucket_for(trade.created_at, from, bucket_size);
                let idx = find_or_insert_bucket(&mut points, env, bucket);
                let mut pt = points.get(idx).unwrap();
                pt.value = pt.value.saturating_add(trade.amount);
                points.set(idx, pt);
                total_volume = total_volume.saturating_add(trade.amount);
                total_trades += 1;
            }
        }
        id += 1;
    }

    // Sort points ascending by ledger (insertion sort — Soroban Vec has no sort)
    let len = points.len();
    let mut i: u32 = 1;
    while i < len {
        let mut j = i;
        while j > 0 {
            let a = points.get(j - 1).unwrap();
            let b = points.get(j).unwrap();
            if a.ledger > b.ledger {
                points.set(j - 1, b);
                points.set(j, a);
                j -= 1;
            } else {
                break;
            }
        }
        i += 1;
    }

    Ok(VolumeChartData { points, total_volume, total_trades })
}

/// Compute the platform-wide trade success rate.
pub fn get_success_rate(env: &Env) -> SuccessRateData {
    let stats = get_platform_analytics(env);
    let completed = stats.completed_trades;
    let disputed = stats.disputed_trades;
    let cancelled = stats.cancelled_trades;
    let settled = completed.saturating_add(disputed).saturating_add(cancelled);
    let success_rate_bps = if settled == 0 {
        0
    } else {
        ((completed * 10000) / settled) as u32
    };
    SuccessRateData { completed, disputed, cancelled, success_rate_bps }
}

/// Return a status distribution breakdown across all trades.
pub fn get_status_distribution(
    env: &Env,
    filter: AnalyticsFilter,
) -> Result<StatusDistribution, ContractError> {
    validate_filter(&filter)?;

    let trade_count = get_trade_counter(env).unwrap_or(0);
    let mut dist = StatusDistribution {
        created: 0,
        funded: 0,
        completed: 0,
        disputed: 0,
        cancelled: 0,
    };

    let mut id: u64 = 1;
    while id <= trade_count {
        if let Ok(trade) = get_trade(env, id) {
            if in_range(trade.created_at, &filter) {
                match trade.status {
                    TradeStatus::Created => dist.created += 1,
                    TradeStatus::Funded => dist.funded += 1,
                    TradeStatus::Completed => dist.completed += 1,
                    TradeStatus::Disputed => dist.disputed += 1,
                    TradeStatus::Cancelled => dist.cancelled += 1,
                }
            }
        }
        id += 1;
    }

    Ok(dist)
}

/// Build a fee collection chart bucketed by ledger range.
pub fn get_fee_chart(
    env: &Env,
    filter: AnalyticsFilter,
) -> Result<FeeChartData, ContractError> {
    validate_filter(&filter)?;

    let from = filter.from_ledger.unwrap_or(0);
    let bucket_size = if filter.bucket_size == 0 { 1 } else { filter.bucket_size };

    let trade_count = get_trade_counter(env).unwrap_or(0);
    let mut points: Vec<ChartPoint> = Vec::new(env);
    let mut total_fees: u64 = 0;

    let mut id: u64 = 1;
    while id <= trade_count {
        if let Ok(trade) = get_trade(env, id) {
            // Only count fees from completed trades
            if trade.status == TradeStatus::Completed && in_range(trade.updated_at, &filter) {
                let bucket = bucket_for(trade.updated_at, from, bucket_size);
                let idx = find_or_insert_bucket(&mut points, env, bucket);
                let mut pt = points.get(idx).unwrap();
                pt.value = pt.value.saturating_add(trade.fee);
                points.set(idx, pt);
                total_fees = total_fees.saturating_add(trade.fee);
            }
        }
        id += 1;
    }

    // Sort ascending
    let len = points.len();
    let mut i: u32 = 1;
    while i < len {
        let mut j = i;
        while j > 0 {
            let a = points.get(j - 1).unwrap();
            let b = points.get(j).unwrap();
            if a.ledger > b.ledger {
                points.set(j - 1, b);
                points.set(j, a);
                j -= 1;
            } else {
                break;
            }
        }
        i += 1;
    }

    Ok(FeeChartData { points, total_fees })
}

// ---------------------------------------------------------------------------
// User-level analytics
// ---------------------------------------------------------------------------

/// Return a stats snapshot for a single user — drives user stats charts.
pub fn get_user_stats(env: &Env, address: &Address) -> UserStatsSnapshot {
    let stats = get_analytics(env, address);
    let settled = (stats.completed_trades as u64)
        .saturating_add(stats.disputed_trades as u64)
        .saturating_add(stats.cancelled_trades as u64);
    let success_rate_bps = if settled == 0 {
        0
    } else {
        ((stats.completed_trades as u64 * 10000) / settled) as u32
    };
    UserStatsSnapshot {
        address: address.clone(),
        total_trades: stats.total_trades,
        total_volume: stats.total_volume,
        success_rate_bps,
        trades_as_seller: stats.trades_as_seller,
        trades_as_buyer: stats.trades_as_buyer,
    }
}

/// Build a per-user volume chart from their trade history.
pub fn get_user_volume_chart(
    env: &Env,
    address: &Address,
    filter: AnalyticsFilter,
) -> Result<VolumeChartData, ContractError> {
    validate_filter(&filter)?;

    let from = filter.from_ledger.unwrap_or(0);
    let bucket_size = if filter.bucket_size == 0 { 1 } else { filter.bucket_size };

    let ids = get_trade_ids_for_address(env, address);
    let mut points: Vec<ChartPoint> = Vec::new(env);
    let mut total_volume: u64 = 0;
    let mut total_trades: u64 = 0;

    for id in ids.iter() {
        if let Ok(trade) = get_trade(env, id) {
            if in_range(trade.created_at, &filter) {
                let bucket = bucket_for(trade.created_at, from, bucket_size);
                let idx = find_or_insert_bucket(&mut points, env, bucket);
                let mut pt = points.get(idx).unwrap();
                pt.value = pt.value.saturating_add(trade.amount);
                points.set(idx, pt);
                total_volume = total_volume.saturating_add(trade.amount);
                total_trades += 1;
            }
        }
    }

    // Sort ascending
    let len = points.len();
    let mut i: u32 = 1;
    while i < len {
        let mut j = i;
        while j > 0 {
            let a = points.get(j - 1).unwrap();
            let b = points.get(j).unwrap();
            if a.ledger > b.ledger {
                points.set(j - 1, b);
                points.set(j, a);
                j -= 1;
            } else {
                break;
            }
        }
        i += 1;
    }

    Ok(VolumeChartData { points, total_volume, total_trades })
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/// Export platform analytics as a CSV string (no PII — addresses excluded).
///
/// Columns: total_trades,total_volume,total_fees_collected,
///          active_trades,completed_trades,disputed_trades,cancelled_trades
pub fn export_platform_csv(env: &Env) -> soroban_sdk::String {
    let s = get_platform_analytics(env);
    let header = soroban_sdk::String::from_str(
        env,
        "total_trades,total_volume,total_fees_collected,active_trades,completed_trades,disputed_trades,cancelled_trades\n",
    );
    let mut buf = [0u8; 128];
    let mut pos = 0;
    pos = write_u64(&mut buf, pos, s.total_trades);       buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, s.total_volume);       buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, s.total_fees_collected); buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, s.active_trades);      buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, s.completed_trades);   buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, s.disputed_trades);    buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, s.cancelled_trades);   buf[pos] = b'\n'; pos += 1;
    let row = soroban_sdk::String::from_bytes(env, &buf[..pos]);
    events::emit_analytics_exported(env, 0);
    concat_str(env, &header, &row)
}

/// Export volume chart data as CSV (no PII).
///
/// Columns: ledger,volume
pub fn export_volume_csv(env: &Env, data: &VolumeChartData) -> soroban_sdk::String {
    let header = soroban_sdk::String::from_str(env, "ledger,volume\n");
    let mut out = header;
    for pt in data.points.iter() {
        let mut buf = [0u8; 64];
        let mut pos = 0;
        pos = write_u64(&mut buf, pos, pt.ledger as u64); buf[pos] = b','; pos += 1;
        pos = write_u64(&mut buf, pos, pt.value);         buf[pos] = b'\n'; pos += 1;
        let row = soroban_sdk::String::from_bytes(env, &buf[..pos]);
        out = concat_str(env, &out, &row);
    }
    events::emit_analytics_exported(env, 1);
    out
}

/// Export user stats as CSV (no PII — address excluded).
///
/// Columns: total_trades,total_volume,success_rate_bps,trades_as_seller,trades_as_buyer
pub fn export_user_stats_csv(env: &Env, snapshot: &UserStatsSnapshot) -> soroban_sdk::String {
    let header = soroban_sdk::String::from_str(
        env,
        "total_trades,total_volume,success_rate_bps,trades_as_seller,trades_as_buyer\n",
    );
    let mut buf = [0u8; 128];
    let mut pos = 0;
    pos = write_u64(&mut buf, pos, snapshot.total_trades as u64);  buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, snapshot.total_volume);         buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, snapshot.success_rate_bps as u64); buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, snapshot.trades_as_seller as u64); buf[pos] = b','; pos += 1;
    pos = write_u64(&mut buf, pos, snapshot.trades_as_buyer as u64);  buf[pos] = b'\n'; pos += 1;
    let row = soroban_sdk::String::from_bytes(env, &buf[..pos]);
    events::emit_analytics_exported(env, 2);
    concat_str(env, &header, &row)
}

// ---------------------------------------------------------------------------
// Internal string / number helpers (no_std safe)
// ---------------------------------------------------------------------------

fn concat_str(env: &Env, a: &soroban_sdk::String, b: &soroban_sdk::String) -> soroban_sdk::String {
    let a_len = a.len() as usize;
    let b_len = b.len() as usize;
    let total = a_len + b_len;
    let mut buf = [0u8; 512];
    if total <= 512 {
        a.copy_into_slice(&mut buf[..a_len]);
        b.copy_into_slice(&mut buf[a_len..total]);
        soroban_sdk::String::from_bytes(env, &buf[..total])
    } else {
        a.clone()
    }
}

fn write_u64(buf: &mut [u8], mut pos: usize, mut n: u64) -> usize {
    if n == 0 {
        buf[pos] = b'0';
        return pos + 1;
    }
    let start = pos;
    while n > 0 {
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        pos += 1;
    }
    buf[start..pos].reverse();
    pos
}
