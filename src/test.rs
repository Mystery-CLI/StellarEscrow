#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

use crate::{
    HistoryFilter, SortCriterion, SortOrder, StellarEscrowContract,
    StellarEscrowContractClient, TradeFilter, TradeSortField, TradeStatus,
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

use crate::AnalyticsFilter;

fn analytics_filter_all(env: &Env) -> AnalyticsFilter {
    AnalyticsFilter {
        from_ledger: None,
        to_ledger: None,
        bucket_size: 0,
// Onboarding tests
// =============================================================================

use crate::{OnboardingStep, StepStatus};

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
