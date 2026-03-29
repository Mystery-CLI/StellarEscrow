//! Automated Market Making (AMM) for StellarEscrow.
//!
//! # Design
//! - Constant-product invariant: `x * y = k`
//! - Each pool is keyed by its numeric id in persistent storage.
//! - LP shares are tracked per provider per pool.
//! - Yield farming: accumulated swap fees are claimable by LPs proportional to their share.
//! - Slippage protection: every swap specifies a `min_out` amount.

use soroban_sdk::{contracttype, symbol_short, token, Address, Env, IntoVal, Val};

use crate::errors::ContractError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default swap fee: 30 bps (0.30 %)
pub const DEFAULT_SWAP_FEE_BPS: u32 = 30;
/// Minimum liquidity locked forever on first deposit (prevents division-by-zero attacks)
pub const MINIMUM_LIQUIDITY: u64 = 1_000;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single liquidity pool.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Pool {
    pub id: u64,
    pub token_a: Address,
    pub token_b: Address,
    /// Reserve of token_a held by the pool (in token's native units).
    pub reserve_a: u64,
    /// Reserve of token_b held by the pool.
    pub reserve_b: u64,
    /// Total LP shares outstanding.
    pub total_shares: u64,
    /// Swap fee in basis points.
    pub fee_bps: u32,
    /// Accumulated fees in token_a units (claimable by LPs).
    pub fees_a: u64,
    /// Accumulated fees in token_b units.
    pub fees_b: u64,
    /// Total volume traded in token_a
    pub volume_a: u64,
    /// Total volume traded in token_b
    pub volume_b: u64,
    /// Pool creation timestamp
    pub created_at: u64,
    /// Last swap timestamp
    pub last_swap_at: u64,
}

/// LP position for a single provider in a pool.
#[contracttype]
#[derive(Clone, Debug)]
pub struct LpPosition {
    pub pool_id: u64,
    pub shares: u64,
    /// Snapshot of fees_a at last claim (for yield calculation).
    pub fee_debt_a: u64,
    /// Snapshot of fees_b at last claim.
    pub fee_debt_b: u64,
    /// Timestamp when position was created
    pub created_at: u64,
    /// Total rewards earned (for yield farming tracking)
    pub total_rewards_a: u64,
    pub total_rewards_b: u64,
}

/// Result returned from a swap.
#[contracttype]
#[derive(Clone, Debug)]
pub struct SwapResult {
    pub amount_out: u64,
    pub fee_charged: u64,
    pub price_impact_bps: u32,
}

// ---------------------------------------------------------------------------
// Storage helpers
// ---------------------------------------------------------------------------

fn pool_key(env: &Env, id: u64) -> Val {
    (symbol_short!("POOL"), id).into_val(env)
}

fn lp_key(env: &Env, pool_id: u64, provider: &Address) -> Val {
    (symbol_short!("LP"), pool_id, provider.clone()).into_val(env)
}

fn get_pool_counter(env: &Env) -> u64 {
    env.storage().instance().get(&symbol_short!("AMM_CTR")).unwrap_or(0u64)
}

fn set_pool_counter(env: &Env, v: u64) {
    env.storage().instance().set(&symbol_short!("AMM_CTR"), &v);
}

fn load_pool(env: &Env, id: u64) -> Result<Pool, ContractError> {
    env.storage()
        .persistent()
        .get(&pool_key(env, id))
        .ok_or(ContractError::AmmPoolNotFound)
}

fn save_pool(env: &Env, pool: &Pool) {
    env.storage().persistent().set(&pool_key(env, pool.id), pool);
}

fn load_lp(env: &Env, pool_id: u64, provider: &Address) -> LpPosition {
    env.storage()
        .persistent()
        .get(&lp_key(env, pool_id, provider))
        .unwrap_or(LpPosition { 
            pool_id, 
            shares: 0, 
            fee_debt_a: 0, 
            fee_debt_b: 0,
            created_at: env.ledger().timestamp(),
            total_rewards_a: 0,
            total_rewards_b: 0,
        })
}

