use super::{MonitoringSnapshot, HealthStatus};
use super::alerts::AlertSeverity;
use serde::{Deserialize, Serialize};

/// Dashboard summary returned by GET /monitoring/dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub health_status: HealthStatus,
    pub active_alert_count: usize,
    pub critical_alert_count: usize,
    pub metrics: std::collections::HashMap<String, f64>,
    pub top_alerts: Vec<AlertSummary>,
    pub grafana_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub name: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub fired_at: chrono::DateTime<chrono::Utc>,
}

pub fn build_dashboard(snapshot: &MonitoringSnapshot, grafana_url: Option<String>) -> DashboardSummary {
    let critical_count = snapshot
        .active_alerts
        .iter()
        .filter(|a| a.severity == AlertSeverity::Critical)
        .count();

    let top_alerts: Vec<AlertSummary> = snapshot
        .active_alerts
        .iter()
        .take(10)
        .map(|a| AlertSummary {
            name: a.rule_name.clone(),
            severity: a.severity.clone(),
            message: a.message.clone(),
            fired_at: a.fired_at,
        })
        .collect();

    DashboardSummary {
        health_status: snapshot.health_status.clone(),
        active_alert_count: snapshot.active_alerts.len(),
        critical_alert_count: critical_count,
        metrics: snapshot.metrics_summary.values.clone(),
        top_alerts,
        grafana_url,
    }
}
