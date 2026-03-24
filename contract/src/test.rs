#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, token, Address, Env};

use crate::{OptionalMetadata, StellarEscrowContract, StellarEscrowContractClient, TradeStatus};

fn setup() -> (Env, Address, Address, Address, Address, Address, StellarEscrowContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();

    // Mint to buyer
    token::StellarAssetClient::new(&env, &token_addr).mint(&buyer, &1_000_000_000i128);

    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token_addr, &100u32); // 1% fee

    (env, token_addr, admin, seller, buyer, arbitrator, client)
}

fn fund(env: &Env, token_addr: &Address, buyer: &Address, contract: &Address, amount: i128) {
    token::Client::new(env, token_addr).approve(buyer, contract, &amount, &200u32);
}

#[test]
fn test_initialize_and_fee() {
    let (_, _, _, _, _, _, client) = setup();
    assert_eq!(client.get_platform_fee_bps(), 100);
}

#[test]
fn test_initialize_twice_fails() {
    let (env, token_addr, admin, _, _, _, client) = setup();
    assert!(client.try_initialize(&admin, &token_addr, &100u32).is_err());
}

#[test]
fn test_register_and_remove_arbitrator() {
    let (_, _, _, _, _, arbitrator, client) = setup();
    client.register_arbitrator(&arbitrator);
    assert!(client.is_arbitrator_registered(&arbitrator));
    client.remove_arbitrator_fn(&arbitrator);
    assert!(!client.is_arbitrator_registered(&arbitrator));
}

#[test]
fn test_update_fee() {
    let (_, _, _, _, _, _, client) = setup();
    client.update_fee(&200u32);
    assert_eq!(client.get_platform_fee_bps(), 200);
}

#[test]
fn test_update_fee_invalid() {
    let (_, _, _, _, _, _, client) = setup();
    assert!(client.try_update_fee(&10001u32).is_err());
}

#[test]
fn test_create_trade() {
    let (_, _, _, seller, buyer, _, client) = setup();
    let id = client.create_trade(&seller, &buyer, &1_000_000u64, &None, &OptionalMetadata::None);
    assert_eq!(id, 1);
    let trade = client.get_trade(&id);
    assert_eq!(trade.status, TradeStatus::Created);
    assert_eq!(trade.amount, 1_000_000u64);
}

#[test]
fn test_create_trade_zero_amount_fails() {
    let (_, _, _, seller, buyer, _, client) = setup();
    assert!(client.try_create_trade(&seller, &buyer, &0u64, &None, &OptionalMetadata::None).is_err());
}

#[test]
fn test_fund_trade() {
    let (env, token_addr, _, seller, buyer, _, client) = setup();
    let id = client.create_trade(&seller, &buyer, &1_000_000u64, &None, &OptionalMetadata::None);
    fund(&env, &token_addr, &buyer, &client.address, 1_000_000);
    client.fund_trade(&id);
    assert_eq!(client.get_trade(&id).status, TradeStatus::Funded);
}

#[test]
fn test_complete_and_confirm_trade() {
    let (env, token_addr, _, seller, buyer, _, client) = setup();
    let amount = 1_000_000u64;
    let id = client.create_trade(&seller, &buyer, &amount, &None, &OptionalMetadata::None);
    fund(&env, &token_addr, &buyer, &client.address, amount as i128);
    client.fund_trade(&id);
    client.complete_trade(&id);
    client.confirm_receipt(&id);

    // fee = 1% of 1_000_000 = 10_000; seller receives 990_000
    assert_eq!(token::Client::new(&env, &token_addr).balance(&seller), 990_000i128);
    assert_eq!(client.get_accumulated_fees(), 10_000u64);
}

#[test]
fn test_cancel_trade() {
    let (_, _, _, seller, buyer, _, client) = setup();
    let id = client.create_trade(&seller, &buyer, &1_000_000u64, &None, &OptionalMetadata::None);
    client.cancel_trade(&id);
    assert_eq!(client.get_trade(&id).status, TradeStatus::Cancelled);
}

