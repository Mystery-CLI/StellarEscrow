use soroban_sdk::{token, Address, Env, Vec};

use crate::errors::ContractError;
use crate::events;
use crate::storage::{
    get_accumulated_fees, get_fee_bps, get_gov_token, get_proposal, get_proposal_counter,
    get_usdc_token, has_voted, increment_proposal_counter, mark_voted, remove_delegate,
    save_proposal, set_accumulated_fees, set_delegate, set_fee_bps, get_delegate, set_gov_token,
};
use crate::types::{
    Proposal, ProposalAction, ProposalStatus, TierConfig, GOV_PROPOSAL_THRESHOLD,
    GOV_QUORUM_BPS, GOV_TOTAL_SUPPLY, GOV_VOTING_PERIOD,
};

/// Returns the effective voting power of `voter`, following one level of delegation.
fn voting_power(env: &Env, voter: &Address) -> i128 {
    let gov = match get_gov_token(env) {
        Some(t) => t,
        None => return 0,
    };
    // If voter has delegated their votes, they cannot vote themselves
    if get_delegate(env, voter).is_some() {
        return 0;
    }
    token::Client::new(env, &gov).balance(voter)
}

/// Create a new governance proposal.
pub fn create_proposal(
    env: &Env,
    proposer: &Address,
    action: ProposalAction,
) -> Result<u64, ContractError> {
    let power = voting_power(env, proposer);
    if power < GOV_PROPOSAL_THRESHOLD {
        return Err(ContractError::InsufficientVotingPower);
    }
    let id = increment_proposal_counter(env)?;
    let now = env.ledger().sequence();
    let proposal = Proposal {
        id,
        proposer: proposer.clone(),
        action,
        votes_for: 0,
        votes_against: 0,
        status: ProposalStatus::Active,
        created_at: now,
        ends_at: now.checked_add(GOV_VOTING_PERIOD).ok_or(ContractError::Overflow)?,
    };
    save_proposal(env, id, &proposal);
    events::emit_proposal_created(env, id, proposer.clone());
    Ok(id)
}

/// Cast a vote on an active proposal.
pub fn cast_vote(
    env: &Env,
    voter: &Address,
    proposal_id: u64,
    support: bool,
) -> Result<(), ContractError> {
    let mut proposal = get_proposal(env, proposal_id)?;
    if proposal.status != ProposalStatus::Active {
        return Err(ContractError::ProposalNotActive);
    }
    if env.ledger().sequence() > proposal.ends_at {
        return Err(ContractError::VotingEnded);
    }
    if has_voted(env, proposal_id, voter) {
        return Err(ContractError::AlreadyVoted);
    }
    let weight = voting_power(env, voter);
    if weight == 0 {
        return Err(ContractError::InsufficientVotingPower);
    }
    mark_voted(env, proposal_id, voter);
    if support {
        proposal.votes_for = proposal.votes_for.checked_add(weight).ok_or(ContractError::Overflow)?;
    } else {
        proposal.votes_against = proposal.votes_against.checked_add(weight).ok_or(ContractError::Overflow)?;
    }
    save_proposal(env, proposal_id, &proposal);
    events::emit_vote_cast(env, proposal_id, voter.clone(), support, weight);
    Ok(())
}

/// Finalize and execute a passed proposal.
pub fn execute_proposal(env: &Env, proposal_id: u64) -> Result<(), ContractError> {
    let mut proposal = get_proposal(env, proposal_id)?;
    if proposal.status == ProposalStatus::Executed {
        return Err(ContractError::ProposalAlreadyExecuted);
    }
    if proposal.status == ProposalStatus::Active && env.ledger().sequence() <= proposal.ends_at {
        return Err(ContractError::ProposalNotActive);
    }

    // Determine outcome
    let quorum = GOV_TOTAL_SUPPLY
        .checked_mul(GOV_QUORUM_BPS as i128)
        .ok_or(ContractError::Overflow)?
        / 10000;
    let total_votes = proposal.votes_for.checked_add(proposal.votes_against).ok_or(ContractError::Overflow)?;

    if total_votes < quorum || proposal.votes_for <= proposal.votes_against {
        proposal.status = ProposalStatus::Rejected;
        save_proposal(env, proposal_id, &proposal);
        return Err(ContractError::ProposalNotPassed);
    }

    // Execute action
    match proposal.action.clone() {
        ProposalAction::UpdateFeeBps(bps) => {
            if bps > 10000 {
                return Err(ContractError::InvalidFeeBps);
            }
            set_fee_bps(env, bps);
            events::emit_fee_updated(env, bps);
        }
        ProposalAction::UpdateTierConfig(config) => {
            crate::tiers::set_tier_config(env, &config)?;
        }
        ProposalAction::DistributeFees(recipient) => {
            let fees = get_accumulated_fees(env)?;
            if fees == 0 {
                return Err(ContractError::NoFeesToWithdraw);
            }
            let token = get_usdc_token(env)?;
            token::Client::new(env, &token).transfer(
                &env.current_contract_address(),
                &recipient,
                &(fees as i128),
            );
            set_accumulated_fees(env, 0);
            events::emit_fees_distributed(env, recipient, fees);
        }
    }

    proposal.status = ProposalStatus::Executed;
    save_proposal(env, proposal_id, &proposal);
    events::emit_proposal_executed(env, proposal_id);
    Ok(())
}

