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
    pub trades_completed: u64,
    pub disputes_raised: u64,
    pub volume_stroops: u64,
    pub captured_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// In-memory event aggregator for real-time statistics.
pub struct Aggregator {
    counts: HashMap<String, u64>,
    trades_created: u64,
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
            "trade_confirmed" => self.trades_completed += 1,
            "dispute_raised" => self.disputes_raised += 1,
            _ => {}
        }
    }

    pub fn window(&self) -> MetricWindow {
        MetricWindow {
            window_seconds: 300,
            event_counts: self.counts.clone(),
            total_events: self.total,
            trades_created: self.trades_created,
            trades_completed: self.trades_completed,
            disputes_raised: self.disputes_raised,
            volume_stroops: self.volume_stroops,
            captured_at: Some(Utc::now()),
        }
    }
}