#[test]
fn test_dispute_and_resolve_to_buyer() {
    let (env, token_addr, _, seller, buyer, arbitrator, client) = setup();
    client.register_arbitrator(&arbitrator);
    let amount = 1_000_000u64;
    let id = client.create_trade(&seller, &buyer, &amount, &Some(arbitrator.clone()), &OptionalMetadata::None);
    fund(&env, &token_addr, &buyer, &client.address, amount as i128);
    client.fund_trade(&id);
    client.raise_dispute(&id, &buyer);

    let before = token::Client::new(&env, &token_addr).balance(&buyer);
    client.resolve_dispute(&id, &crate::DisputeResolution::ReleaseToBuyer);
    let after = token::Client::new(&env, &token_addr).balance(&buyer);
    assert_eq!(after - before, 990_000i128);
}

#[test]
fn test_withdraw_fees() {
    let (env, token_addr, _, seller, buyer, _, client) = setup();
    let amount = 1_000_000u64;
    let id = client.create_trade(&seller, &buyer, &amount, &None, &OptionalMetadata::None);
    fund(&env, &token_addr, &buyer, &client.address, amount as i128);
    client.fund_trade(&id);
    client.complete_trade(&id);
    client.confirm_receipt(&id);

    let recipient = Address::generate(&env);
    client.withdraw_fees(&recipient);
    assert_eq!(token::Client::new(&env, &token_addr).balance(&recipient), 10_000i128);
    assert_eq!(client.get_accumulated_fees(), 0u64);
}

#[test]
fn test_pause_and_unpause() {
    let (_, _, _, seller, buyer, _, client) = setup();
    client.pause();
    assert!(client.is_paused());
    assert!(client.try_create_trade(&seller, &buyer, &1_000_000u64, &None, &OptionalMetadata::None).is_err());
    client.unpause();
    assert!(!client.is_paused());
}

#[test]
fn test_no_fees_to_withdraw_fails() {
    let (env, _, _, _, _, _, client) = setup();
    let recipient = Address::generate(&env);
    assert!(client.try_withdraw_fees(&recipient).is_err());
}

#[test]
fn test_version_starts_at_1() {
    let (_, _, _, _, _, _, client) = setup();
    assert_eq!(client.version(), 1);
}

#[test]
fn test_migrate_increments_version() {
    let (_, _, _, _, _, _, client) = setup();
    assert_eq!(client.version(), 1);
    client.migrate(&1u32);
    assert_eq!(client.version(), 2);
}

#[test]
fn test_migrate_wrong_version_fails() {
    let (_, _, _, _, _, _, client) = setup();
    // current version is 1, passing 2 should fail
    assert!(client.try_migrate(&2u32).is_err());
}

#[test]
fn test_migrate_double_application_fails() {
    let (_, _, _, _, _, _, client) = setup();
    client.migrate(&1u32); // version -> 2
    // applying again with old expected_version should fail
    assert!(client.try_migrate(&1u32).is_err());
}

// ---------------------------------------------------------------------------
// Cross-chain bridge tests
// ---------------------------------------------------------------------------

fn setup_bridge() -> (Env, Address, Address, Address, Address, StellarEscrowContractClient<'static>) {
    let (env, token_addr, admin, seller, buyer, _, client) = setup();
    let oracle = Address::generate(&env);
    client.set_bridge_oracle(&oracle);
    (env, token_addr, admin, seller, buyer, client)
}

#[test]
fn test_set_bridge_oracle() {
    let (env, _, _, _, _, client) = setup_bridge();
    // oracle is set — creating a cross-chain trade should not fail with BridgeOracleNotSet
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let id = client.create_cross_chain_trade(
        &seller, &buyer, &1_000_000u64, &None,
        &soroban_sdk::String::from_str(&env, "ethereum"), &1000u32,
    );
    assert_eq!(id, 1);
}

#[test]
fn test_create_cross_chain_trade_no_oracle_fails() {
    let (env, _, _, seller, buyer, _, client) = setup();
    assert!(client.try_create_cross_chain_trade(
        &seller, &buyer, &1_000_000u64, &None,
        &soroban_sdk::String::from_str(&env, "ethereum"), &1000u32,
    ).is_err());
}

