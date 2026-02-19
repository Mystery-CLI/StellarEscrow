#![cfg(test)]

use crate::{DisputeResolution, StellarEscrowContract, StellarEscrowContractClient, TradeStatus};
use soroban_sdk::{
    symbol_short, testutils::{Address as _, Events}, token, Address, Env, IntoVal, String
};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::Client<'a> {
    let token = token::StellarAssetClient::new(env, &env.register_stellar_asset_contract_v2(admin.clone()).address());
    token
}

fn create_escrow_contract<'a>(env: &Env) -> StellarEscrowContractClient<'a> {
    StellarEscrowContractClient::new(env, &env.register(StellarEscrowContract, ()))
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    contract.initialize(&admin, &token.address, &250);

    assert_eq!(contract.get_platform_fee_bps(), 250);
    assert_eq!(contract.get_accumulated_fees(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    contract.initialize(&admin, &token.address, &250);
    contract.initialize(&admin, &token.address, &250);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_initialize_invalid_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    contract.initialize(&admin, &token.address, &10001);
}

#[test]
fn test_register_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);
    let arbitrator = Address::generate(&env);

    contract.initialize(&admin, &token.address, &250);
    contract.register_arbitrator(&arbitrator);

    assert!(contract.is_arbitrator_registered(&arbitrator));
}

#[test]
fn test_remove_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);
    let arbitrator = Address::generate(&env);

    contract.initialize(&admin, &token.address, &250);
    contract.register_arbitrator(&arbitrator);
    assert!(contract.is_arbitrator_registered(&arbitrator));

    contract.remove_arbitrator_fn(&arbitrator);
    assert!(!contract.is_arbitrator_registered(&arbitrator));
}

#[test]
fn test_update_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    contract.initialize(&admin, &token.address, &250);
    assert_eq!(contract.get_platform_fee_bps(), 250);

    contract.update_fee(&500);
    assert_eq!(contract.get_platform_fee_bps(), 500);
}

#[test]
fn test_create_trade() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    contract.initialize(&admin, &token.address, &250);

    let trade_id = contract.create_trade(&seller, &buyer, &10000, &None);
    assert_eq!(trade_id, 1);

    let trade = contract.get_trade(&trade_id);
    assert_eq!(trade.seller, seller);
    assert_eq!(trade.buyer, buyer);
    assert_eq!(trade.amount, 10000);
    assert_eq!(trade.fee, 250);
    assert_eq!(trade.status, TradeStatus::Created);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_create_trade_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    contract.initialize(&admin, &token.address, &250);
    contract.create_trade(&seller, &buyer, &0, &None);
}

#[test]
fn test_fund_trade() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    token.mint(&buyer, &10000);

    contract.initialize(&admin, &token.address, &250);
    let trade_id = contract.create_trade(&seller, &buyer, &10000, &None);

    contract.fund_trade(&trade_id);

    let trade = contract.get_trade(&trade_id);
    assert_eq!(trade.status, TradeStatus::Funded);
    assert_eq!(token.balance(&contract.address), 10000);
}

#[test]
fn test_complete_trade_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    token.mint(&buyer, &10000);

    contract.initialize(&admin, &token.address, &250);
    let trade_id = contract.create_trade(&seller, &buyer, &10000, &None);

    contract.fund_trade(&trade_id);
    contract.complete_trade(&trade_id);

    let trade = contract.get_trade(&trade_id);
    assert_eq!(trade.status, TradeStatus::Completed);

    contract.confirm_receipt(&trade_id);

    assert_eq!(token.balance(&seller), 9750);
    assert_eq!(contract.get_accumulated_fees(), 250);
}

#[test]
fn test_cancel_trade() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    contract.initialize(&admin, &token.address, &250);
    let trade_id = contract.create_trade(&seller, &buyer, &10000, &None);

    contract.cancel_trade(&trade_id);

    let trade = contract.get_trade(&trade_id);
    assert_eq!(trade.status, TradeStatus::Cancelled);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_cancel_funded_trade() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    token.mint(&buyer, &10000);

    contract.initialize(&admin, &token.address, &250);
    let trade_id = contract.create_trade(&seller, &buyer, &10000, &None);

    contract.fund_trade(&trade_id);
    contract.cancel_trade(&trade_id);
}

