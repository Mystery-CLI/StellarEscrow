/// Multi-signature arbitration for high-value trades.
///
/// Flow:
///   1. Seller calls `create_multisig_trade` with a `MultiSigConfig`
///      (arbitrators list, threshold, voting_timeout_seconds).
///   2. Buyer or seller raises a dispute — voting window opens automatically.
///   3. Each panel arbitrator calls `cast_vote` with their preferred resolution.
///   4. Once `threshold` arbitrators agree on the same outcome, consensus is
///      reached and `resolve_dispute` executes it.
///   5. If the window expires without consensus, admin calls
///      `resolve_expired_dispute` — defaults to refunding the buyer.

use soroban_sdk::{Address, Env, Vec};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    get_all_votes_for_trade, get_trade, has_arbitrator, has_arbitrator_voted,
    save_arbitrator_vote,
};
use crate::types::{
    ArbitrationConfig, ArbitratorVote, DisputeResolution, MultiSigConfig, Trade,
    TradeStatus, VotingSummary,
};

// ---------------------------------------------------------------------------
// Cast vote
// ---------------------------------------------------------------------------

pub fn cast_vote(
    env: &Env,
    trade_id: u64,
    arbitrator: &Address,
    resolution: DisputeResolution,
) -> Result<(), ContractError> {
    let trade = get_trade(env, trade_id)?;
    if trade.status != TradeStatus::Disputed {
        return Err(ContractError::InvalidStatus);
    }
    let config = extract_multisig_config(&trade)?;

    if !is_panel_member(arbitrator, &config.arbitrators) {
        return Err(ContractError::Unauthorized);
    }
    if !has_arbitrator(env, arbitrator) {
        return Err(ContractError::ArbitratorNotRegistered);
    }
    if has_arbitrator_voted(env, trade_id, arbitrator) {
        return Err(ContractError::AlreadyVoted);
    }
    if is_voting_expired(env, &config) {
        return Err(ContractError::VotingExpired);
    }

    arbitrator.require_auth();

    save_arbitrator_vote(
        env,
        trade_id,
        arbitrator,
        &ArbitratorVote {
            arbitrator: arbitrator.clone(),
            resolution: resolution.clone(),
            timestamp: env.ledger().timestamp(),
        },
    );

    events::emit_arbitrator_vote_cast(env, trade_id, arbitrator.clone(), resolution);
    Ok(())
}

// ---------------------------------------------------------------------------
// Voting summary / consensus
// ---------------------------------------------------------------------------

pub fn voting_summary(env: &Env, trade_id: u64) -> Result<VotingSummary, ContractError> {
    let trade = get_trade(env, trade_id)?;
    let config = extract_multisig_config(&trade)?;

    let votes = get_all_votes_for_trade(env, trade_id, &config.arbitrators);
    let votes_cast = votes.len() as u32;
    let total_arbitrators = config.arbitrators.len() as u32;
    let expired = is_voting_expired(env, &config);
    let threshold = config.threshold;

    // Tally votes per resolution variant
    let mut release_to_buyer: u32 = 0;
    let mut release_to_seller: u32 = 0;
    // Partial votes: Vec of (buyer_bps, count)
    let mut partial: Vec<(u32, u32)> = Vec::new(env);

    for i in 0..votes.len() {
        match votes.get(i).unwrap().resolution {
            DisputeResolution::ReleaseToBuyer => release_to_buyer += 1,
            DisputeResolution::ReleaseToSeller => release_to_seller += 1,
            DisputeResolution::Partial { buyer_bps } => {
                let mut found = false;
                for j in 0..partial.len() {
                    let (bps, cnt) = partial.get(j).unwrap();
                    if bps == buyer_bps {
                        partial.set(j, (bps, cnt + 1));
                        found = true;
                        break;
                    }
                }
                if !found {
                    partial.push_back((buyer_bps, 1));
                }
            }
        }
    }

    let consensus: Option<DisputeResolution> = if release_to_buyer >= threshold {
        Some(DisputeResolution::ReleaseToBuyer)
    } else if release_to_seller >= threshold {
        Some(DisputeResolution::ReleaseToSeller)
    } else {
        let mut found: Option<DisputeResolution> = None;
        for i in 0..partial.len() {
            let (bps, cnt) = partial.get(i).unwrap();
            if cnt >= threshold {
                found = Some(DisputeResolution::Partial { buyer_bps: bps });
                break;
            }
        }
        found
    };

    if consensus.is_some() {
        events::emit_multisig_consensus(env, trade_id);
    }

    Ok(VotingSummary {
        total_arbitrators,
        votes_cast,
        threshold,
        has_consensus: consensus.is_some(),
        consensus_resolution: consensus,
        voting_expired: expired,
    })
}

// ---------------------------------------------------------------------------
// Expired-dispute resolution
// ---------------------------------------------------------------------------

/// Force-resolve after the voting window expires without consensus.
/// Safe default: refund the buyer. Admin-only.
pub fn resolve_expired_dispute(
    env: &Env,
    trade_id: u64,
    admin: &Address,
) -> Result<DisputeResolution, ContractError> {
    let trade = get_trade(env, trade_id)?;
    if trade.status != TradeStatus::Disputed {
        return Err(ContractError::InvalidStatus);
    }
    let config = extract_multisig_config(&trade)?;
    if !is_voting_expired(env, &config) {
        return Err(ContractError::VotingNotExpired);
    }
    admin.require_auth();
    events::emit_multisig_expired(env, trade_id);
    Ok(DisputeResolution::ReleaseToBuyer)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_multisig_config(trade: &Trade) -> Result<MultiSigConfig, ContractError> {
    match &trade.arbitrator {
        Some(ArbitrationConfig::MultiSig(cfg)) => Ok(cfg.clone()),
        _ => Err(ContractError::InvalidStatus),
    }
}

fn is_panel_member(arbitrator: &Address, panel: &Vec<Address>) -> bool {
    for i in 0..panel.len() {
        if panel.get(i).unwrap() == *arbitrator {
            return true;
        }
    }
    false
}

fn is_voting_expired(env: &Env, config: &MultiSigConfig) -> bool {
    match config.voting_started_at {
        Some(started) => {
            env.ledger().timestamp() > started.saturating_add(config.voting_timeout_seconds)
        }
        None => false,
    }
}