#[test]
fn test_confirm_bridge_deposit() {
    let (env, _, _, seller, buyer, client) = setup_bridge();
    let oracle = client.get_cross_chain_info(&{
        // create a trade first to get the oracle address indirectly
        let id = client.create_cross_chain_trade(
            &seller, &buyer, &1_000_000u64, &None,
            &soroban_sdk::String::from_str(&env, "ethereum"), &1000u32,
        );
        id
    });
    // trade should be AwaitingBridge
    let _ = oracle; // cross_chain_info returned

    // re-create cleanly
    let id = client.create_cross_chain_trade(
        &seller, &buyer, &1_000_000u64, &None,
        &soroban_sdk::String::from_str(&env, "polygon"), &1000u32,
    );
    assert_eq!(client.get_trade(&id).status, TradeStatus::AwaitingBridge);

    client.confirm_bridge_deposit(&id, &soroban_sdk::String::from_str(&env, "0xabc123"));
    assert_eq!(client.get_trade(&id).status, TradeStatus::Funded);

    let info = client.get_cross_chain_info(&id).unwrap();
    assert_eq!(info.source_chain, soroban_sdk::String::from_str(&env, "polygon"));
}

#[test]
fn test_expire_bridge_trade() {
    let (env, _, _, seller, buyer, client) = setup_bridge();
    let id = client.create_cross_chain_trade(
        &seller, &buyer, &1_000_000u64, &None,
        &soroban_sdk::String::from_str(&env, "ethereum"), &10u32,
    );
    // advance ledger past expiry
    env.ledger().with_mut(|l| l.sequence_number += 11);
    client.expire_bridge_trade(&id);
    assert_eq!(client.get_trade(&id).status, TradeStatus::Cancelled);
}

#[test]
fn test_expire_bridge_trade_before_expiry_fails() {
    let (env, _, _, seller, buyer, client) = setup_bridge();
    let id = client.create_cross_chain_trade(
        &seller, &buyer, &1_000_000u64, &None,
        &soroban_sdk::String::from_str(&env, "ethereum"), &1000u32,
    );
    assert!(client.try_expire_bridge_trade(&id).is_err());
}

#[test]
fn test_confirm_expired_bridge_trade_fails() {
    let (env, _, _, seller, buyer, client) = setup_bridge();
    let id = client.create_cross_chain_trade(
        &seller, &buyer, &1_000_000u64, &None,
        &soroban_sdk::String::from_str(&env, "ethereum"), &10u32,
    );
    env.ledger().with_mut(|l| l.sequence_number += 11);
    assert!(client.try_confirm_bridge_deposit(
        &id, &soroban_sdk::String::from_str(&env, "0xabc")
    ).is_err());
}

// ---------------------------------------------------------------------------
// Insurance tests
// ---------------------------------------------------------------------------

fn setup_insurance() -> (Env, Address, Address, Address, Address, StellarEscrowContractClient<'static>) {
    let (env, token_addr, admin, seller, buyer, _, client) = setup();
    let provider = Address::generate(&env);
    client.register_insurance_provider(&provider);
    // mint tokens to buyer for premium payments and to provider for payouts
    token::StellarAssetClient::new(&env, &token_addr).mint(&provider, &10_000_000i128);
    (env, token_addr, admin, seller, buyer, client)
}

fn funded_trade(env: &Env, token_addr: &Address, seller: &Address, buyer: &Address, client: &StellarEscrowContractClient) -> u64 {
    let id = client.create_trade(seller, buyer, &1_000_000u64, &None, &OptionalMetadata::None);
    token::Client::new(env, token_addr).approve(buyer, &client.address, &1_000_000i128, &200u32);
    client.fund_trade(&id);
    id
}

#[test]
fn test_register_insurance_provider() {
    let (env, _, _, _, _, client) = setup_insurance();
    let provider = Address::generate(&env);
    client.register_insurance_provider(&provider);
    assert!(client.is_insurance_provider_registered(&provider));
    client.remove_insurance_provider(&provider);
    assert!(!client.is_insurance_provider_registered(&provider));
}

#[test]
fn test_purchase_insurance() {
    let (env, token_addr, _, seller, buyer, client) = setup_insurance();
    let provider = Address::generate(&env);
    client.register_insurance_provider(&provider);
    let id = funded_trade(&env, &token_addr, &seller, &buyer, &client);

    let buyer_before = token::Client::new(&env, &token_addr).balance(&buyer);
    // 100 bps = 1% of 1_000_000 = 10_000 premium
    client.purchase_insurance(&id, &provider, &100u32, &500_000u64);
    let buyer_after = token::Client::new(&env, &token_addr).balance(&buyer);
    assert_eq!(buyer_before - buyer_after, 10_000i128);

    let policy = client.get_insurance_policy(&id).unwrap();
    assert_eq!(policy.premium, 10_000u64);
    assert_eq!(policy.coverage, 500_000u64);
    assert!(!policy.claimed);
}

