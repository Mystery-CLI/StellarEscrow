#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Bytes, Env};

use crate::{
    AnalyticsFilter, ContractError, DisputeResolution, HistoryFilter, MetadataEntry,
    OnboardingStep, SortCriterion, SortOrder, StellarEscrowContract,
    StellarEscrowContractClient, StepStatus, TemplateTerms, TierConfig, TradeAction,
    TradeFilter, TradeMetadata, TradeSortField, TradeStatus, UserTier,
    VerificationStatus,
};

fn setup() -> (Env, StellarEscrowContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token = Address::generate(&env);
    client.initialize(&admin, &token, &100); // 1% fee
    (env, client, admin, seller, buyer)
}

fn setup_uninitialized(
) -> (
    Env,
    StellarEscrowContractClient<'static>,
    Address,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token = Address::generate(&env);
    (env, client, admin, seller, buyer, token)
}

fn no_filter(env: &Env) -> HistoryFilter {
    HistoryFilter { status: None, from_ledger: None, to_ledger: None }
}

fn empty_trade_filter() -> TradeFilter {
    TradeFilter {
        status: None,
        min_amount: None,
        max_amount: None,
        from_ledger: None,
        to_ledger: None,
        seller: None,
        buyer: None,
    }
}

fn sort_by_created_asc() -> SortCriterion {
    SortCriterion { field: TradeSortField::CreatedAt, order: SortOrder::Ascending }
}

fn sample_metadata(env: &Env) -> TradeMetadata {
    let mut entries = soroban_sdk::Vec::new(env);
    entries.push_back(MetadataEntry {
        key: soroban_sdk::String::from_str(env, "item"),
        value: soroban_sdk::String::from_str(env, "laptop"),
    });
    TradeMetadata { entries }
}

fn metadata_with_too_many_entries(env: &Env) -> TradeMetadata {
    let mut entries = soroban_sdk::Vec::new(env);
    for i in 0..11u32 {
        let key = std::format!("k{i}");
        let value = std::format!("v{i}");
        entries.push_back(MetadataEntry {
            key: soroban_sdk::String::from_str(env, key.as_str()),
            value: soroban_sdk::String::from_str(env, value.as_str()),
        });
    }
    TradeMetadata { entries }
}

fn metadata_with_oversized_value(env: &Env) -> TradeMetadata {
    let mut entries = soroban_sdk::Vec::new(env);
    let oversized = "a".repeat(257);
    entries.push_back(MetadataEntry {
        key: soroban_sdk::String::from_str(env, "notes"),
        value: soroban_sdk::String::from_str(env, oversized.as_str()),
    });
    TradeMetadata { entries }
}

fn sample_template_terms(
    env: &Env,
    fixed_amount: Option<u64>,
    arbitrator: Option<Address>,
    metadata: Option<TradeMetadata>,
) -> TemplateTerms {
    TemplateTerms {
        description: soroban_sdk::String::from_str(env, "Standard escrow terms"),
        default_arbitrator: arbitrator,
        fixed_amount,
        default_metadata: metadata,
    }
}

// =============================================================================
// Existing history tests
// =============================================================================

#[test]
fn test_history_empty_for_new_address() {
    let (env, client, _, seller, _) = setup();
    let page = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 0);
    assert_eq!(page.records.len(), 0);
}

#[test]
fn test_history_shows_created_trade() {
    let (env, client, _, seller, buyer) = setup();
    let trade_id = client.create_trade(&seller, &buyer, &1000, &None, &None);
    let page = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    let record = page.records.get(0).unwrap();
    assert_eq!(record.trade_id, trade_id);
    assert_eq!(record.amount, 1000);
    assert_eq!(record.status, TradeStatus::Created);
}

#[test]
fn test_history_visible_from_buyer_address() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    let page = client.get_transaction_history(&buyer, &no_filter(&env), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
}

#[test]
fn test_history_filter_by_status() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    client.cancel_trade(&1);
    let filter = HistoryFilter { status: Some(TradeStatus::Cancelled), from_ledger: None, to_ledger: None };
    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().status, TradeStatus::Cancelled);
}

#[test]
fn test_history_filter_by_ledger_range() {
    let (env, client, _, seller, buyer) = setup();
    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    env.ledger().set_sequence_number(100);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    let filter = HistoryFilter { status: None, from_ledger: Some(50), to_ledger: Some(200) };
    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 2000);
}

#[test]
fn test_history_sort_descending() {
    let (env, client, _, seller, buyer) = setup();
    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &100, &None, &None);
    env.ledger().set_sequence_number(10);
    client.create_trade(&seller, &buyer, &200, &None, &None);
    let page = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Descending, &0, &10);
    assert_eq!(page.records.get(0).unwrap().amount, 200);
    assert_eq!(page.records.get(1).unwrap().amount, 100);
}

