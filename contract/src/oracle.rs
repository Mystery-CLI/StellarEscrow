//! Oracle integration for real-time asset pricing.
//!
//! # Design
//! - Up to `MAX_ORACLES` oracle contracts can be registered per asset pair (base/quote).
//! - Each oracle must expose `lastprice(base, quote) -> Option<PriceData>`.
//! - `get_price` queries oracles in ascending priority order, returning the first
//!   fresh (non-stale) response — graceful fallback across multiple sources.
//! - A price is stale if `now - timestamp > PRICE_MAX_AGE_SECS`.
//! - `validate_trade_price` checks a trade amount falls within a USD-equivalent range.

use soroban_sdk::{contractclient, contracttype, symbol_short, Address, Env, Vec};

use crate::errors::ContractError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const MAX_ORACLES: u32 = 5;
pub const PRICE_MAX_AGE_SECS: u64 = 300; // 5 minutes

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Price data returned by an oracle contract.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PriceData {
    /// Price in quote asset, scaled by 10^decimals.
    pub price: i128,
    pub decimals: u32,
    /// Ledger timestamp (UTC seconds) when this price was last updated.
    pub timestamp: u64,
}

/// A registered oracle entry for an asset pair.
#[contracttype]
#[derive(Clone, Debug)]
pub struct OracleEntry {
    pub address: Address,
    /// Lower value = queried first.
    pub priority: u32,
}

/// Result of a price-based trade validation.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PriceValidation {
    pub valid: bool,
    pub oracle_price: i128,
    pub decimals: u32,
    /// USD-equivalent value of the trade amount (scaled by 10^decimals).
    pub usd_value: i128,
}

// ---------------------------------------------------------------------------
// Oracle cross-contract interface
// ---------------------------------------------------------------------------

#[contractclient(name = "OracleClient")]
pub trait OracleInterface {
    fn lastprice(env: Env, base: Address, quote: Address) -> Option<PriceData>;
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

fn oracle_key(base: &Address, quote: &Address) -> (soroban_sdk::Symbol, Address, Address) {
    (symbol_short!("ORC"), base.clone(), quote.clone())
}

pub fn get_oracles(env: &Env, base: &Address, quote: &Address) -> Vec<OracleEntry> {
    env.storage()
        .persistent()
        .get(&oracle_key(base, quote))
        .unwrap_or(Vec::new(env))
}

fn save_oracles(env: &Env, base: &Address, quote: &Address, list: &Vec<OracleEntry>) {
    env.storage().persistent().set(&oracle_key(base, quote), list);
}

// ---------------------------------------------------------------------------
// Admin: register / remove
// ---------------------------------------------------------------------------

pub fn register_oracle(
    env: &Env,
    base: &Address,
    quote: &Address,
    oracle: Address,
    priority: u32,
) -> Result<(), ContractError> {
    let mut list = get_oracles(env, base, quote);
    if list.len() >= MAX_ORACLES {
        return Err(ContractError::OracleListFull);
    }
    for i in 0..list.len() {
        if list.get(i).unwrap().address == oracle {
            return Err(ContractError::OracleAlreadyRegistered);
        }
    }
    list.push_back(OracleEntry { address: oracle, priority });
    save_oracles(env, base, quote, &list);
    Ok(())
}

pub fn remove_oracle(
    env: &Env,
    base: &Address,
    quote: &Address,
    oracle: &Address,
) -> Result<(), ContractError> {
    let list = get_oracles(env, base, quote);
    let mut new_list: Vec<OracleEntry> = Vec::new(env);
    let mut found = false;
    for i in 0..list.len() {
        let entry = list.get(i).unwrap();
        if &entry.address == oracle {
            found = true;
        } else {
            new_list.push_back(entry);
        }
    }
    if !found {
        return Err(ContractError::OracleNotFound);
    }
    save_oracles(env, base, quote, &new_list);
    Ok(())
}

// ---------------------------------------------------------------------------
// Price fetching with multi-source fallback
// ---------------------------------------------------------------------------

/// Query oracles in priority order; return first fresh price.
/// Returns `Err(OracleUnavailable)` if all sources fail or are stale.
pub fn get_price(env: &Env, base: &Address, quote: &Address) -> Result<PriceData, ContractError> {
    let list = get_oracles(env, base, quote);
    if list.is_empty() {
        return Err(ContractError::OracleNotFound);
    }

    // Sort by priority ascending (insertion sort — no_std)
    let mut sorted = list.clone();
    let len = sorted.len();
    for i in 1..len {
        let mut j = i;
        while j > 0 {
            let a = sorted.get(j - 1).unwrap();
            let b = sorted.get(j).unwrap();
            if a.priority > b.priority {
                sorted.set(j - 1, b);
                sorted.set(j, a);
                j -= 1;
            } else {
                break;
            }
        }
    }

    let now = env.ledger().timestamp();
    for i in 0..sorted.len() {
        let entry = sorted.get(i).unwrap();
        let client = OracleClient::new(env, &entry.address);
        if let Some(data) = client.lastprice(base, quote) {
            if now.saturating_sub(data.timestamp) <= PRICE_MAX_AGE_SECS {
                crate::events::emit_oracle_price_fetched(env, base.clone(), quote.clone(), data.price, data.decimals);
                return Ok(data);
            }
        }
        // Stale or unavailable — try next oracle
    }

    crate::events::emit_oracle_unavailable(env, base.clone(), quote.clone());
    Err(ContractError::OracleUnavailable)
}

// ---------------------------------------------------------------------------
// Price-based trade validation
// ---------------------------------------------------------------------------

/// Check that `trade_amount` (token base units) falls within `[min_usd, max_usd]`
/// at the current oracle price. Both bounds are in oracle-scaled units (price * 10^decimals).
///
/// On oracle failure returns `Err(OracleUnavailable)` — callers decide whether
/// to block or allow the trade (graceful degradation).
pub fn validate_trade_price(
    env: &Env,
    base: &Address,
    quote: &Address,
    trade_amount: u64,
    min_usd: i128,
    max_usd: i128,
) -> Result<PriceValidation, ContractError> {
    let pd = get_price(env, base, quote)?;
    let scale = 10_i128.pow(pd.decimals);
    let usd_value = (trade_amount as i128)
        .checked_mul(pd.price)
        .ok_or(ContractError::Overflow)?
        .checked_div(scale)
        .ok_or(ContractError::Overflow)?;

    Ok(PriceValidation {
        valid: usd_value >= min_usd && usd_value <= max_usd,
        oracle_price: pd.price,
        decimals: pd.decimals,
        usd_value,
    })
}