fn save_lp(env: &Env, provider: &Address, pos: &LpPosition) {
    env.storage().persistent().set(&lp_key(env, pos.pool_id, provider), pos);
}

// ---------------------------------------------------------------------------
// Canonical pair ordering
// ---------------------------------------------------------------------------

/// Returns (token_a, token_b) in canonical order so the same pair always maps
/// to the same pool regardless of argument order.
pub fn canonical_pair(a: Address, b: Address) -> (Address, Address) {
    if a < b { (a, b) } else { (b, a) }
}

// ---------------------------------------------------------------------------
// Pool creation
// ---------------------------------------------------------------------------

/// Create a new AMM pool for a token pair.
/// Returns the new pool id.
pub fn create_pool(
    env: &Env,
    token_a: Address,
    token_b: Address,
    fee_bps: u32,
) -> Result<u64, ContractError> {
    if fee_bps > 1000 {
        return Err(ContractError::InvalidFeeBps);
    }
    if token_a == token_b {
        return Err(ContractError::AmmInvalidPair);
    }
    let (ta, tb) = canonical_pair(token_a, token_b);
    let id = get_pool_counter(env)
        .checked_add(1)
        .ok_or(ContractError::Overflow)?;
    set_pool_counter(env, id);
    let pool = Pool {
        id,
        token_a: ta,
        token_b: tb,
        reserve_a: 0,
        reserve_b: 0,
        total_shares: 0,
        fee_bps,
        fees_a: 0,
        fees_b: 0,
        volume_a: 0,
        volume_b: 0,
        created_at: env.ledger().timestamp(),
        last_swap_at: 0,
    };
    save_pool(env, &pool);
    Ok(id)
}

// ---------------------------------------------------------------------------
// Liquidity management
// ---------------------------------------------------------------------------

/// Add liquidity to a pool.
/// Provider must have pre-approved the contract to transfer `amount_a` and `amount_b`.
/// Returns LP shares minted.
pub fn add_liquidity(
    env: &Env,
    pool_id: u64,
    provider: &Address,
    amount_a: u64,
    amount_b: u64,
    min_shares: u64,
) -> Result<u64, ContractError> {
    provider.require_auth();
    if amount_a == 0 || amount_b == 0 {
        return Err(ContractError::InvalidAmount);
    }

    let mut pool = load_pool(env, pool_id)?;

    // Enforce ratio on subsequent deposits
    let (actual_a, actual_b) = if pool.total_shares == 0 {
        (amount_a, amount_b)
    } else {
        let optimal_b = (amount_a as u128)
            .checked_mul(pool.reserve_b as u128)
            .ok_or(ContractError::Overflow)?
            .checked_div(pool.reserve_a as u128)
            .ok_or(ContractError::Overflow)? as u64;

        if optimal_b <= amount_b {
            (amount_a, optimal_b)
        } else {
            let optimal_a = (amount_b as u128)
                .checked_mul(pool.reserve_a as u128)
                .ok_or(ContractError::Overflow)?
                .checked_div(pool.reserve_b as u128)
                .ok_or(ContractError::Overflow)? as u64;
            (optimal_a, amount_b)
        }
    };

    // Mint shares: geometric mean on first deposit, proportional thereafter
    let shares = if pool.total_shares == 0 {
        let product = (actual_a as u128)
            .checked_mul(actual_b as u128)
            .ok_or(ContractError::Overflow)?;
        let sqrt = integer_sqrt(product);
        sqrt.checked_sub(MINIMUM_LIQUIDITY as u128)
            .ok_or(ContractError::InvalidAmount)? as u64
    } else {
        let shares_a = (actual_a as u128)
            .checked_mul(pool.total_shares as u128)
            .ok_or(ContractError::Overflow)?
            .checked_div(pool.reserve_a as u128)
            .ok_or(ContractError::Overflow)? as u64;
        let shares_b = (actual_b as u128)
            .checked_mul(pool.total_shares as u128)
            .ok_or(ContractError::Overflow)?
            .checked_div(pool.reserve_b as u128)
            .ok_or(ContractError::Overflow)? as u64;
        shares_a.min(shares_b)
    };

    if shares < min_shares {
        return Err(ContractError::AmmSlippageExceeded);
    }

    let contract = env.current_contract_address();
    token::Client::new(env, &pool.token_a).transfer(provider, &contract, &(actual_a as i128));
    token::Client::new(env, &pool.token_b).transfer(provider, &contract, &(actual_b as i128));

    pool.reserve_a = pool.reserve_a.checked_add(actual_a).ok_or(ContractError::Overflow)?;
    pool.reserve_b = pool.reserve_b.checked_add(actual_b).ok_or(ContractError::Overflow)?;
    // On first deposit, lock MINIMUM_LIQUIDITY permanently
    pool.total_shares = if pool.total_shares == 0 {
        shares.checked_add(MINIMUM_LIQUIDITY).ok_or(ContractError::Overflow)?
    } else {
        pool.total_shares.checked_add(shares).ok_or(ContractError::Overflow)?
    };
    save_pool(env, &pool);

    let mut pos = load_lp(env, pool_id, provider);
    pos.shares = pos.shares.checked_add(shares).ok_or(ContractError::Overflow)?;
    pos.fee_debt_a = pool.fees_a;
    pos.fee_debt_b = pool.fees_b;
    save_lp(env, provider, &pos);

    Ok(shares)
}