#[test]
fn test_history_pagination() {
    let (env, client, _, seller, buyer) = setup();
    for _ in 0..5 {
        client.create_trade(&seller, &buyer, &1000, &None, &None);
    }
    let page1 = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Ascending, &0, &3);
    assert_eq!(page1.records.len(), 3);
    assert_eq!(page1.total, 5);
    let page2 = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Ascending, &3, &3);
    assert_eq!(page2.records.len(), 2);
}

// ---------------------------------------------------------------------------
// Trade creation form tests
// ---------------------------------------------------------------------------

#[test]
fn test_form_validate_valid_input() {
    let (_env, client, _, seller, buyer) = setup();
    client.validate_trade_form(&make_form_input(&seller, &buyer, 1_000_000, None));
}

#[test]
fn test_form_validate_zero_amount() {
    let (_env, client, _, seller, buyer) = setup();
    let result = client.try_validate_trade_form(&make_form_input(&seller, &buyer, 0, None));
    assert_eq!(result, Err(Ok(ContractError::InvalidAmount)));
}

#[test]
fn test_form_validate_buyer_seller_same() {
    let (_env, client, _, seller, _) = setup();
    let result = client.try_validate_trade_form(&make_form_input(&seller, &seller, 1_000_000, None));
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_form_validate_unregistered_arbitrator() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    let csv = client.export_transaction_csv(&seller, &HistoryFilter { status: None, from_ledger: None, to_ledger: None });
    assert!(csv.len() > 0);
}

// =============================================================================
// Advanced filtering tests
// =============================================================================

#[test]
fn test_search_trades_empty_state() {
    let (_env, client, _, _, _) = setup();
    let page = client.search_trades(&empty_trade_filter(), &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 0);
}

#[test]
fn test_search_trades_returns_all_without_filter() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    let page = client.search_trades(&empty_trade_filter(), &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 2);
}

#[test]
fn test_search_filter_by_status() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    client.cancel_trade(&1);
    let filter = TradeFilter { status: Some(TradeStatus::Cancelled), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().status, TradeStatus::Cancelled);
}

#[test]
fn test_search_filter_by_min_amount() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    let filter = TradeFilter { min_amount: Some(1000), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 2000);
}

#[test]
fn test_search_filter_by_max_amount() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    let filter = TradeFilter { max_amount: Some(1000), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 500);
}

#[test]
fn test_search_filter_by_amount_range() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &100, &None, &None);
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &5000, &None, &None);
    let filter = TradeFilter { min_amount: Some(200), max_amount: Some(1000), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 500);
}

#[test]
fn test_search_filter_by_seller() {
    let (env, client, _, seller, buyer) = setup();
    let other_seller = Address::generate(&env);
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&other_seller, &buyer, &2000, &None, &None);
    let filter = TradeFilter { seller: Some(seller.clone()), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().seller, seller);
}

#[test]
fn test_search_filter_by_buyer() {
    let (env, client, _, seller, buyer) = setup();
    let other_buyer = Address::generate(&env);
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &other_buyer, &2000, &None, &None);
    let filter = TradeFilter { buyer: Some(buyer.clone()), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().buyer, buyer);
}

#[test]
fn test_search_multi_criteria_filter() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &1500, &None, &None);
    client.cancel_trade(&1);
    // status=Cancelled AND amount>=200
    let filter = TradeFilter {
        status: Some(TradeStatus::Cancelled),
        min_amount: Some(200),
        ..empty_trade_filter()
    };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 500);
}

#[test]
fn test_search_sort_by_amount_descending() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &300, &None, &None);
    client.create_trade(&seller, &buyer, &100, &None, &None);
    client.create_trade(&seller, &buyer, &200, &None, &None);
    let sort = SortCriterion { field: TradeSortField::Amount, order: SortOrder::Descending };
    let page = client.search_trades(&empty_trade_filter(), &sort, &0, &10).unwrap();
    assert_eq!(page.records.get(0).unwrap().amount, 300);
    assert_eq!(page.records.get(1).unwrap().amount, 200);
    assert_eq!(page.records.get(2).unwrap().amount, 100);
}

#[test]
fn test_search_sort_by_amount_ascending() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &300, &None, &None);
    client.create_trade(&seller, &buyer, &100, &None, &None);
    client.create_trade(&seller, &buyer, &200, &None, &None);
    let sort = SortCriterion { field: TradeSortField::Amount, order: SortOrder::Ascending };
    let page = client.search_trades(&empty_trade_filter(), &sort, &0, &10).unwrap();
    assert_eq!(page.records.get(0).unwrap().amount, 100);
    assert_eq!(page.records.get(2).unwrap().amount, 300);
}

