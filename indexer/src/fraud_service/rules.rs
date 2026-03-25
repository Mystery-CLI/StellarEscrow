use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::models::{Event, TradeCreatedData};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_name: String,
    pub score: i32,
    pub description: String,
}

pub struct RuleEngine {
    blacklist: Vec<String>,
}

impl RuleEngine {
    pub fn new(blacklist: Vec<String>) -> Self {
        Self { blacklist }
    }

    pub fn check_blacklist(&self, address: &str) -> Option<RuleResult> {
        if self.blacklist.contains(&address.to_string()) {
            Some(RuleResult {
                rule_name: "Blacklisted Address".to_string(),
                score: 100,
                description: format!("Address {} is in the global blacklist.", address),
            })
        } else {
            None
        }
    }

    pub fn check_velocity(&self, recent_events: &[Event], current_addr: &str) -> Option<RuleResult> {
        let one_hour_ago = Utc::now() - Duration::hours(1);
        let count = recent_events.iter()
            .filter(|e| e.timestamp > one_hour_ago)
            .filter(|e| {
                if let Ok(data) = serde_json::from_value::<TradeCreatedData>(e.data.clone()) {
                    data.seller == current_addr || data.buyer == current_addr
                } else {
                    false
                }
            })
            .count();

        if count > 10 {
            Some(RuleResult {
                rule_name: "High Velocity".to_string(),
                score: 40,
                description: format!("Address {} has created {} trades in the last hour.", current_addr, count),
            })
        } else {
            None
        }
    }

    pub fn check_amount_outlier(&self, amount: u64, avg_amount: f64) -> Option<RuleResult> {
        if avg_amount > 0.0 && (amount as f64) > avg_amount * 5.0 {
            Some(RuleResult {
                rule_name: "Large Amount Outlier".to_string(),
                score: 30,
                description: format!("Transaction amount {} is significantly higher than the average {}.", amount, avg_amount),
            })
        } else {
            None
        }
    }

    pub fn check_linked_accounts(&self, recent_events: &[Event], current_addr: &str, counterparty: &str) -> Option<RuleResult> {
        // Detect if the seller and buyer have many small transactions between them or share a common third party
        let common_count = recent_events.iter()
            .filter(|e| {
                if let Ok(data) = serde_json::from_value::<TradeCreatedData>(e.data.clone()) {
                    (data.seller == current_addr && data.buyer == counterparty) ||
                    (data.seller == counterparty && data.buyer == current_addr)
                } else {
                    false
                }
            })
            .count();

        if common_count > 5 {
            Some(RuleResult {
                rule_name: "Linked Account Pattern".to_string(),
                score: 50,
                description: format!("Frequent interaction ({} trades) between {} and {} suggests linked accounts.", common_count, current_addr, counterparty),
            })
        } else {
            None
        }
    }

    pub fn check_slippage(&self, amount: u64, fee: u64) -> Option<RuleResult> {
        // Simple slippage/fee anomaly check: if fee is unusually high (> 10% of amount)
        if amount > 0 && (fee as f64 / amount as f64) > 0.1 {
            Some(RuleResult {
                rule_name: "High Fee/Slippage".to_string(),
                score: 30,
                description: format!("Fee {} is over 10% of trade amount {}. Potential slippage manipulation.", fee, amount),
            })
        } else {
            None
        }
    }
}