/// Delegate voting power to another address.
pub fn delegate(env: &Env, delegator: &Address, delegatee: &Address) {
    set_delegate(env, delegator, delegatee);
    events::emit_delegated(env, delegator.clone(), delegatee.clone());
}

/// Remove delegation, reclaiming own voting power.
pub fn undelegate(env: &Env, delegator: &Address) {
    remove_delegate(env, delegator);
}

/// Get proposal by ID.
pub fn get(env: &Env, proposal_id: u64) -> Result<Proposal, ContractError> {
    get_proposal(env, proposal_id)
}

/// Get current proposal count.
pub fn proposal_count(env: &Env) -> u64 {
    get_proposal_counter(env)
}

/// Initialize governance token (admin only, called once during setup).
/// Mints `GOV_TOTAL_SUPPLY` tokens to the specified initial holder.
pub fn initialize_gov_token(
    env: &Env,
    gov_token: &Address,
    initial_holder: &Address,
) -> Result<(), ContractError> {
    if get_gov_token(env).is_some() {
        return Err(ContractError::AlreadyInitialized);
    }
    set_gov_token(env, gov_token);
    
    // Mint initial supply to the holder
    let client = token::Client::new(env, gov_token);
    client.mint(initial_holder, &GOV_TOTAL_SUPPLY);
    
    events::emit_gov_token_initialized(env, gov_token.clone(), initial_holder.clone(), GOV_TOTAL_SUPPLY);
    Ok(())
}

/// Get the governance token address.
pub fn get_gov_token_address(env: &Env) -> Result<Address, ContractError> {
    get_gov_token(env).ok_or(ContractError::NotInitialized)
}

/// Get voting power of an address (accounting for delegation).
pub fn get_voting_power(env: &Env, voter: &Address) -> i128 {
    voting_power(env, voter)
}

/// Get total voting power (total supply of governance tokens).
pub fn get_total_voting_power(_env: &Env) -> i128 {
    GOV_TOTAL_SUPPLY
}

/// Get quorum requirement in tokens.
pub fn get_quorum_requirement(_env: &Env) -> i128 {
    GOV_TOTAL_SUPPLY
        .checked_mul(GOV_QUORUM_BPS as i128)
        .unwrap_or(0)
        / 10000
}

/// Get proposal threshold in tokens.
pub fn get_proposal_threshold(_env: &Env) -> i128 {
    GOV_PROPOSAL_THRESHOLD
}

/// Get voting period in ledgers.
pub fn get_voting_period(_env: &Env) -> u32 {
    GOV_VOTING_PERIOD
}

/// Distribute fees to governance token holders via a proposal.
/// This is called after a proposal to distribute fees is executed.
pub fn distribute_fees_to_holders(
    env: &Env,
    recipient: &Address,
) -> Result<(), ContractError> {
    let fees = get_accumulated_fees(env)?;
    if fees == 0 {
        return Err(ContractError::NoFeesToWithdraw);
    }
    let token = get_usdc_token(env)?;
    token::Client::new(env, &token).transfer(
        &env.current_contract_address(),
        recipient,
        &(fees as i128),
    );
    set_accumulated_fees(env, 0);
    events::emit_fees_distributed(env, recipient.clone(), fees);
    Ok(())
}

/// Get proposal details by ID.
pub fn get_proposal_details(env: &Env, proposal_id: u64) -> Result<Proposal, ContractError> {
    get_proposal(env, proposal_id)
}

/// Check if a proposal has passed (voting ended and quorum + majority met).
pub fn has_proposal_passed(env: &Env, proposal_id: u64) -> Result<bool, ContractError> {
    let proposal = get_proposal(env, proposal_id)?;
    
    // Voting must have ended
    if env.ledger().sequence() <= proposal.ends_at {
        return Ok(false);
    }
    
    // Check quorum
    let quorum = GOV_TOTAL_SUPPLY
        .checked_mul(GOV_QUORUM_BPS as i128)
        .ok_or(ContractError::Overflow)?
        / 10000;
    let total_votes = proposal.votes_for.checked_add(proposal.votes_against).ok_or(ContractError::Overflow)?;
    
    if total_votes < quorum {
        return Ok(false);
    }
    
    // Check majority (votes_for > votes_against)
    Ok(proposal.votes_for > proposal.votes_against)
}

/// Get voting summary for a proposal.
pub fn get_voting_summary(env: &Env, proposal_id: u64) -> Result<(i128, i128, bool), ContractError> {
    let proposal = get_proposal(env, proposal_id)?;
    let voting_ended = env.ledger().sequence() > proposal.ends_at;
    Ok((proposal.votes_for, proposal.votes_against, voting_ended))
}