#[test]
fn test_search_invalid_amount_range_returns_error() {
    let (_env, client, _, _, _) = setup();
    let filter = TradeFilter { min_amount: Some(1000), max_amount: Some(100), ..empty_trade_filter() };
    assert!(client.try_search_trades(&filter, &sort_by_created_asc(), &0, &10).is_err());
}

#[test]
fn test_search_invalid_ledger_range_returns_error() {
    let (_env, client, _, _, _) = setup();
    let filter = TradeFilter { from_ledger: Some(500), to_ledger: Some(100), ..empty_trade_filter() };
    assert!(client.try_search_trades(&filter, &sort_by_created_asc(), &0, &10).is_err());
}

#[test]
fn test_search_pagination() {
    let (_env, client, _, seller, buyer) = setup();
    for _ in 0..5 {
        client.create_trade(&seller, &buyer, &1000, &None, &None);
    }
    let page1 = client.search_trades(&empty_trade_filter(), &sort_by_created_asc(), &0, &3).unwrap();
    assert_eq!(page1.records.len(), 3);
    assert_eq!(page1.total, 5);
    let page2 = client.search_trades(&empty_trade_filter(), &sort_by_created_asc(), &3, &3).unwrap();
    assert_eq!(page2.records.len(), 2);
}

#[test]
fn test_search_trades_for_address() {
    let (env, client, _, seller, buyer) = setup();
    let other = Address::generate(&env);
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&other, &buyer, &2000, &None, &None);
    // seller's index only has trade 1
    let page = client.search_trades_for_address(&seller, &empty_trade_filter(), &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 1000);
}

// =============================================================================
// Filter preset tests
// =============================================================================

#[test]
fn test_save_and_retrieve_preset() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "my preset");
    let preset_id = client.save_filter_preset(
        &seller, &name, &empty_trade_filter(), &sort_by_created_asc(),
    ).unwrap();
    let preset = client.get_filter_preset(&preset_id).unwrap();
    assert_eq!(preset.id, preset_id);
    assert_eq!(preset.owner, seller);
}

#[test]
fn test_list_presets_for_user() {
    let (env, client, _, seller, _) = setup();
    let name1 = soroban_sdk::String::from_str(&env, "preset one");
    let name2 = soroban_sdk::String::from_str(&env, "preset two");
    client.save_filter_preset(&seller, &name1, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    client.save_filter_preset(&seller, &name2, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    let presets = client.list_filter_presets(&seller);
    assert_eq!(presets.len(), 2);
}

#[test]
fn test_update_preset() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "original");
    let preset_id = client.save_filter_preset(&seller, &name, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    let new_name = soroban_sdk::String::from_str(&env, "updated");
    let new_filter = TradeFilter { min_amount: Some(100), ..empty_trade_filter() };
    client.update_filter_preset(&seller, &preset_id, &new_name, &new_filter, &sort_by_created_asc()).unwrap();
    let preset = client.get_filter_preset(&preset_id).unwrap();
    assert_eq!(preset.filter.min_amount, Some(100));
}

#[test]
fn test_delete_preset() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "to delete");
    let preset_id = client.save_filter_preset(&seller, &name, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    client.delete_filter_preset(&seller, &preset_id).unwrap();
    assert!(client.try_get_filter_preset(&preset_id).is_err());
}

#[test]
fn test_delete_preset_removes_from_list() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "p1");
    let preset_id = client.save_filter_preset(&seller, &name, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    client.delete_filter_preset(&seller, &preset_id).unwrap();
    let presets = client.list_filter_presets(&seller);
    assert_eq!(presets.len(), 0);
}

#[test]
fn test_unauthorized_delete_fails() {
    let (env, client, _, seller, buyer) = setup();
    let name = soroban_sdk::String::from_str(&env, "mine");
    let preset_id = client.save_filter_preset(&seller, &name, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    assert!(client.try_delete_filter_preset(&buyer, &preset_id).is_err());
}

#[test]
fn test_search_with_preset() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &1500, &None, &None);
    let filter = TradeFilter { min_amount: Some(1000), ..empty_trade_filter() };
    let name = soroban_sdk::String::from_str(&env, "big trades");
    let preset_id = client.save_filter_preset(&seller, &name, &filter, &sort_by_created_asc()).unwrap();
    let page = client.search_with_preset(&preset_id, &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 1500);
}

#[test]
fn test_preset_persistence_across_calls() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "persistent");
    let filter = TradeFilter { status: Some(TradeStatus::Created), ..empty_trade_filter() };
    let preset_id = client.save_filter_preset(&seller, &name, &filter, &sort_by_created_asc()).unwrap();
    // Retrieve in a separate call — verifies persistent storage
    let preset = client.get_filter_preset(&preset_id).unwrap();
    assert_eq!(preset.filter.status, Some(TradeStatus::Created));
}

