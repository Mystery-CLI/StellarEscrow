#[cfg(test)]
mod tests {
    use crate::database::Database;
    use crate::fraud_service::rules::RuleEngine;
    use crate::fraud_service::{FraudDetectionService, FraudReport};
    use crate::models::{Event, TradeConfirmedData, TradeCreatedData};
    use chrono::Utc;
    use sqlx::PgPool;
    use std::sync::Arc;
    use uuid::Uuid;

    // Helper to create a mock Event
    fn create_mock_event(event_type: &str, data: serde_json::Value) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            category: "trade".to_string(),
            schema_version: 2,
            contract_id: "test_contract".to_string(),
            ledger: 1000,
            transaction_hash: "test_hash".to_string(),
            timestamp: Utc::now(),
            data,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_fraud_detection_flow() {
        // Mock rule engine for isolated testing
        let engine = RuleEngine::new(vec!["GBLACK...".to_string()]);

        // 1. Test Blacklist Rule
        let res = engine.check_blacklist("GBLACK...");
        assert!(res.is_some(), "Blacklisted address should be flagged");
        assert_eq!(res.unwrap().score, 100);

        // 2. Test Velocity Rule (Simulated)
        let mut recent_events = Vec::new();
        for i in 0..15 {
            recent_events.push(create_mock_event(
                "trade_created",
                serde_json::to_value(TradeCreatedData {
                    trade_id: i,
                    seller: "GUSER...".to_string(),
                    buyer: "GBUYER...".to_string(),
                    amount: 100,
                })
                .unwrap(),
            ));
        }
        let res = engine.check_velocity(&recent_events, "GUSER...");
        assert!(res.is_some(), "High velocity should be flagged");
        assert_eq!(res.unwrap().rule_name, "High Velocity");

        // 3. Test Linked Accounts Rule
        let res = engine.check_linked_accounts(&recent_events, "GUSER...", "GBUYER...");
        assert!(res.is_some(), "Linked accounts should be flagged");
        assert_eq!(res.unwrap().rule_name, "Linked Account Pattern");

        // 4. Test Slippage Rule
        let res = engine.check_slippage(1000, 200); // 20% fee
        assert!(res.is_some(), "High fee/slippage should be flagged");
        assert_eq!(res.unwrap().rule_name, "High Fee/Slippage");
    }
}
