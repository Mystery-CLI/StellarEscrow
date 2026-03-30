use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub timestamp: DateTime<Utc>,
    /// Latest value per metric name
    pub values: HashMap<String, f64>,
}

/// In-memory metrics collector with Prometheus text export.
pub struct MetricsCollector {
    points: Mutex<Vec<MetricPoint>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            points: Mutex::new(Vec::new()),
        }
    }

    pub fn record(&self, name: &str, value: f64, labels: Vec<(&str, &str)>) {
        let label_map: HashMap<String, String> = labels
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let point = MetricPoint {
            name: name.to_string(),
            value,
            labels: label_map,
            recorded_at: Utc::now(),
        };

        if let Ok(mut pts) = self.points.lock() {
            pts.push(point);
            // Keep last 10k points to avoid unbounded growth
            if pts.len() > 10_000 {
                pts.drain(0..1_000);
            }
        }
    }

    /// Get the latest value for each metric name.
    pub fn snapshot(&self) -> MetricsSummary {
        let mut values: HashMap<String, f64> = HashMap::new();
        if let Ok(pts) = self.points.lock() {
            for pt in pts.iter() {
                values.insert(pt.name.clone(), pt.value);
            }
        }
        MetricsSummary {
            timestamp: Utc::now(),
            values,
        }
    }

    /// Export metrics in Prometheus text format.
    pub fn to_prometheus(&self) -> String {
        let snapshot = self.snapshot();
        let mut lines = Vec::new();

        for (name, value) in &snapshot.values {
            let safe_name = name.replace('.', "_").replace('-', "_");
            lines.push(format!("# HELP {} StellarEscrow metric", safe_name));
            lines.push(format!("# TYPE {} gauge", safe_name));
            lines.push(format!("{} {}", safe_name, value));
        }

        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// Well-known metric names used across the service
// ---------------------------------------------------------------------------

pub const METRIC_EVENTS_PROCESSED: &str = "stellar_escrow_events_processed_total";
pub const METRIC_TRADES_CREATED: &str = "stellar_escrow_trades_created_total";
pub const METRIC_TRADES_COMPLETED: &str = "stellar_escrow_trades_completed_total";
pub const METRIC_TRADES_DISPUTED: &str = "stellar_escrow_trades_disputed_total";
pub const METRIC_COMPLIANCE_CHECKS: &str = "stellar_escrow_compliance_checks_total";
pub const METRIC_COMPLIANCE_BLOCKED: &str = "stellar_escrow_compliance_blocked_total";
pub const METRIC_AML_HIGH_RISK: &str = "stellar_escrow_aml_high_risk_total";
pub const METRIC_DB_QUERY_DURATION_MS: &str = "stellar_escrow_db_query_duration_ms";
pub const METRIC_API_REQUEST_DURATION_MS: &str = "stellar_escrow_api_request_duration_ms";
pub const METRIC_WEBSOCKET_CONNECTIONS: &str = "stellar_escrow_websocket_connections";
pub const METRIC_FRAUD_ALERTS: &str = "stellar_escrow_fraud_alerts_total";
pub const METRIC_ERROR_RATE: &str = "stellar_escrow_error_rate";

// APM-specific metrics exposed to Prometheus
pub const METRIC_API_AVG_RESPONSE_MS: &str = "stellar_escrow_api_avg_response_ms";
pub const METRIC_API_P95_RESPONSE_MS: &str = "stellar_escrow_api_p95_response_ms";
pub const METRIC_API_P99_RESPONSE_MS: &str = "stellar_escrow_api_p99_response_ms";
pub const METRIC_API_REQUESTS_TOTAL: &str = "stellar_escrow_api_requests_total";
pub const METRIC_API_REQUESTS_PER_MINUTE: &str = "stellar_escrow_api_requests_per_minute";
pub const METRIC_ACTIVE_PERF_ALERTS: &str = "stellar_escrow_active_perf_alerts";