// =============================================================================
// Analytics Charts & Graphs Tests
// =============================================================================

fn analytics_filter_all(env: &Env) -> AnalyticsFilter {
    AnalyticsFilter {
        from_ledger: None,
        to_ledger: None,
        bucket_size: 0,
    }
}

// Onboarding tests
// =============================================================================

#[test]
fn test_onboarding_start_creates_progress() {
    let (_, client, _, seller, _) = setup();

    let progress = client.start_onboarding(&seller);

    assert!(!progress.finished);
    assert_eq!(progress.current_step, OnboardingStep::RegisterProfile);
    assert_eq!(progress.step_statuses.len(), 4);
    // All steps start as Pending
    for i in 0..4 {
        assert_eq!(progress.step_statuses.get(i).unwrap(), StepStatus::Pending);
    }
}

#[test]
fn test_volume_chart_empty_state() {
    let (env, client, _, _, _) = setup();
    let data = client.get_volume_chart(&analytics_filter_all(&env)).unwrap();
    assert_eq!(data.total_volume, 0);
    assert_eq!(data.total_trades, 0);
    assert_eq!(data.points.len(), 0);
}

#[test]
fn test_volume_chart_accumulates_trade_amounts() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    let data = client.get_volume_chart(&analytics_filter_all(&env)).unwrap();
    assert_eq!(data.total_trades, 2);
    assert_eq!(data.total_volume, 3000);
}

#[test]
fn test_volume_chart_filter_by_ledger_range() {
    let (env, client, _, seller, buyer) = setup();

    env.ledger().set_sequence_number(10);
    client.create_trade(&seller, &buyer, &500, &None, &None);

    env.ledger().set_sequence_number(200);
    client.create_trade(&seller, &buyer, &1500, &None, &None);

    let filter = AnalyticsFilter {
        from_ledger: Some(100),
        to_ledger: Some(300),
        bucket_size: 0,
    };
    let data = client.get_volume_chart(&filter).unwrap();
    assert_eq!(data.total_trades, 1);
    assert_eq!(data.total_volume, 1500);
}

#[test]
fn test_volume_chart_invalid_range_returns_error() {
    let (env, client, _, _, _) = setup();
    let filter = AnalyticsFilter {
        from_ledger: Some(500),
        to_ledger: Some(100),
        bucket_size: 0,
    };
    let result = client.try_get_volume_chart(&filter);
    assert!(result.is_err());
}

#[test]
fn test_success_rate_empty_state() {
    let (_env, client, _, _, _) = setup();
    let data = client.get_success_rate();
    assert_eq!(data.success_rate_bps, 0);
    assert_eq!(data.completed, 0);
}

#[test]
fn test_status_distribution_reflects_trade_states() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    client.cancel_trade(&2);

    let dist = client.get_status_distribution(&analytics_filter_all(&env)).unwrap();
    assert_eq!(dist.created, 1);
    assert_eq!(dist.cancelled, 1);
}

#[test]
fn test_fee_chart_empty_state() {
    let (env, client, _, _, _) = setup();
    let data = client.get_fee_chart(&analytics_filter_all(&env)).unwrap();
    assert_eq!(data.total_fees, 0);
    assert_eq!(data.points.len(), 0);
}

#[test]
fn test_user_stats_empty_state() {
    let (_env, client, _, seller, _) = setup();
    let snapshot = client.get_user_stats(&seller);
    assert_eq!(snapshot.total_trades, 0);
    assert_eq!(snapshot.total_volume, 0);
    assert_eq!(snapshot.success_rate_bps, 0);
}

#[test]
fn test_user_stats_reflects_created_trades() {
    let (_env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    let snapshot = client.get_user_stats(&seller);
    assert_eq!(snapshot.total_trades, 2);
    assert_eq!(snapshot.total_volume, 3000);
}

#[test]
fn test_export_platform_analytics_csv_has_header() {
    let (_env, client, _, _, _) = setup();
    let csv = client.export_platform_analytics_csv();
    assert!(csv.len() > 0);
}

#[test]
fn test_export_volume_chart_csv_valid_output() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    let csv = client.export_volume_chart_csv(&analytics_filter_all(&env)).unwrap();
    assert!(csv.len() > 0);
}

#[test]
fn test_export_user_stats_csv_valid_output() {
    let (_env, client, _, seller, _) = setup();
    let csv = client.export_user_stats_csv(&seller);
    assert!(csv.len() > 0);
}

#[test]
fn test_user_volume_chart_empty_state() {
    let (env, client, _, seller, _) = setup();
    let data = client.get_user_volume_chart(&seller, &analytics_filter_all(&env)).unwrap();
    assert_eq!(data.total_trades, 0);
    assert_eq!(data.total_volume, 0);
}