#[test]
fn test_dispute_resolution_to_buyer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    token.mint(&buyer, &10000);

    contract.initialize(&admin, &token.address, &250);
    contract.register_arbitrator(&arbitrator);

    let trade_id = contract.create_trade(&seller, &buyer, &10000, &Some(arbitrator.clone()));
    contract.fund_trade(&trade_id);
    contract.raise_dispute(&trade_id);

    let trade = contract.get_trade(&trade_id);
    assert_eq!(trade.status, TradeStatus::Disputed);

    contract.resolve_dispute(&trade_id, &DisputeResolution::ReleaseToBuyer);

    assert_eq!(token.balance(&buyer), 9750);
    assert_eq!(contract.get_accumulated_fees(), 250);
}

#[test]
fn test_dispute_resolution_to_seller() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    token.mint(&buyer, &10000);

    contract.initialize(&admin, &token.address, &250);
    contract.register_arbitrator(&arbitrator);

    let trade_id = contract.create_trade(&seller, &buyer, &10000, &Some(arbitrator.clone()));
    contract.fund_trade(&trade_id);
    contract.raise_dispute(&trade_id);

    contract.resolve_dispute(&trade_id, &DisputeResolution::ReleaseToSeller);

    assert_eq!(token.balance(&seller), 9750);
    assert_eq!(contract.get_accumulated_fees(), 250);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_raise_dispute_without_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    token.mint(&buyer, &10000);

    contract.initialize(&admin, &token.address, &250);
    let trade_id = contract.create_trade(&seller, &buyer, &10000, &None);
    contract.fund_trade(&trade_id);

    contract.raise_dispute(&trade_id);
}

#[test]
fn test_withdraw_fees() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let fee_recipient = Address::generate(&env);

    token.mint(&buyer, &10000);

    contract.initialize(&admin, &token.address, &250);
    let trade_id = contract.create_trade(&seller, &buyer, &10000, &None);

    contract.fund_trade(&trade_id);
    contract.complete_trade(&trade_id);
    contract.confirm_receipt(&trade_id);

    assert_eq!(contract.get_accumulated_fees(), 250);

    contract.withdraw_fees(&fee_recipient);

    assert_eq!(token.balance(&fee_recipient), 250);
    assert_eq!(contract.get_accumulated_fees(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_withdraw_fees_when_none() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);
    let fee_recipient = Address::generate(&env);

    contract.initialize(&admin, &token.address, &250);
    contract.withdraw_fees(&fee_recipient);
}

#[test]
fn test_multiple_trades() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller1 = Address::generate(&env);
    let buyer1 = Address::generate(&env);
    let seller2 = Address::generate(&env);
    let buyer2 = Address::generate(&env);

    token.mint(&buyer1, &10000);
    token.mint(&buyer2, &20000);

    contract.initialize(&admin, &token.address, &250);

    let trade_id1 = contract.create_trade(&seller1, &buyer1, &10000, &None);
    let trade_id2 = contract.create_trade(&seller2, &buyer2, &20000, &None);

    assert_eq!(trade_id1, 1);
    assert_eq!(trade_id2, 2);

    contract.fund_trade(&trade_id1);
    contract.fund_trade(&trade_id2);

    contract.complete_trade(&trade_id1);
    contract.complete_trade(&trade_id2);

    contract.confirm_receipt(&trade_id1);
    contract.confirm_receipt(&trade_id2);

    assert_eq!(token.balance(&seller1), 9750);
    assert_eq!(token.balance(&seller2), 19500);
    assert_eq!(contract.get_accumulated_fees(), 750);
}

#[test]
fn test_event_emission() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let contract = create_escrow_contract(&env);

    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    token.mint(&buyer, &10000);

    contract.initialize(&admin, &token.address, &250);
    let trade_id = contract.create_trade(&seller, &buyer, &10000, &None);

    let events = env.events().all();
    let event = events.last().unwrap();

    assert_eq!(
        event.topics,
        (symbol_short!("created"),).into_val(&env)
    );
}