#[test]
fn test_purchase_insurance_unregistered_provider_fails() {
    let (env, token_addr, _, seller, buyer, client) = setup_insurance();
    let id = funded_trade(&env, &token_addr, &seller, &buyer, &client);
    let rando = Address::generate(&env);
    assert!(client.try_purchase_insurance(&id, &rando, &100u32, &500_000u64).is_err());
}

#[test]
fn test_purchase_insurance_premium_too_high_fails() {
    let (env, token_addr, _, seller, buyer, client) = setup_insurance();
    let provider = Address::generate(&env);
    client.register_insurance_provider(&provider);
    let id = funded_trade(&env, &token_addr, &seller, &buyer, &client);
    // 1001 bps > MAX_INSURANCE_PREMIUM_BPS (1000)
    assert!(client.try_purchase_insurance(&id, &provider, &1001u32, &500_000u64).is_err());
}

#[test]
fn test_claim_insurance() {
    let (env, token_addr, _, seller, buyer, client) = setup_insurance();
    let provider = Address::generate(&env);
    client.register_insurance_provider(&provider);
    token::StellarAssetClient::new(&env, &token_addr).mint(&provider, &10_000_000i128);

    let id = funded_trade(&env, &token_addr, &seller, &buyer, &client);
    client.purchase_insurance(&id, &provider, &100u32, &500_000u64);

    // raise dispute to make trade eligible for claim
    let arb = Address::generate(&env);
    client.register_arbitrator(&arb);
    // re-create a trade with arbitrator for dispute
    let id2 = client.create_trade(&seller, &buyer, &1_000_000u64, &Some(arb.clone()), &OptionalMetadata::None);
    token::Client::new(&env, &token_addr).approve(&buyer, &client.address, &1_000_000i128, &200u32);
    client.fund_trade(&id2);
    client.purchase_insurance(&id2, &provider, &100u32, &500_000u64);
    client.raise_dispute(&id2, &buyer);

    let seller_before = token::Client::new(&env, &token_addr).balance(&seller);
    client.claim_insurance(&id2, &seller, &200_000u64);
    let seller_after = token::Client::new(&env, &token_addr).balance(&seller);
    assert_eq!(seller_after - seller_before, 200_000i128);

    let policy = client.get_insurance_policy(&id2).unwrap();
    assert!(policy.claimed);
}

#[test]
fn test_claim_insurance_double_claim_fails() {
    let (env, token_addr, _, seller, buyer, client) = setup_insurance();
    let provider = Address::generate(&env);
    client.register_insurance_provider(&provider);
    token::StellarAssetClient::new(&env, &token_addr).mint(&provider, &10_000_000i128);

    let arb = Address::generate(&env);
    client.register_arbitrator(&arb);
    let id = client.create_trade(&seller, &buyer, &1_000_000u64, &Some(arb.clone()), &OptionalMetadata::None);
    token::Client::new(&env, &token_addr).approve(&buyer, &client.address, &1_000_000i128, &200u32);
    client.fund_trade(&id);
    client.purchase_insurance(&id, &provider, &100u32, &500_000u64);
    client.raise_dispute(&id, &buyer);

    client.claim_insurance(&id, &seller, &100_000u64);
    assert!(client.try_claim_insurance(&id, &seller, &100_000u64).is_err());
}

#[test]
fn test_claim_capped_at_coverage() {
    let (env, token_addr, _, seller, buyer, client) = setup_insurance();
    let provider = Address::generate(&env);
    client.register_insurance_provider(&provider);
    token::StellarAssetClient::new(&env, &token_addr).mint(&provider, &10_000_000i128);

    let arb = Address::generate(&env);
    client.register_arbitrator(&arb);
    let id = client.create_trade(&seller, &buyer, &1_000_000u64, &Some(arb.clone()), &OptionalMetadata::None);
    token::Client::new(&env, &token_addr).approve(&buyer, &client.address, &1_000_000i128, &200u32);
    client.fund_trade(&id);
    // coverage = 50_000
    client.purchase_insurance(&id, &provider, &100u32, &50_000u64);
    client.raise_dispute(&id, &buyer);

    let seller_before = token::Client::new(&env, &token_addr).balance(&seller);
    // request 999_999 but coverage is only 50_000
    client.claim_insurance(&id, &seller, &999_999u64);
    let seller_after = token::Client::new(&env, &token_addr).balance(&seller);
    assert_eq!(seller_after - seller_before, 50_000i128);
}
