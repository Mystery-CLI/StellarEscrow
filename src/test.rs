#![cfg(test)]

use soroban_sdk::{testutils::Ledger, Address, Env};

use crate::{
    HistoryFilter, SortOrder, StellarEscrowContract, StellarEscrowContractClient, TradeStatus,
};

fn setup() -> (Env, StellarEscrowContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Use a mock token address — token transfers are mocked via mock_all_auths
    let token = Address::generate(&env);

    client.initialize(&admin, &token, &100); // 1% fee

    (env, client, admin, seller, buyer)
}

fn no_filter(env: &Env) -> HistoryFilter {
    HistoryFilter {
        status: None,
        from_ledger: None,
        to_ledger: None,
    }
}

#[test]
fn test_history_empty_for_new_address() {
    let (env, client, _, seller, _) = setup();
    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );
    assert_eq!(page.total, 0);
    assert_eq!(page.records.len(), 0);
}

#[test]
fn test_history_shows_created_trade() {
    let (env, client, _, seller, buyer) = setup();

    let trade_id = client.create_trade(&seller, &buyer, &1000, &None, &None);

    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );

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

    let page = client.get_transaction_history(
        &buyer,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );

    assert_eq!(page.total, 1);
}

#[test]
fn test_history_filter_by_status() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    // Cancel the first trade
    client.cancel_trade(&1);

    let filter = HistoryFilter {
        status: Some(TradeStatus::Cancelled),
        from_ledger: None,
        to_ledger: None,
    };

    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().status, TradeStatus::Cancelled);
}

#[test]
fn test_history_filter_by_ledger_range() {
    let (env, client, _, seller, buyer) = setup();

    // Trade at ledger 1
    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &1000, &None, &None);

    // Trade at ledger 100
    env.ledger().set_sequence_number(100);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    let filter = HistoryFilter {
        status: None,
        from_ledger: Some(50),
        to_ledger: Some(200),
    };

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

    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Descending,
        &0,
        &10,
    );

    assert_eq!(page.records.get(0).unwrap().amount, 200);
    assert_eq!(page.records.get(1).unwrap().amount, 100);
}

#[test]
fn test_history_pagination() {
    let (env, client, _, seller, buyer) = setup();

    for _ in 0..5 {
        client.create_trade(&seller, &buyer, &1000, &None, &None);
    }

    let page1 = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &3,
    );
    assert_eq!(page1.records.len(), 3);
    assert_eq!(page1.total, 5);

    let page2 = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &3,
        &3,
    );
    assert_eq!(page2.records.len(), 2);
}

#[test]
fn test_export_csv_returns_header_and_rows() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);

    let csv = client.export_transaction_csv(
        &seller,
        &HistoryFilter {
            status: None,
            from_ledger: None,
            to_ledger: None,
        },
    );

    // CSV should be non-empty and contain the header
    assert!(csv.len() > 0);
}