#[test]
fn test_onboarding_start_is_idempotent() {
    let (_, client, _, seller, _) = setup();

    let first = client.start_onboarding(&seller);
    let second = client.start_onboarding(&seller);

    // Second call returns the same progress without resetting it
    assert_eq!(first.started_at, second.started_at);
    assert_eq!(first.current_step, second.current_step);
}

#[test]
fn test_onboarding_complete_step_advances_progress() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    // Complete step 0 (RegisterProfile)
    let progress = client.complete_onboarding_step(&seller, &0);

    assert_eq!(progress.step_statuses.get(0).unwrap(), StepStatus::Done);
    assert_eq!(progress.current_step, OnboardingStep::AcknowledgeFees);
    assert!(!progress.finished);
}

#[test]
fn test_onboarding_complete_all_steps_marks_finished() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    for i in 0..4u32 {
        client.complete_onboarding_step(&seller, &i);
    }

    let progress = client.get_onboarding_progress(&seller).unwrap();
    assert!(progress.finished);
    assert_eq!(progress.current_step, OnboardingStep::Completed);
    for i in 0..4 {
        assert_eq!(progress.step_statuses.get(i).unwrap(), StepStatus::Done);
    }
}

#[test]
fn test_onboarding_skip_step_advances_without_completing() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    // Skip step 0
    let progress = client.skip_onboarding_step(&seller, &0);

    assert_eq!(progress.step_statuses.get(0).unwrap(), StepStatus::Skipped);
    assert_eq!(progress.current_step, OnboardingStep::AcknowledgeFees);
    assert!(!progress.finished);
}

#[test]
fn test_onboarding_skip_all_steps_marks_finished() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    for i in 0..4u32 {
        client.skip_onboarding_step(&seller, &i);
    }

    let progress = client.get_onboarding_progress(&seller).unwrap();
    assert!(progress.finished);
    assert_eq!(progress.current_step, OnboardingStep::Completed);
    for i in 0..4 {
        assert_eq!(progress.step_statuses.get(i).unwrap(), StepStatus::Skipped);
    }
}

#[test]
fn test_onboarding_exit_marks_all_pending_as_skipped() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);
    // Complete step 0 first
    client.complete_onboarding_step(&seller, &0);

    // Exit — steps 1, 2, 3 should become Skipped
    let progress = client.exit_onboarding(&seller);

    assert!(progress.finished);
    assert_eq!(progress.step_statuses.get(0).unwrap(), StepStatus::Done);
    assert_eq!(progress.step_statuses.get(1).unwrap(), StepStatus::Skipped);
    assert_eq!(progress.step_statuses.get(2).unwrap(), StepStatus::Skipped);
    assert_eq!(progress.step_statuses.get(3).unwrap(), StepStatus::Skipped);
}

#[test]
fn test_onboarding_progress_is_persisted_and_resumable() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);
    client.complete_onboarding_step(&seller, &0);
    client.skip_onboarding_step(&seller, &1);

    // Simulate resume: start_onboarding returns existing progress
    let resumed = client.start_onboarding(&seller);
    assert_eq!(resumed.step_statuses.get(0).unwrap(), StepStatus::Done);
    assert_eq!(resumed.step_statuses.get(1).unwrap(), StepStatus::Skipped);
    assert_eq!(resumed.current_step, OnboardingStep::CreateFirstTemplate);
}

#[test]
fn test_onboarding_get_progress_returns_none_before_start() {
    let (_, client, _, seller, _) = setup();

    let progress = client.get_onboarding_progress(&seller);
    assert!(progress.is_none());
}

#[test]
fn test_onboarding_is_active_reflects_state() {
    let (_, client, _, seller, _) = setup();

    assert!(!client.is_onboarding_active(&seller));

    client.start_onboarding(&seller);
    assert!(client.is_onboarding_active(&seller));

    for i in 0..4u32 {
        client.complete_onboarding_step(&seller, &i);
    }
    assert!(!client.is_onboarding_active(&seller));
}

#[test]
fn test_onboarding_does_not_affect_existing_trades() {
    let (env, client, _, seller, buyer) = setup();

    // Create a trade before onboarding
    let trade_id = client.create_trade(&seller, &buyer, &1000, &None, &None);

    // Run through onboarding
    client.start_onboarding(&seller);
    client.exit_onboarding(&seller);

    // Trade is unaffected
    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().trade_id, trade_id);
}

#[test]
fn test_onboarding_independent_per_user() {
    let (_, client, _, seller, buyer) = setup();

    client.start_onboarding(&seller);
    client.complete_onboarding_step(&seller, &0);

    // Buyer has no onboarding yet
    assert!(client.get_onboarding_progress(&buyer).is_none());

    // Start buyer's onboarding — starts fresh
    let buyer_progress = client.start_onboarding(&buyer);
    assert_eq!(buyer_progress.current_step, OnboardingStep::RegisterProfile);

    // Seller's progress is unchanged
    let seller_progress = client.get_onboarding_progress(&seller).unwrap();
    assert_eq!(seller_progress.step_statuses.get(0).unwrap(), StepStatus::Done);
}

