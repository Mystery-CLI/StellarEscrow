#![cfg(test)]

mod common;

use common::{approve_funding, create_trade, setup};
use soroban_sdk::{testutils::Address as _, token, Address, String};
use stellar_escrow_contract::{DisputeResolution, OptionalMetadata, TradeStatus};

#[test]
fn security_blocks_state_changes_while_paused() {
    let h = setup();
    let id = create_trade(&h, 1_000_000);

    h.client.pause();

    assert!(h.client.try_cancel_trade(&id).is_err());
    assert!(h.client.try_complete_trade(&id).is_err());
    assert!(
        h.client
            .try_create_trade(&h.seller, &h.buyer, &1_000_000u64, &None, &OptionalMetadata::None)
            .is_err()
    );
}

#[test]
fn security_rejects_bridge_trade_creation_without_oracle() {
    let h = setup();
    let fresh_env = soroban_sdk::Env::default();
    fresh_env.mock_all_auths();
    let admin = Address::generate(&fresh_env);
    let seller = Address::generate(&fresh_env);
    let buyer = Address::generate(&fresh_env);
    let sac = fresh_env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();
    let contract_id = fresh_env.register_contract(None, stellar_escrow_contract::StellarEscrowContract);
    let client = stellar_escrow_contract::StellarEscrowContractClient::new(&fresh_env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32);

    let compliant = stellar_escrow_contract::types::UserCompliance {
        kyc_status: stellar_escrow_contract::types::KycStatus::Verified,
        aml_cleared: true,
        jurisdiction: String::from_str(&fresh_env, "US"),
    };
    client.set_user_compliance(&admin, &seller, &compliant);
    client.set_user_compliance(&admin, &buyer, &compliant);

    assert!(
        client
            .try_create_cross_chain_trade(
                &seller,
                &buyer,
                &1_000_000u64,
                &None,
                &String::from_str(&fresh_env, "ethereum"),
                &12u32,
            )
            .is_err()
    );

    let _ = h;
}

#[test]
fn security_only_registered_arbitrators_can_back_disputed_trades() {
    let h = setup();
    let rogue_arbitrator = Address::generate(&h.env);

    assert!(
        h.client
            .try_create_trade(
                &h.seller,
                &h.buyer,
                &1_000_000u64,
                &Some(rogue_arbitrator),
                &OptionalMetadata::None,
            )
            .is_err()
    );
}

#[test]
fn security_rejects_dispute_resolution_without_dispute_state() {
    let h = setup();
    let id = create_trade(&h, 1_000_000);

    assert!(
        h.client
            .try_resolve_dispute(&id, &DisputeResolution::ReleaseToBuyer)
            .is_err()
    );
}

#[test]
fn security_rejects_fee_withdrawal_when_pool_is_empty() {
    let h = setup();
    let recipient = Address::generate(&h.env);

    assert!(h.client.try_withdraw_fees(&recipient).is_err());
}

#[test]
fn security_prevents_double_bridge_confirmation() {
    let h = setup();
    let oracle = Address::generate(&h.env);
    h.client.set_bridge_oracle(&oracle);

    let id = h.client.create_cross_chain_trade(
        &h.seller,
        &h.buyer,
        &1_000_000u64,
        &None,
        &String::from_str(&h.env, "base"),
        &50u32,
    );

    h.client
        .confirm_bridge_deposit(&id, &String::from_str(&h.env, "0xabc123"));

    assert!(
        h.client
            .try_confirm_bridge_deposit(&id, &String::from_str(&h.env, "0xdef456"))
            .is_err()
    );
    assert_eq!(h.client.get_trade(&id).status, TradeStatus::Funded);
}

#[test]
fn security_requires_insurance_provider_registration() {
    let h = setup();
    let provider = Address::generate(&h.env);
    let id = create_trade(&h, 1_000_000);
    approve_funding(&h, 1_000_000);
    h.client.fund_trade(&id);

    assert!(
        h.client
            .try_purchase_insurance(&id, &provider, &100u32, &500_000u64)
            .is_err()
    );

    let _buyer_balance = token::Client::new(&h.env, &h.token_addr).balance(&h.buyer);
}
