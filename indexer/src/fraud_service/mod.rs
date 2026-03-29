pub mod ml;
pub mod rules;

#[cfg(test)]
mod test;

use crate::database::Database;
use crate::models::Event;
use ml::{MLAnalyzer, MLResult};
use rules::{RuleEngine, RuleResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudReport {
    pub trade_id: u64,
    pub risk_score: i32,
    pub rules_triggered: Vec<RuleResult>,
    pub ml_result: MLResult,
    pub status: String,
}

pub struct FraudDetectionService {
    db: Arc<Database>,
    rules: RuleEngine,
    ml: MLAnalyzer,
}

impl FraudDetectionService {
    pub async fn new(db: Arc<Database>) -> Self {
        // Fetch blacklist from DB in a real scenario
        let blacklist = vec![];
        Self {
            db,
            rules: RuleEngine::new(blacklist),
            ml: MLAnalyzer::new(),
        }
    }

    pub async fn process_event(&self, event: &Event) -> Option<FraudReport> {
        if event.event_type != "trade_created" {
            return None;
        }

        let data: crate::models::TradeCreatedData =
            serde_json::from_value(event.data.clone()).ok()?;
        let trade_id = data.trade_id;

        let mut triggered_rules = Vec::new();

        // 1. Rule-based checks
        if let Some(res) = self.rules.check_blacklist(&data.seller) {
            triggered_rules.push(res);
        }
        if let Some(res) = self.rules.check_blacklist(&data.buyer) {
            triggered_rules.push(res);
        }

        // Fetch recent events for velocity check
        let recent_events = self
            .db
            .get_events(&crate::models::EventQuery {
                limit: Some(100),
                offset: None,
                event_type: Some("trade_created".to_string()),
                trade_id: None,
                from_ledger: None,
                to_ledger: None,
            })
            .await
            .unwrap_or_default();

        if let Some(res) = self.rules.check_velocity(&recent_events, &data.seller) {
            triggered_rules.push(res);
        }

        if let Some(res) =
            self.rules
                .check_linked_accounts(&recent_events, &data.seller, &data.buyer)
        {
            triggered_rules.push(res);
        }

        // Fetch fee (this might need to be done after trade is confirmed, but for now we check at creation if possible,
        // or we check 'slippage' as amount/fee ratio if fee is known)
        // Note: in TradeCreatedData, fee might not be present yet depending on contract,
        // but we can estimate or check if the contract provides it.
        // For now, let's assume we check it during confirmed events too.

        // 2. ML check
        let features = vec![data.amount as f32, triggered_rules.len() as f32];
        let ml_res = self.ml.analyze(features);

        // 3. Aggregate score
        let rule_score: i32 = triggered_rules.iter().map(|r| r.score).sum();
        let final_score = (rule_score + ml_res.score as i32).min(100);

        let status = if final_score >= 80 {
            "high_risk"
        } else if final_score >= 40 {
            "medium_risk"
        } else {
            "low_risk"
        };

        Some(FraudReport {
            trade_id,
            risk_score: final_score,
            rules_triggered: triggered_rules,
            ml_result: ml_res,
            status: status.to_string(),
        })
    }

    pub async fn process_confirmed_event(&self, event: &Event) -> Option<FraudReport> {
        if event.event_type != "trade_confirmed" {
            return None;
        }

        let data: crate::models::TradeConfirmedData =
            serde_json::from_value(event.data.clone()).ok()?;
        let trade_id = data.trade_id;

        let mut triggered_rules = Vec::new();

        // Slippage check
        if let Some(res) = self.rules.check_slippage(data.payout + data.fee, data.fee) {
            triggered_rules.push(res);
        }

        if triggered_rules.is_empty() {
            return None;
        }

        let risk_score: i32 = triggered_rules.iter().map(|r| r.score).sum();
        let final_score = risk_score.min(100);

        Some(FraudReport {
            trade_id,
            risk_score: final_score,
            rules_triggered: triggered_rules,
            ml_result: MLResult {
                score: 0.0,
                contribution: "N/A".to_string(),
            },
            status: (if final_score >= 80 {
                "high_risk"
            } else {
                "medium_risk"
            })
            .to_string(),
        })
    }
}
