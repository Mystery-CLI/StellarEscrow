use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::errors::ContractError;
use crate::storage::{get_trade, get_trade_counter};
use crate::types::{Trade, TradeStatus};

// ---------------------------------------------------------------------------
// Query parameter types
// ---------------------------------------------------------------------------

/// Sort field for trade queries
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeSortField {
    Id,
    Amount,
}

/// Sort direction
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Filter criteria for trade queries (all fields are optional)
#[contracttype]
#[derive(Clone, Debug)]
pub struct TradeFilter {
    /// Only return trades with this status
    pub status: Option<TradeStatus>,
    /// Only return trades involving this address (as buyer or seller)
    pub participant: Option<Address>,
    /// Minimum trade amount (inclusive)
    pub min_amount: Option<u64>,
    /// Maximum trade amount (inclusive)
    pub max_amount: Option<u64>,
    /// Only return trades with IDs >= this value (useful for time-range proxies)
    pub from_trade_id: Option<u64>,
    /// Only return trades with IDs <= this value
    pub to_trade_id: Option<u64>,
}

/// Pagination parameters
#[contracttype]
#[derive(Clone, Debug)]
pub struct PageParams {
    /// Number of results to skip
    pub offset: u64,
    /// Maximum number of results to return (capped at 100)
    pub limit: u64,
    /// Field to sort by
    pub sort_by: TradeSortField,
    /// Sort direction
    pub direction: SortDirection,
}

/// Aggregated statistics over a set of trades
#[contracttype]
#[derive(Clone, Debug)]
pub struct TradeStats {
    pub total_count: u64,
    pub total_volume: u64,
    pub total_fees: u64,
    pub min_amount: u64,
    pub max_amount: u64,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn matches_filter(trade: &Trade, filter: &TradeFilter) -> bool {
    if let Some(ref status) = filter.status {
        if &trade.status != status {
            return false;
        }
    }
    if let Some(ref participant) = filter.participant {
        if &trade.buyer != participant && &trade.seller != participant {
            return false;
        }
    }
    if let Some(min) = filter.min_amount {
        if trade.amount < min {
            return false;
        }
    }
    if let Some(max) = filter.max_amount {
        if trade.amount > max {
            return false;
        }
    }
    if let Some(from) = filter.from_trade_id {
        if trade.id < from {
            return false;
        }
    }
    if let Some(to) = filter.to_trade_id {
        if trade.id > to {
            return false;
        }
    }
    true
}

/// Collect all trades matching `filter`, then apply pagination + sorting.
/// Returns at most `page.limit` (capped at 100) trades.
pub fn query_trades(
    env: &Env,
    filter: TradeFilter,
    page: PageParams,
) -> Result<Vec<Trade>, ContractError> {
    let total = get_trade_counter(env).unwrap_or(0);
    let limit = page.limit.min(100);

    // Collect matching trades into a fixed-capacity vec
    let mut matched: Vec<Trade> = Vec::new(env);
    for id in 1..=total {
        if let Ok(trade) = get_trade(env, id) {
            if matches_filter(&trade, &filter) {
                matched.push_back(trade);
            }
        }
    }

    // Sort in-place using insertion sort (no alloc, no std)
    let len = matched.len();
    for i in 1..len {
        let mut j = i;
        while j > 0 {
            let a = matched.get(j - 1).unwrap();
            let b = matched.get(j).unwrap();
            let swap = match page.sort_by {
                TradeSortField::Id => match page.direction {
                    SortDirection::Asc => a.id > b.id,
                    SortDirection::Desc => a.id < b.id,
                },
                TradeSortField::Amount => match page.direction {
                    SortDirection::Asc => a.amount > b.amount,
                    SortDirection::Desc => a.amount < b.amount,
                },
            };
            if swap {
                matched.set(j - 1, b);
                matched.set(j, a);
                j -= 1;
            } else {
                break;
            }
        }
    }

    // Apply offset + limit
    let mut result: Vec<Trade> = Vec::new(env);
    let start = page.offset.min(matched.len() as u64) as u32;
    let end = (start as u64 + limit).min(matched.len() as u64) as u32;
    for i in start..end {
        result.push_back(matched.get(i).unwrap());
    }
    Ok(result)
}

/// Compute aggregate statistics over all trades matching `filter`.
pub fn aggregate_trades(env: &Env, filter: TradeFilter) -> Result<TradeStats, ContractError> {
    let total = get_trade_counter(env).unwrap_or(0);
    let mut count: u64 = 0;
    let mut volume: u64 = 0;
    let mut fees: u64 = 0;
    let mut min_amount: u64 = u64::MAX;
    let mut max_amount: u64 = 0;

    for id in 1..=total {
        if let Ok(trade) = get_trade(env, id) {
            if matches_filter(&trade, &filter) {
                count += 1;
                volume = volume.saturating_add(trade.amount);
                fees = fees.saturating_add(trade.fee);
                if trade.amount < min_amount {
                    min_amount = trade.amount;
                }
                if trade.amount > max_amount {
                    max_amount = trade.amount;
                }
            }
        }
    }

    Ok(TradeStats {
        total_count: count,
        total_volume: volume,
        total_fees: fees,
        min_amount: if count == 0 { 0 } else { min_amount },
        max_amount,
    })
}

/// Filter trades by date range (using trade ID as proxy for time).
///
/// Since trade IDs are sequential, we can use them as a proxy for time range.
/// This is more efficient than storing timestamps for each trade.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `from_id` - Start trade ID (inclusive)
/// * `to_id` - End trade ID (inclusive)
///
/// # Returns
/// * `Vec<Trade>` - Trades within the ID range
pub fn query_trades_by_date_range(
    env: &Env,
    from_id: u64,
    to_id: u64,
) -> Result<Vec<Trade>, ContractError> {
    let total = get_trade_counter(env).unwrap_or(0);
    let mut result: Vec<Trade> = Vec::new(env);
    
    for id in from_id..=to_id.min(total) {
        if let Ok(trade) = get_trade(env, id) {
            result.push_back(trade);
        }
    }
    
    Ok(result)
}

/// Get trade statistics for a specific status.
///
/// Convenience function to get stats filtered by status only.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `status` - The trade status to filter by
///
/// # Returns
/// * `TradeStats` - Aggregated statistics for trades with the given status
pub fn get_trade_stats_by_status(
    env: &Env,
    status: TradeStatus,
) -> Result<TradeStats, ContractError> {
    let filter = TradeFilter {
        status: Some(status),
        participant: None,
        min_amount: None,
        max_amount: None,
        from_trade_id: None,
        to_trade_id: None,
    };
    aggregate_trades(env, filter)
}

/// Count trades matching a filter without retrieving them.
///
/// More efficient than query_trades when only the count is needed.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `filter` - The filter criteria
///
/// # Returns
/// * `u64` - Number of matching trades
pub fn count_trades(env: &Env, filter: TradeFilter) -> Result<u64, ContractError> {
    let total = get_trade_counter(env).unwrap_or(0);
    let mut count: u64 = 0;
    
    for id in 1..=total {
        if let Ok(trade) = get_trade(env, id) {
            if matches_filter(&trade, &filter) {
                count += 1;
            }
        }
    }
    
    Ok(count)
}
