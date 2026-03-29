/// Arbitrator reputation: query helpers and reputation-based selection.
///
/// Storage (get/save_arbitrator_reputation, has_rated, mark_rated) lives in
/// storage.rs. This module adds the computation layer on top.

use soroban_sdk::{Address, Env, Vec};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    get_arbitrator_reputation, has_arbitrator, has_rated, mark_rated,
    save_arbitrator_reputation,
};
use crate::types::{ArbitratorReputation, TradeStatus};

// ---------------------------------------------------------------------------
// Rating
// ---------------------------------------------------------------------------

/// Submit a 1–5 star rating for the arbitrator of a disputed/resolved trade.
/// Only the buyer or seller may rate, once each per trade.
pub fn rate_arbitrator(
    env: &Env,
    trade_id: u64,
    rater: &Address,
    arbitrator: &Address,
    buyer: &Address,
    seller: &Address,
    status: &TradeStatus,
    stars: u32,
) -> Result<(), ContractError> {
    if stars < 1 || stars > 5 {
        return Err(ContractError::InvalidRating);
    }
    if rater != buyer && rater != seller {
        return Err(ContractError::Unauthorized);
    }
    // Only allow rating once a dispute has been raised
    if *status != TradeStatus::Disputed {
        return Err(ContractError::InvalidStatus);
    }
    if !has_arbitrator(env, arbitrator) {
        return Err(ContractError::ArbitratorNotRegistered);
    }
    if has_rated(env, trade_id, rater) {
        return Err(ContractError::AlreadyRated);
    }
    mark_rated(env, trade_id, rater);

    let mut rep = get_arbitrator_reputation(env, arbitrator);
    rep.rating_sum = rep.rating_sum.saturating_add(stars);
    rep.rating_count = rep.rating_count.saturating_add(1);
    save_arbitrator_reputation(env, arbitrator, &rep);

    events::emit_arb_rated(env, arbitrator.clone(), trade_id, rater.clone(), stars);
    events::emit_arb_rep_updated(
        env,
        arbitrator.clone(),
        rep.resolved_count,
        rep.rating_sum,
        rep.rating_count,
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Computed statistics
// ---------------------------------------------------------------------------

/// Average star rating scaled ×100 (e.g. 450 = 4.50 stars). Returns 0 if unrated.
pub fn average_rating_x100(rep: &ArbitratorReputation) -> u32 {
    if rep.rating_count == 0 {
        return 0;
    }
    rep.rating_sum
        .saturating_mul(100)
        .checked_div(rep.rating_count)
        .unwrap_or(0)
}

/// Resolution rate in basis points (0–10000). Returns 0 if no disputes assigned.
pub fn resolution_rate_bps(rep: &ArbitratorReputation) -> u32 {
    if rep.total_disputes == 0 {
        return 0;
    }
    ((rep.resolved_count as u64)
        .saturating_mul(10_000)
        .checked_div(rep.total_disputes as u64)
        .unwrap_or(0)) as u32
}

/// Composite score (0–10000): 60 % resolution rate + 40 % normalised rating.
/// Rating is normalised so 5 stars → 10000: avg_rating_x100 × 20.
pub fn composite_score(rep: &ArbitratorReputation) -> u32 {
    let rr = resolution_rate_bps(rep) as u64;
    let ar = (average_rating_x100(rep) as u64).saturating_mul(20).min(10_000);
    ((rr.saturating_mul(6) + ar.saturating_mul(4)) / 10).min(10_000) as u32
}

// ---------------------------------------------------------------------------
// Reputation-based selection
// ---------------------------------------------------------------------------

/// Return the registered arbitrator with the highest composite score from
/// `candidates`. Ties broken by order. Errors if none are registered.
pub fn select_best_arbitrator(
    env: &Env,
    candidates: &Vec<Address>,
) -> Result<Address, ContractError> {
    let mut best: Option<Address> = None;
    let mut best_score: u32 = 0;
    for i in 0..candidates.len() {
        let c = candidates.get(i).unwrap();
        if !has_arbitrator(env, &c) {
            continue;
        }
        let score = composite_score(&get_arbitrator_reputation(env, &c));
        if best.is_none() || score > best_score {
            best_score = score;
            best = Some(c);
        }
    }
    best.ok_or(ContractError::ArbitratorNotRegistered)
}

/// Return reputation records for all addresses in `arbitrators` (same order).
pub fn get_reputations(env: &Env, arbitrators: &Vec<Address>) -> Vec<ArbitratorReputation> {
    let mut out = Vec::new(env);
    for i in 0..arbitrators.len() {
        out.push_back(get_arbitrator_reputation(env, &arbitrators.get(i).unwrap()));
    }
    out
}
