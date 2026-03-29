pub mod metrics;
pub mod alerts;
pub mod dashboard;

use crate::config::MonitoringConfig;
use crate::database::Database;
use alerts::{AlertRule, AlertSeverity, AlertState};
use metrics::MetricsCollector;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSnapshot {
    pub timestamp: DateTime<Utc>,
    pub active_alerts: Vec<AlertState>,
    pub metrics_summary: metrics::MetricsSummary,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct MonitoringService {
    db: Arc<Database>,
    config: MonitoringConfig,
    collector: MetricsCollector,
    alert_rules: Vec<AlertRule>,
    active_alerts: Arc<RwLock<Vec<AlertState>>>,
}

impl MonitoringService {
    pub fn new(db: Arc<Database>, config: MonitoringConfig) -> Self {
        let alert_rules = alerts::default_alert_rules();
        Self {
            db,
            config,
            collector: MetricsCollector::new(),
            alert_rules,
            active_alerts: Arc::new(RwLock::new(vec![])),
        }
    }

    /// Record a metric data point.
    pub fn record(&self, name: &str, value: f64, labels: Vec<(&str, &str)>) {
        self.collector.record(name, value, labels);
    }

    /// Evaluate all alert rules against current metrics.
    pub async fn evaluate_alerts(&self) {
        let snapshot = self.collector.snapshot();
        let mut fired: Vec<AlertState> = vec![];

        for rule in &self.alert_rules {
            if let Some(alert) = alerts::evaluate_rule(rule, &snapshot) {
                if alert.severity == AlertSeverity::Critical || alert.severity == AlertSeverity::High {
                    self.fire_alert(&alert).await;
                }
                fired.push(alert);
            }
        }

        let mut active = self.active_alerts.write().await;
        *active = fired;
    }

    /// Get current monitoring snapshot.
    pub async fn get_snapshot(&self) -> MonitoringSnapshot {
        let active_alerts = self.active_alerts.read().await.clone();
        let metrics_summary = self.collector.snapshot();
        let health_status = self.determine_health(&active_alerts);

        MonitoringSnapshot {
            timestamp: Utc::now(),
            active_alerts,
            metrics_summary,
            health_status,
        }
    }

    /// Get active alerts.
    pub async fn get_active_alerts(&self) -> Vec<AlertState> {
        self.active_alerts.read().await.clone()
    }

    /// Get Prometheus-format metrics text.
    pub fn prometheus_metrics(&self) -> String {
        self.collector.to_prometheus()
    }

    /// Background loop — evaluates alerts on the configured interval.
    pub async fn run_alert_loop(self: Arc<Self>) {
        let interval = std::time::Duration::from_secs(self.config.alert_eval_interval_secs);
        loop {
            tokio::time::sleep(interval).await;
            self.evaluate_alerts().await;
        }
    }

    async fn fire_alert(&self, alert: &AlertState) {
        tracing::warn!(
            alert_name = %alert.rule_name,
            severity = ?alert.severity,
            value = alert.current_value,
            "Alert fired"
        );

        if !self.config.alert_webhook_url.is_empty() {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "alert": alert.rule_name,
                "severity": format!("{:?}", alert.severity),
                "value": alert.current_value,
                "threshold": alert.threshold,
                "fired_at": alert.fired_at,
                "message": alert.message,
            });
            if let Err(e) = client
                .post(&self.config.alert_webhook_url)
                .json(&payload)
                .send()
                .await
            {
                tracing::error!("Failed to send alert webhook: {}", e);
            }
        }

        // Critical alerts also go to incident response
        if alert.severity == AlertSeverity::Critical && !self.config.incident_webhook_url.is_empty() {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "routing_key": "stellar-escrow",
                "event_action": "trigger",
                "payload": {
                    "summary": format!("[CRITICAL] {}: {}", alert.rule_name, alert.message),
                    "severity": "critical",
                    "custom_details": { "value": alert.current_value, "threshold": alert.threshold }
                }
            });
            if let Err(e) = client
                .post(&self.config.incident_webhook_url)
                .json(&payload)
                .send()
                .await
            {
                tracing::error!("Failed to send incident webhook: {}", e);
            }
        }
    }

    fn determine_health(&self, alerts: &[AlertState]) -> HealthStatus {
        if alerts.iter().any(|a| a.severity == AlertSeverity::Critical) {
            HealthStatus::Critical
        } else if alerts.iter().any(|a| a.severity == AlertSeverity::High) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}
