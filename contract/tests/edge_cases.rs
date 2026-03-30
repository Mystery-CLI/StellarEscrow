#![cfg(test)]

mod common;

use common::{
    advance_ledger_sequence, approve_funding, create_completed_trade, create_disputed_trade,
    create_trade, setup,
};
use soroban_sdk::{testutils::Address as _, String};
use stellar_escrow_contract::{DisputeResolution, OptionalMetadata, TradeStatus};

#[test]
fn edge_rejects_duplicate_funding_attempts() {
    let h = setup();
    let id = create_trade(&h, 1_000_000);

    approve_funding(&h, 1_000_000);
    h.client.fund_trade(&id);

    assert!(h.client.try_fund_trade(&id).is_err());
    assert_eq!(h.client.get_trade(&id).status, TradeStatus::Funded);
}

#[test]
fn edge_rejects_receipt_confirmation_before_completion() {
    let h = setup();
    let id = create_trade(&h, 1_000_000);

    approve_funding(&h, 1_000_000);
    h.client.fund_trade(&id);

    assert!(h.client.try_confirm_receipt(&id).is_err());
}

#[test]
fn edge_rejects_partial_resolution_above_100_percent() {
    let h = setup();
    let id = create_disputed_trade(&h, 1_000_000);

    assert!(
        h.client
            .try_resolve_dispute(&id, &DisputeResolution::Partial(10_001))
            .is_err()
    );
}

#[test]
fn edge_rejects_expiring_bridge_trade_before_deadline() {
    let h = setup();
    let oracle = soroban_sdk::Address::generate(&h.env);
    h.client.set_bridge_oracle(&oracle);
    let chain = String::from_str(&h.env, "ethereum");
    let id = h.client.create_cross_chain_trade(
        &h.seller,
        &h.buyer,
        &1_000_000u64,
        &None,
        &chain,
        &20u32,
    );

    assert!(h.client.try_expire_bridge_trade(&id).is_err());
}

#[test]
fn edge_expires_bridge_trade_once_deadline_passes() {
    let h = setup();
    let oracle = soroban_sdk::Address::generate(&h.env);
    h.client.set_bridge_oracle(&oracle);
    let chain = String::from_str(&h.env, "polygon");
    let id = h.client.create_cross_chain_trade(
        &h.seller,
        &h.buyer,
        &1_000_000u64,
        &None,
        &chain,
        &5u32,
    );

    advance_ledger_sequence(&h, 6);
    h.client.expire_bridge_trade(&id);

    assert_eq!(h.client.get_trade(&id).status, TradeStatus::Cancelled);
}

#[test]
fn edge_rejects_insurance_purchase_for_unfunded_trade() {
    let h = setup();
    let provider = soroban_sdk::Address::generate(&h.env);
    h.client.register_insurance_provider(&provider);

    let id = h.client.create_trade(
        &h.seller,
        &h.buyer,
        &1_000_000u64,
        &None,
        &OptionalMetadata::None,
    );

    assert!(
        h.client
            .try_purchase_insurance(&id, &provider, &100u32, &500_000u64)
            .is_err()
    );
}

#[test]
fn edge_rejects_cancelling_completed_trade() {
    let h = setup();
    let id = create_completed_trade(&h, 1_000_000);

    assert!(h.client.try_cancel_trade(&id).is_err());
}
