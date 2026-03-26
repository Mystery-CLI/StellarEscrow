//! Trade creation form: validation, preview, and confirmation.
//!
//! Flow:
//!   1. Caller populates a [`TradeFormInput`].
//!   2. Call [`validate_input`] — returns [`ContractError`] on any problem.
//!   3. Call [`build_preview`] — returns a [`TradePreview`] for the review step.
//!   4. Caller shows the preview to the user.
//!   5. Call [`confirm_trade`] with the original input *and* the preview —
//!      re-validates everything and only proceeds when they match exactly.

use soroban_sdk::Env;

use crate::errors::ContractError;
use crate::storage::{get_fee_bps, has_arbitrator};
use crate::tiers::effective_fee_bps;
use crate::types::{TradeFormInput, TradePreview};

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate all fields of a [`TradeFormInput`].
///
/// Checks (in order):
/// - seller != buyer
/// - amount > 0
/// - amount does not overflow fee arithmetic
/// - if an arbitrator is provided, it must be registered on-chain
pub fn validate_input(env: &Env, input: &TradeFormInput) -> Result<(), ContractError> {
    if input.seller == input.buyer {
        return Err(ContractError::Unauthorized); // buyer == seller not allowed
    }
    if input.amount == 0 {
        return Err(ContractError::InvalidAmount);
    }
    if input.amount.checked_mul(10_000).is_none() {
        return Err(ContractError::Overflow);
    }
    if let Some(ref arb) = input.arbitrator {
        if !has_arbitrator(env, arb) {
            return Err(ContractError::ArbitratorNotRegistered);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Preview
// ---------------------------------------------------------------------------

/// Build a [`TradePreview`] from a validated input.
///
/// Callers should call [`validate_input`] first; this function does not
/// re-run full validation but does require the contract to be initialized
/// (needs fee_bps from storage).
pub fn build_preview(env: &Env, input: &TradeFormInput) -> Result<TradePreview, ContractError> {
    let base_fee_bps = get_fee_bps(env)?;
    let fee_bps = effective_fee_bps(env, &input.seller, base_fee_bps);
    let estimated_fee = input
        .amount
        .checked_mul(fee_bps as u64)
        .ok_or(ContractError::Overflow)?
        .checked_div(10_000)
        .ok_or(ContractError::Overflow)?;

    Ok(TradePreview {
        seller: input.seller.clone(),
        buyer: input.buyer.clone(),
        amount: input.amount,
        currency: input.currency.clone(),
        arbitrator: input.arbitrator.clone(),
        estimated_fee,
    })
}

// ---------------------------------------------------------------------------
// Confirmation
// ---------------------------------------------------------------------------

/// Confirm a trade after the user has reviewed the preview.
///
/// Re-validates the input, checks that the preview matches the current form
/// state, then delegates to the core trade creation logic.
///
/// Returns the new trade ID on success.
pub fn confirm_trade(
    env: &Env,
    input: &TradeFormInput,
    preview: &TradePreview,
) -> Result<u64, ContractError> {
    validate_input(env, input)?;

    if !preview_matches(input, preview) {
        return Err(ContractError::InvalidAmount);
    }

    crate::lib_create_trade(
        env,
        input.seller.clone(),
        input.buyer.clone(),
        input.amount,
        input.arbitrator.clone(),
        None,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn preview_matches(input: &TradeFormInput, preview: &TradePreview) -> bool {
    input.seller == preview.seller
        && input.buyer == preview.buyer
        && input.amount == preview.amount
        && input.currency == preview.currency
        && input.arbitrator == preview.arbitrator
}