// =============================================================================
// Expanded Edge, Security, and Integration Coverage
// =============================================================================

#[test]
fn test_initialize_rejects_second_initialization_attempt() {
    let (env, client, admin, _, _) = setup();
    let second_token = Address::generate(&env);

    let result = client.try_initialize(&admin, &second_token, &250);

    assert_eq!(result, Err(Ok(ContractError::AlreadyInitialized)));
}

#[test]
fn test_mutating_calls_fail_before_initialization() {
    let (_env, client, _admin, seller, buyer, _token) = setup_uninitialized();

    let create_result = client.try_create_trade(&seller, &buyer, &1_000, &None, &None);
    let pause_result = client.try_pause();
    let onboarding_result = client.try_start_onboarding(&seller);

    assert_eq!(create_result, Err(Ok(ContractError::NotInitialized)));
    assert_eq!(pause_result, Err(Ok(ContractError::NotInitialized)));
    assert_eq!(onboarding_result, Err(Ok(ContractError::NotInitialized)));
}

#[test]
fn test_trade_metadata_round_trip_and_clear() {
    let (env, client, _, seller, buyer) = setup();
    let metadata = sample_metadata(&env);

    let trade_id = client.create_trade(&seller, &buyer, &2_000, &None, &Some(metadata.clone()));
    assert_eq!(client.get_trade_metadata(&trade_id), Some(metadata.clone()));

    client.update_trade_metadata(&trade_id, &None);
    assert_eq!(client.get_trade_metadata(&trade_id), None);
}

#[test]
fn test_trade_metadata_validation_rejects_oversized_payloads() {
    let (env, client, _, seller, buyer) = setup();
    let trade_id = client.create_trade(&seller, &buyer, &2_000, &None, &None);

    let too_many = metadata_with_too_many_entries(&env);
    let too_long = metadata_with_oversized_value(&env);

    let create_too_many = client.try_create_trade(&seller, &buyer, &2_000, &None, &Some(too_many.clone()));
    let update_too_many = client.try_update_trade_metadata(&trade_id, &Some(too_many));
    let create_too_long = client.try_create_trade(&seller, &buyer, &2_000, &None, &Some(too_long.clone()));
    let update_too_long = client.try_update_trade_metadata(&trade_id, &Some(too_long));

    assert_eq!(create_too_many, Err(Ok(ContractError::MetadataTooManyEntries)));
    assert_eq!(update_too_many, Err(Ok(ContractError::MetadataTooManyEntries)));
    assert_eq!(create_too_long, Err(Ok(ContractError::MetadataValueTooLong)));
    assert_eq!(update_too_long, Err(Ok(ContractError::MetadataValueTooLong)));
}

#[test]
fn test_pause_blocks_mutations_until_contract_is_unpaused() {
    let (env, client, _, seller, buyer) = setup();
    let mut batch = soroban_sdk::Vec::new(&env);
    batch.push_back((buyer.clone(), 750u64, None));

    client.pause();

    let create_result = client.try_create_trade(&seller, &buyer, &500, &None, &None);
    let batch_result = client.try_batch_create_trades(&seller, &batch);

    assert_eq!(create_result, Err(Ok(ContractError::ContractPaused)));
    assert_eq!(batch_result, Err(Ok(ContractError::ContractPaused)));

    client.unpause();
    let trade_id = client.create_trade(&seller, &buyer, &500, &None, &None);
    assert_eq!(client.get_trade(&trade_id).status, TradeStatus::Created);
}

