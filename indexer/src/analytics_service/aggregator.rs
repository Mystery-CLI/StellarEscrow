use crate::models::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

/// Rolling 5-minute real-time window of event counts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricWindow {
    pub window_seconds: u64,
    pub event_counts: HashMap<String, u64>,
    pub total_events: u64,
    pub trades_created: u64,
    pub trades_funded: u64,
    pub trades_completed: u64,
    pub disputes_raised: u64,
    pub volume_stroops: u64,
    pub completion_rate_bps: u32,
    pub dispute_rate_bps: u32,
    pub captured_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// In-memory event aggregator for real-time statistics.
pub struct Aggregator {
    counts: HashMap<String, u64>,
    trades_created: u64,
    trades_funded: u64,
    trades_completed: u64,
    disputes_raised: u64,
    volume_stroops: u64,
    total: u64,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            trades_created: 0,
            trades_funded: 0,
            trades_completed: 0,
            disputes_raised: 0,
            volume_stroops: 0,
            total: 0,
        }
    }

    pub fn ingest(&mut self, event: &Event) {
        *self.counts.entry(event.event_type.clone()).or_insert(0) += 1;
        self.total += 1;

        match event.event_type.as_str() {
            "trade_created" => {
                self.trades_created += 1;
                if let Some(amount) = event.data.get("amount").and_then(|v| v.as_u64()) {
                    self.volume_stroops = self.volume_stroops.saturating_add(amount);
                }
            }
            "trade_funded" => self.trades_funded += 1,
            "trade_confirmed" => self.trades_completed += 1,
            "dispute_raised" => self.disputes_raised += 1,
            _ => {}
        }
    }

    pub fn window(&self) -> MetricWindow {
        let completion_rate_bps = if self.trades_created > 0 {
            ((self.trades_completed * 10_000) / self.trades_created) as u32
        } else {
            0
        };
        let dispute_rate_bps = if self.trades_created > 0 {
            ((self.disputes_raised * 10_000) / self.trades_created) as u32
        } else {
            0
        };

        MetricWindow {
            window_seconds: 300,
            event_counts: self.counts.clone(),
            total_events: self.total,
            trades_created: self.trades_created,
            trades_funded: self.trades_funded,
            trades_completed: self.trades_completed,
            disputes_raised: self.disputes_raised,
            volume_stroops: self.volume_stroops,
            completion_rate_bps,
            dispute_rate_bps,
            captured_at: Some(Utc::now()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Event;
    use chrono::Utc;
    use uuid::Uuid;

    fn build_event(event_type: &str, data: serde_json::Value) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            category: "trade".to_string(),
            schema_version: 1,
            contract_id: "contract".to_string(),
            ledger: 1,
            transaction_hash: "tx".to_string(),
            timestamp: Utc::now(),
            data,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn ingest_updates_totals_and_rates() {
        let mut agg = Aggregator::new();
        agg.ingest(&build_event("trade_created", serde_json::json!({"amount": 100})));
        agg.ingest(&build_event("trade_created", serde_json::json!({"amount": 200})));
        agg.ingest(&build_event("trade_funded", serde_json::json!({"trade_id": 1})));
        agg.ingest(&build_event("trade_confirmed", serde_json::json!({"trade_id": 1})));
        agg.ingest(&build_event("dispute_raised", serde_json::json!({"trade_id": 2})));

        let window = agg.window();
        assert_eq!(window.total_events, 5);
        assert_eq!(window.trades_created, 2);
        assert_eq!(window.trades_funded, 1);
        assert_eq!(window.trades_completed, 1);
        assert_eq!(window.disputes_raised, 1);
        assert_eq!(window.volume_stroops, 300);
        assert_eq!(window.completion_rate_bps, 5_000);
        assert_eq!(window.dispute_rate_bps, 5_000);
    }
}