/// Remove liquidity from a pool.
/// Returns (amount_a, amount_b) sent back to the provider.
pub fn remove_liquidity(
    env: &Env,
    pool_id: u64,
    provider: &Address,
    shares: u64,
    min_a: u64,
    min_b: u64,
) -> Result<(u64, u64), ContractError> {
    provider.require_auth();
    if shares == 0 {
        return Err(ContractError::InvalidAmount);
    }

    let mut pool = load_pool(env, pool_id)?;
    let mut pos = load_lp(env, pool_id, provider);

    if pos.shares < shares {
        return Err(ContractError::AmmInsufficientShares);
    }

    let amount_a = (shares as u128)
        .checked_mul(pool.reserve_a as u128)
        .ok_or(ContractError::Overflow)?
        .checked_div(pool.total_shares as u128)
        .ok_or(ContractError::Overflow)? as u64;
    let amount_b = (shares as u128)
        .checked_mul(pool.reserve_b as u128)
        .ok_or(ContractError::Overflow)?
        .checked_div(pool.total_shares as u128)
        .ok_or(ContractError::Overflow)? as u64;

    if amount_a < min_a || amount_b < min_b {
        return Err(ContractError::AmmSlippageExceeded);
    }

    // Claim pending yield before burning shares
    claim_yield_inner(env, &pool, &mut pos, provider)?;

    pool.reserve_a = pool.reserve_a.checked_sub(amount_a).ok_or(ContractError::Overflow)?;
    pool.reserve_b = pool.reserve_b.checked_sub(amount_b).ok_or(ContractError::Overflow)?;
    pool.total_shares = pool.total_shares.checked_sub(shares).ok_or(ContractError::Overflow)?;
    save_pool(env, &pool);

    pos.shares = pos.shares.checked_sub(shares).ok_or(ContractError::Overflow)?;
    save_lp(env, provider, &pos);

    let contract = env.current_contract_address();
    token::Client::new(env, &pool.token_a).transfer(&contract, provider, &(amount_a as i128));
    token::Client::new(env, &pool.token_b).transfer(&contract, provider, &(amount_b as i128));

    Ok((amount_a, amount_b))
}

// ---------------------------------------------------------------------------
// Swap (price discovery + slippage protection)
// ---------------------------------------------------------------------------