#[test]
fn test_raise_dispute_rejects_non_participant_even_when_arbitrator_exists() {
    let (env, client, _, seller, buyer) = setup();
    let arbitrator = Address::generate(&env);
    let outsider = Address::generate(&env);

    client.register_arbitrator(&arbitrator);
    let trade_id = client.create_trade(&seller, &buyer, &5_000, &Some(arbitrator), &None);
    client.fund_trade(&trade_id);

    let result = client.try_raise_dispute(&trade_id, &outsider);

    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_dispute_resolution_flow_exposes_arbitrator_action_and_collects_fees() {
    let (env, client, _, seller, buyer) = setup();
    let arbitrator = Address::generate(&env);

    client.register_arbitrator(&arbitrator);
    let trade_id = client.create_trade(&seller, &buyer, &10_000, &Some(arbitrator.clone()), &None);
    client.fund_trade(&trade_id);
    client.raise_dispute(&trade_id, &buyer);

    let detail = client.get_trade_detail(&trade_id, &arbitrator);
    assert_eq!(detail.trade.status, TradeStatus::Disputed);
    assert_eq!(detail.available_actions.len(), 1);
    assert_eq!(detail.available_actions.get(0).unwrap(), TradeAction::ResolveDispute);

    client.resolve_dispute(&trade_id, &DisputeResolution::ReleaseToSeller);

    let trade = client.get_trade(&trade_id);
    assert_eq!(trade.status, TradeStatus::Disputed);
    assert_eq!(client.get_accumulated_fees(), trade.fee);
}

#[test]
fn test_batch_trade_flow_covers_create_fund_complete_and_confirm() {
    let (env, client, _, seller, buyer) = setup();
    let mut batch = soroban_sdk::Vec::new(&env);
    batch.push_back((buyer.clone(), 1_000u64, None));
    batch.push_back((buyer.clone(), 2_500u64, None));

    let trade_ids = client.batch_create_trades(&seller, &batch);
    assert_eq!(trade_ids.len(), 2);

    client.batch_fund_trades(&buyer, &trade_ids);
    for trade_id in trade_ids.iter() {
        client.complete_trade(&trade_id);
    }

    client.batch_confirm_trades(&buyer, &trade_ids);

    let trade_one = client.get_trade(&trade_ids.get(0).unwrap());
    let trade_two = client.get_trade(&trade_ids.get(1).unwrap());
    assert_eq!(trade_one.status, TradeStatus::Completed);
    assert_eq!(trade_two.status, TradeStatus::Completed);
    assert!(client.get_accumulated_fees() > 0);
}

#[test]
fn test_batch_create_rejects_empty_and_oversized_batches() {
    let (env, client, _, seller, buyer) = setup();
    let empty = soroban_sdk::Vec::new(&env);
    let mut oversized = soroban_sdk::Vec::new(&env);
    for _ in 0..101 {
        oversized.push_back((buyer.clone(), 100u64, None));
    }

    let empty_result = client.try_batch_create_trades(&seller, &empty);
    let oversized_result = client.try_batch_create_trades(&seller, &oversized);

    assert_eq!(empty_result, Err(Ok(ContractError::EmptyBatch)));
    assert_eq!(oversized_result, Err(Ok(ContractError::BatchLimitExceeded)));
}

#[test]
fn test_custom_fee_configuration_changes_trade_fee_and_tier_state() {
    let (_env, client, _, seller, buyer) = setup();
    let config = TierConfig {
        bronze_fee_bps: 100,
        silver_fee_bps: 80,
        gold_fee_bps: 50,
    };

    client.set_tier_config(&config);
    client.set_user_custom_fee(&seller, &25);

    let fee_bps = client.get_effective_fee_bps(&seller);
    let trade_id = client.create_trade(&seller, &buyer, &10_000, &None, &None);
    let trade = client.get_trade(&trade_id);
    let tier = client.get_user_tier(&seller).unwrap();

    assert_eq!(fee_bps, 25);
    assert_eq!(trade.fee, 25);
    assert_eq!(tier.tier, UserTier::Custom);
    assert_eq!(tier.custom_fee_bps, Some(25));

    client.remove_user_custom_fee(&seller);
    let tier_after_removal = client.get_user_tier(&seller).unwrap();
    assert_eq!(tier_after_removal.tier, UserTier::Bronze);
    assert_eq!(tier_after_removal.custom_fee_bps, None);
}

#[test]
fn test_template_lifecycle_enforces_defaults_versions_and_deactivation() {
    let (env, client, _, seller, buyer) = setup();
    let arbitrator = Address::generate(&env);
    let metadata = sample_metadata(&env);

    client.register_arbitrator(&arbitrator);
    let terms_v1 = sample_template_terms(&env, Some(5_000), Some(arbitrator.clone()), Some(metadata.clone()));
    let template_name = soroban_sdk::String::from_str(&env, "Hardware escrow");
    let template_id = client.create_template(&seller, &template_name, &terms_v1);

    let trade_id = client.create_trade_from_template(&seller, &buyer, &template_id, &5_000);
    let trade = client.get_trade(&trade_id);
    assert_eq!(trade.arbitrator, Some(arbitrator.clone()));
    assert_eq!(trade.metadata, Some(metadata.clone()));

    let terms_v2 = sample_template_terms(&env, Some(7_000), Some(arbitrator), None);
    let updated_name = soroban_sdk::String::from_str(&env, "Hardware escrow v2");
    client.update_template(&seller, &template_id, &updated_name, &terms_v2);

    let template = client.get_template(&template_id);
    assert_eq!(template.current_version, 2);

    let mismatch = client.try_create_trade_from_template(&seller, &buyer, &template_id, &5_000);
    assert_eq!(mismatch, Err(Ok(ContractError::TemplateAmountMismatch)));

    client.deactivate_template(&seller, &template_id);
    let inactive = client.try_create_trade_from_template(&seller, &buyer, &template_id, &7_000);
    assert_eq!(inactive, Err(Ok(ContractError::TemplateInactive)));
}

#[test]
fn test_admin_transfer_and_fee_withdraw_flow_are_persisted() {
    let (env, client, _, seller, buyer) = setup();
    let new_admin = Address::generate(&env);
    let fee_receiver = Address::generate(&env);

    let trade_id = client.create_trade(&seller, &buyer, &10_000, &None, &None);
    client.fund_trade(&trade_id);
    client.complete_trade(&trade_id);
    client.confirm_receipt(&trade_id);

    assert!(client.get_accumulated_fees() > 0);

    client.transfer_admin(&new_admin);
    client.update_fee(&250);
    client.withdraw_fees(&fee_receiver);

    assert_eq!(client.get_platform_fee_bps(), 250);
    assert_eq!(client.get_accumulated_fees(), 0);

    let logs = client.get_audit_logs(&0, &10);
    assert!(logs.len() >= 2);
    assert_eq!(logs.get(0).unwrap().action, soroban_sdk::String::from_str(&env, "admin.fees_withdrawn"));
    assert_eq!(logs.get(1).unwrap().actor, new_admin);
}

#[test]
fn test_trade_lifecycle_updates_dashboard_detail_and_user_stats() {
    let (_env, client, _, seller, buyer) = setup();

    let trade_id = client.create_trade(&seller, &buyer, &4_000, &None, &None);
    client.fund_trade(&trade_id);
    client.complete_trade(&trade_id);
    client.confirm_receipt(&trade_id);

    let trade = client.get_trade(&trade_id);
    let detail = client.get_trade_detail(&trade_id, &buyer);
    let dashboard = client.get_dashboard();
    let seller_stats = client.get_user_analytics(&seller);
    let buyer_stats = client.get_user_analytics(&buyer);

    assert_eq!(detail.seller_payout, trade.amount - trade.fee);
    assert_eq!(detail.timeline.len(), 3);
    assert_eq!(detail.timeline.get(0).unwrap().status, TradeStatus::Created);
    assert_eq!(detail.timeline.get(1).unwrap().status, TradeStatus::Funded);
    assert_eq!(detail.timeline.get(2).unwrap().status, TradeStatus::Completed);
    assert_eq!(dashboard.platform.completed_trades, 1);
    assert_eq!(seller_stats.completed_trades, 1);
    assert_eq!(buyer_stats.completed_trades, 1);
}

#[test]
fn test_user_profile_preference_and_security_settings_round_trip() {
    let (env, client, _, seller, _) = setup();
    let empty = Bytes::new(&env);
    let avatar = Bytes::new(&env);
    let pref_key = soroban_sdk::String::from_str(&env, "theme");
    let pref_value = soroban_sdk::String::from_str(&env, "light");

    client.register_user(&seller, &empty, &empty);
    client.set_user_preference(&seller, &pref_key, &pref_value);
    client.set_user_verification(&seller, &VerificationStatus::Verified);
    client.update_avatar(&seller, &Some(avatar));
    client.update_security_settings(&seller, &true, &900);

    let preference = client.get_user_preference(&seller, &pref_key);
    let profile = client.get_user_profile(&seller);

    assert_eq!(preference.value, pref_value);
    assert_eq!(profile.verification, VerificationStatus::Verified);
    assert!(profile.avatar_hash.is_some());
    assert!(profile.two_fa_enabled);
    assert_eq!(profile.session_timeout_secs, 900);
}

#[test]
fn test_audit_log_captures_trade_and_admin_events_in_reverse_order() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1_500, &None, &None);
    client.pause();
    client.unpause();

    assert_eq!(client.audit_count(), 3);

    let logs = client.get_audit_logs(&0, &3);
    assert_eq!(logs.len(), 3);
    assert_eq!(logs.get(0).unwrap().action, soroban_sdk::String::from_str(&env, "contract.unpause"));
    assert_eq!(logs.get(1).unwrap().action, soroban_sdk::String::from_str(&env, "contract.pause"));
    assert_eq!(logs.get(2).unwrap().action, soroban_sdk::String::from_str(&env, "trade.created"));
}

#[test]
fn test_onboarding_rejects_invalid_or_replayed_step_completion() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    let out_of_range = client.try_complete_onboarding_step(&seller, &10);
    assert_eq!(out_of_range, Err(Ok(ContractError::InvalidAmount)));

    client.complete_onboarding_step(&seller, &0);
    let duplicate = client.try_complete_onboarding_step(&seller, &0);
    assert_eq!(duplicate, Err(Ok(ContractError::InvalidStatus)));
}
