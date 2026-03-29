use super::metrics::{
    MetricsSummary, METRIC_AML_HIGH_RISK, METRIC_COMPLIANCE_BLOCKED, METRIC_ERROR_RATE,
    METRIC_FRAUD_ALERTS, METRIC_TRADES_DISPUTED,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub metric: String,
    pub threshold: f64,
    /// "gt" | "lt" | "gte" | "lte"
    pub operator: String,
    pub severity: AlertSeverity,
    pub message_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertState {
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub current_value: f64,
    pub threshold: f64,
    pub message: String,
    pub fired_at: DateTime<Utc>,
}

/// Evaluate a single alert rule against the current metrics snapshot.
pub fn evaluate_rule(rule: &AlertRule, snapshot: &MetricsSummary) -> Option<AlertState> {
    let value = *snapshot.values.get(&rule.metric)?;

    let triggered = match rule.operator.as_str() {
        "gt" => value > rule.threshold,
        "gte" => value >= rule.threshold,
        "lt" => value < rule.threshold,
        "lte" => value <= rule.threshold,
        _ => false,
    };

    if !triggered {
        return None;
    }

    Some(AlertState {
        rule_name: rule.name.clone(),
        severity: rule.severity.clone(),
        current_value: value,
        threshold: rule.threshold,
        message: rule
            .message_template
            .replace("{value}", &value.to_string())
            .replace("{threshold}", &rule.threshold.to_string()),
        fired_at: Utc::now(),
    })
}

/// Default alert rules for the StellarEscrow platform.
pub fn default_alert_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "high_error_rate".to_string(),
            metric: METRIC_ERROR_RATE.to_string(),
            threshold: 5.0,
            operator: "gt".to_string(),
            severity: AlertSeverity::Critical,
            message_template: "Error rate {value}% exceeds threshold {threshold}%".to_string(),
        },
        AlertRule {
            name: "high_dispute_rate".to_string(),
            metric: METRIC_TRADES_DISPUTED.to_string(),
            threshold: 50.0,
            operator: "gt".to_string(),
            severity: AlertSeverity::High,
            message_template: "Dispute count {value} exceeds threshold {threshold}".to_string(),
        },
        AlertRule {
            name: "compliance_blocks_spike".to_string(),
            metric: METRIC_COMPLIANCE_BLOCKED.to_string(),
            threshold: 10.0,
            operator: "gt".to_string(),
            severity: AlertSeverity::High,
            message_template: "Compliance blocks {value} exceeds threshold {threshold}".to_string(),
        },
        AlertRule {
            name: "aml_high_risk_spike".to_string(),
            metric: METRIC_AML_HIGH_RISK.to_string(),
            threshold: 5.0,
            operator: "gt".to_string(),
            severity: AlertSeverity::Critical,
            message_template: "AML high-risk addresses {value} exceeds threshold {threshold}".to_string(),
        },
        AlertRule {
            name: "fraud_alerts_spike".to_string(),
            metric: METRIC_FRAUD_ALERTS.to_string(),
            threshold: 20.0,
            operator: "gt".to_string(),
            severity: AlertSeverity::High,
            message_template: "Fraud alerts {value} exceeds threshold {threshold}".to_string(),
        },
    ]
}
