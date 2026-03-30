//! Test data factories and versioning for integration tests (issue #343).
pub const TEST_DATA_VERSION: &str = "1.0.0";

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::models::Event;

/// Deterministic sample trade event for tests (no real PII).
pub fn sample_trade_created_event() -> Event {
    Event {
        id: Uuid::nil(),
        event_type: "trade_created".to_string(),
        category: "trade".to_string(),
        schema_version: 1,
        contract_id: "CTESTCONTRACT000000000000000000000000000000000000000000000".to_string(),
        ledger: 1,
        transaction_hash: "test-tx".to_string(),
        timestamp: Utc::now(),
        data: json!({
            "trade_id": 42_u64,
            "buyer": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            "seller": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            "v": 1
        }),
        created_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_produces_valid_event_type() {
        let e = sample_trade_created_event();
        assert_eq!(e.event_type, "trade_created");
    }
}