/// Swap `amount_in` of `token_in` for the other token in the pool.
/// `min_out` enforces slippage protection — reverts if output < min_out.
pub fn swap(
    env: &Env,
    pool_id: u64,
    caller: &Address,
    token_in: &Address,
    amount_in: u64,
    min_out: u64,
) -> Result<SwapResult, ContractError> {
    caller.require_auth();
    if amount_in == 0 {
        return Err(ContractError::InvalidAmount);
    }

    let mut pool = load_pool(env, pool_id)?;

    let (reserve_in, reserve_out, is_a_in) = if *token_in == pool.token_a {
        (pool.reserve_a, pool.reserve_b, true)
    } else if *token_in == pool.token_b {
        (pool.reserve_b, pool.reserve_a, false)
    } else {
        return Err(ContractError::AmmPoolNotFound);
    };

    let fee = (amount_in as u128)
        .checked_mul(pool.fee_bps as u128)
        .ok_or(ContractError::Overflow)?
        .checked_div(10_000)
        .ok_or(ContractError::Overflow)? as u64;
    let amount_in_after_fee = amount_in.checked_sub(fee).ok_or(ContractError::Overflow)?;

    // Constant-product: out = reserve_out * amount_in_after_fee / (reserve_in + amount_in_after_fee)
    let numerator = (reserve_out as u128)
        .checked_mul(amount_in_after_fee as u128)
        .ok_or(ContractError::Overflow)?;
    let denominator = (reserve_in as u128)
        .checked_add(amount_in_after_fee as u128)
        .ok_or(ContractError::Overflow)?;
    let amount_out = numerator.checked_div(denominator).ok_or(ContractError::Overflow)? as u64;

    if amount_out < min_out {
        return Err(ContractError::AmmSlippageExceeded);
    }

    // Price impact in bps: amount_out / reserve_out * 10000
    let price_impact_bps = ((amount_out as u128)
        .checked_mul(10_000)
        .ok_or(ContractError::Overflow)?
        .checked_div(reserve_out as u128)
        .ok_or(ContractError::Overflow)?) as u32;

    let contract = env.current_contract_address();
    token::Client::new(env, token_in).transfer(caller, &contract, &(amount_in as i128));
    let token_out = if is_a_in { &pool.token_b } else { &pool.token_a };
    token::Client::new(env, token_out).transfer(&contract, caller, &(amount_out as i128));

    if is_a_in {
        pool.reserve_a = pool.reserve_a.checked_add(amount_in).ok_or(ContractError::Overflow)?;
        pool.reserve_b = pool.reserve_b.checked_sub(amount_out).ok_or(ContractError::Overflow)?;
        pool.fees_a = pool.fees_a.checked_add(fee).ok_or(ContractError::Overflow)?;
        pool.volume_a = pool.volume_a.checked_add(amount_in).ok_or(ContractError::Overflow)?;
    } else {
        pool.reserve_b = pool.reserve_b.checked_add(amount_in).ok_or(ContractError::Overflow)?;
        pool.reserve_a = pool.reserve_a.checked_sub(amount_out).ok_or(ContractError::Overflow)?;
        pool.fees_b = pool.fees_b.checked_add(fee).ok_or(ContractError::Overflow)?;
        pool.volume_b = pool.volume_b.checked_add(amount_in).ok_or(ContractError::Overflow)?;
    }
    pool.last_swap_at = env.ledger().timestamp();
    save_pool(env, &pool);

    Ok(SwapResult { amount_out, fee_charged: fee, price_impact_bps })
}

// ---------------------------------------------------------------------------
// Price discovery
// ---------------------------------------------------------------------------

/// Returns the spot price of token_a in terms of token_b (scaled by 1e7).
pub fn spot_price(env: &Env, pool_id: u64) -> Result<u64, ContractError> {
    let pool = load_pool(env, pool_id)?;
    if pool.reserve_a == 0 {
        return Err(ContractError::AmmPoolNotFound);
    }
    let price = (pool.reserve_b as u128)
        .checked_mul(10_000_000)
        .ok_or(ContractError::Overflow)?
        .checked_div(pool.reserve_a as u128)
        .ok_or(ContractError::Overflow)? as u64;
    Ok(price)
}

