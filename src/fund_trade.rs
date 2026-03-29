//! Buyer funding flow: allowance check, preview, and confirmed fund execution.
//!
//! Flow:
//!   1. Call [`get_funding_preview`] — returns a [`FundingPreview`] with trade
//!      details, the USDC amount required, and whether the buyer's current
//!      allowance is sufficient.
//!   2. If `allowance_sufficient` is false the caller must first submit a USDC
//!      `approve` transaction granting the escrow contract at least `amount`.
//!   3. Call [`execute_fund`] with the original `trade_id` and the preview —
//!      re-validates state, checks the preview matches, then transfers USDC and
//!      marks the trade Funded.

use soroban_sdk::{token::TokenClient, Address, Env};

use crate::errors::ContractError;
use crate::storage::{get_trade, get_usdc_token, is_initialized, is_paused, save_trade,
                     append_timeline_entry};
use crate::types::{FundingPreview, TimelineEntry, Trade, TradeStatus};

// ---------------------------------------------------------------------------
// Preview
// ---------------------------------------------------------------------------

/// Build a [`FundingPreview`] for the given trade and buyer.
///
/// Returns `ContractError::TradeNotFound` if the trade does not exist,
/// `ContractError::InvalidStatus` if it is not in `Created` state, and
/// `ContractError::Unauthorized` if `buyer` is not the trade's buyer.
pub fn get_funding_preview(
    env: &Env,
    trade_id: u64,
    buyer: &Address,
) -> Result<FundingPreview, ContractError> {
    if !is_initialized(env) {
        return Err(ContractError::NotInitialized);
    }
    let trade = get_trade(env, trade_id)?;
    validate_fundable(&trade, buyer)?;

    let token = get_usdc_token(env)?;
    let token_client = TokenClient::new(env, &token);

    let buyer_balance = token_client.balance(buyer) as u64;
    let current_allowance = token_client.allowance(buyer, &env.current_contract_address()) as u64;
    let allowance_sufficient = current_allowance >= trade.amount;

    Ok(FundingPreview {
        trade_id,
        buyer: buyer.clone(),
        seller: trade.seller.clone(),
        amount: trade.amount,
        fee: trade.fee,
        buyer_balance,
        allowance_sufficient,
    })
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

/// Fund a trade after the buyer has reviewed the preview.
///
/// Re-validates trade state, checks the preview matches current on-chain data,
/// then transfers `trade.amount` USDC from `buyer` to the escrow contract and
/// advances the trade to `Funded`.
///
/// The buyer must have authorised this call (`buyer.require_auth()`).
pub fn execute_fund(
    env: &Env,
    trade_id: u64,
    buyer: &Address,
    preview: &FundingPreview,
) -> Result<(), ContractError> {
    if !is_initialized(env) {
        return Err(ContractError::NotInitialized);
    }
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }

    let mut trade = get_trade(env, trade_id)?;
    validate_fundable(&trade, buyer)?;

    // Ensure the preview the user confirmed matches current on-chain state.
    if !preview_matches_trade(preview, &trade) {
        return Err(ContractError::InvalidAmount);
    }

    // Prevent duplicate submission: re-check allowance on-chain.
    let token = get_usdc_token(env)?;
    let token_client = TokenClient::new(env, &token);

    let allowance = token_client.allowance(buyer, &env.current_contract_address());
    if (allowance as u64) < trade.amount {
        return Err(ContractError::InsufficientAllowance);
    }

    // Transfer USDC from buyer to escrow.
    token_client.transfer(buyer, &env.current_contract_address(), &(trade.amount as i128));

    // Advance trade state.
    trade.status = TradeStatus::Funded;
    trade.updated_at = env.ledger().sequence();
    save_trade(env, trade_id, &trade);
    append_timeline_entry(
        env,
        trade_id,
        TimelineEntry { status: TradeStatus::Funded, ledger: trade.updated_at },
    );

    crate::events::emit_trade_funded(env, trade_id);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate_fundable(trade: &Trade, buyer: &Address) -> Result<(), ContractError> {
    if trade.status != TradeStatus::Created {
        return Err(ContractError::InvalidStatus);
    }
    if &trade.buyer != buyer {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

fn preview_matches_trade(preview: &FundingPreview, trade: &Trade) -> bool {
    preview.trade_id == trade.id
        && preview.buyer == trade.buyer
        && preview.seller == trade.seller
        && preview.amount == trade.amount
        && preview.fee == trade.fee
}
