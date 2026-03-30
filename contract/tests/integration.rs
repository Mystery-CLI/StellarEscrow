#![cfg(test)]

mod common;

use common::{approve_funding, create_trade, setup};
use soroban_sdk::{testutils::Address as _, token, Address, String};
use stellar_escrow_contract::{DisputeResolution, OptionalMetadata, TradeStatus};

#[test]
fn integration_happy_path_accumulates_and_withdraws_fees() {
    let h = setup();
    let id = create_trade(&h, 1_000_000);

    approve_funding(&h, 1_000_000);
    h.client.fund_trade(&id);
    h.client.complete_trade(&id);
    h.client.confirm_receipt(&id);

    let recipient = Address::generate(&h.env);
    h.client.withdraw_fees(&recipient);

    assert_eq!(token::Client::new(&h.env, &h.token_addr).balance(&h.seller), 990_000);
    assert_eq!(token::Client::new(&h.env, &h.token_addr).balance(&recipient), 10_000);
}

#[test]
fn integration_dispute_partial_resolution_preserves_fee_accounting() {
    let h = setup();
    let buyer_before = token::Client::new(&h.env, &h.token_addr).balance(&h.buyer);
    let seller_before = token::Client::new(&h.env, &h.token_addr).balance(&h.seller);
    let id = h.client.create_trade(
        &h.seller,
        &h.buyer,
        &1_000_000u64,
        &Some(h.arbitrator.clone()),
        &OptionalMetadata::None,
    );

    approve_funding(&h, 1_000_000);
    h.client.fund_trade(&id);
    h.client.raise_dispute(&id, &h.buyer);
    h.client
        .resolve_dispute(&id, &DisputeResolution::Partial(4_000));

    assert_eq!(h.client.get_trade(&id).status, TradeStatus::Disputed);
    assert_eq!(h.client.get_accumulated_fees(), 10_000);
    assert_eq!(
        token::Client::new(&h.env, &h.token_addr).balance(&h.buyer) - buyer_before,
        -604_000
    );
    assert_eq!(
        token::Client::new(&h.env, &h.token_addr).balance(&h.seller) - seller_before,
        594_000
    );
}

#[test]
fn integration_bridge_trade_can_complete_standard_settlement() {
    let h = setup();
    let oracle = Address::generate(&h.env);
    h.client.set_bridge_oracle(&oracle);

    let id = h.client.create_cross_chain_trade(
        &h.seller,
        &h.buyer,
        &1_500_000u64,
        &None,
        &String::from_str(&h.env, "ethereum"),
        &25u32,
    );

    h.client
        .confirm_bridge_deposit(&id, &String::from_str(&h.env, "0xbridge"));
    h.client.complete_trade(&id);
    h.client.confirm_receipt(&id);

    assert_eq!(h.client.get_trade(&id).status, TradeStatus::Completed);
    assert_eq!(token::Client::new(&h.env, &h.token_addr).balance(&h.seller), 1_485_000);
}

#[test]
fn integration_insurance_claim_flow_pays_out_provider_coverage() {
    let h = setup();
    let provider = Address::generate(&h.env);
    h.client.register_insurance_provider(&provider);
    token::StellarAssetClient::new(&h.env, &h.token_addr).mint(&provider, &10_000_000i128);

    let id = h.client.create_trade(
        &h.seller,
        &h.buyer,
        &1_000_000u64,
        &Some(h.arbitrator.clone()),
        &OptionalMetadata::None,
    );

    approve_funding(&h, 1_000_000);
    h.client.fund_trade(&id);
    h.client.purchase_insurance(&id, &provider, &100u32, &250_000u64);
    h.client.raise_dispute(&id, &h.buyer);
    h.client.claim_insurance(&id, &h.seller, &200_000u64);

    let policy = h.client.get_insurance_policy(&id).unwrap();
    assert!(policy.claimed);
    assert_eq!(token::Client::new(&h.env, &h.token_addr).balance(&h.seller), 200_000);
}