/// Simulate a swap without executing it. Returns expected output and price impact.
pub fn quote_swap(
    env: &Env,
    pool_id: u64,
    token_in: &Address,
    amount_in: u64,
) -> Result<SwapResult, ContractError> {
    if amount_in == 0 {
        return Err(ContractError::InvalidAmount);
    }
    let pool = load_pool(env, pool_id)?;
    let (reserve_in, reserve_out) = if *token_in == pool.token_a {
        (pool.reserve_a, pool.reserve_b)
    } else if *token_in == pool.token_b {
        (pool.reserve_b, pool.reserve_a)
    } else {
        return Err(ContractError::AmmPoolNotFound);
    };

    let fee = (amount_in as u128)
        .checked_mul(pool.fee_bps as u128)
        .ok_or(ContractError::Overflow)?
        .checked_div(10_000)
        .ok_or(ContractError::Overflow)? as u64;
    let amount_in_after_fee = amount_in.checked_sub(fee).ok_or(ContractError::Overflow)?;
    let amount_out = (reserve_out as u128)
        .checked_mul(amount_in_after_fee as u128)
        .ok_or(ContractError::Overflow)?
        .checked_div(
            (reserve_in as u128)
                .checked_add(amount_in_after_fee as u128)
                .ok_or(ContractError::Overflow)?,
        )
        .ok_or(ContractError::Overflow)? as u64;
    let price_impact_bps = ((amount_out as u128)
        .checked_mul(10_000)
        .ok_or(ContractError::Overflow)?
        .checked_div(reserve_out as u128)
        .ok_or(ContractError::Overflow)?) as u32;

    Ok(SwapResult { amount_out, fee_charged: fee, price_impact_bps })
}

// ---------------------------------------------------------------------------
// Yield farming (fee claiming)
// ---------------------------------------------------------------------------

/// Claim accumulated swap-fee yield for a liquidity provider.
/// Returns (claimed_a, claimed_b).
pub fn claim_yield(
    env: &Env,
    pool_id: u64,
    provider: &Address,
) -> Result<(u64, u64), ContractError> {
    provider.require_auth();
    let pool = load_pool(env, pool_id)?;
    let mut pos = load_lp(env, pool_id, provider);
    let owed_a = pending_fees(pos.shares, pool.fees_a, pos.fee_debt_a, pool.total_shares)?;
    let owed_b = pending_fees(pos.shares, pool.fees_b, pos.fee_debt_b, pool.total_shares)?;
    claim_yield_inner(env, &pool, &mut pos, provider)?;
    save_lp(env, provider, &pos);
    Ok((owed_a, owed_b))
}

fn pending_fees(
    shares: u64,
    total_fees: u64,
    fee_debt: u64,
    total_shares: u64,
) -> Result<u64, ContractError> {
    if total_shares == 0 || shares == 0 {
        return Ok(0);
    }
    Ok((shares as u128)
        .checked_mul(total_fees.saturating_sub(fee_debt) as u128)
        .ok_or(ContractError::Overflow)?
        .checked_div(total_shares as u128)
        .ok_or(ContractError::Overflow)? as u64)
}

fn claim_yield_inner(
    env: &Env,
    pool: &Pool,
    pos: &mut LpPosition,
    provider: &Address,
) -> Result<(), ContractError> {
    let owed_a = pending_fees(pos.shares, pool.fees_a, pos.fee_debt_a, pool.total_shares)?;
    let owed_b = pending_fees(pos.shares, pool.fees_b, pos.fee_debt_b, pool.total_shares)?;

    let contract = env.current_contract_address();
    if owed_a > 0 {
        token::Client::new(env, &pool.token_a).transfer(&contract, provider, &(owed_a as i128));
        pos.total_rewards_a = pos.total_rewards_a.checked_add(owed_a).ok_or(ContractError::Overflow)?;
    }
    if owed_b > 0 {
        token::Client::new(env, &pool.token_b).transfer(&contract, provider, &(owed_b as i128));
        pos.total_rewards_b = pos.total_rewards_b.checked_add(owed_b).ok_or(ContractError::Overflow)?;
    }

    pos.fee_debt_a = pool.fees_a;
    pos.fee_debt_b = pool.fees_b;
    Ok(())
}

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

pub fn get_pool(env: &Env, pool_id: u64) -> Result<Pool, ContractError> {
    load_pool(env, pool_id)
}

pub fn get_lp_position(env: &Env, pool_id: u64, provider: &Address) -> LpPosition {
    load_lp(env, pool_id, provider)
}

// ---------------------------------------------------------------------------
// Integer square root (no_std, no floats)
// ---------------------------------------------------------------------------

fn integer_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
