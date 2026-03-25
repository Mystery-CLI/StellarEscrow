use axum::{extract::State, response::Json};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: ServiceStatus,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
    pub last_checked: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub requests_total: u64,
    pub requests_per_minute: f64,
    pub avg_response_ms: f64,
    pub p95_response_ms: f64,
    pub error_rate: f64,
    pub active_ws_connections: u64,
    pub events_indexed_total: u64,
    pub last_ledger_processed: Option<i64>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: ServiceStatus,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub components: Vec<ComponentHealth>,
    pub metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub component: String,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

// ─── Metrics Collector ───────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct MetricsCollector {
    pub requests_total: u64,
    pub errors_total: u64,
    pub response_times_ms: Vec<u64>,
    pub active_ws_connections: u64,
    pub events_indexed_total: u64,
    pub last_ledger_processed: Option<i64>,
    pub window_start: Option<Instant>,
    pub window_requests: u64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            window_start: Some(Instant::now()),
            ..Default::default()
        }
    }

    pub fn record_request(&mut self, duration_ms: u64, is_error: bool) {
        self.requests_total += 1;
        self.window_requests += 1;
        if is_error {
            self.errors_total += 1;
        }
        // Keep last 1000 samples for percentile calculation
        if self.response_times_ms.len() >= 1000 {
            self.response_times_ms.remove(0);
        }
        self.response_times_ms.push(duration_ms);
    }

    pub fn snapshot(&self, started_at: Instant) -> PerformanceMetrics {
        let uptime_seconds = started_at.elapsed().as_secs();

        let avg_response_ms = if self.response_times_ms.is_empty() {
            0.0
        } else {
            self.response_times_ms.iter().sum::<u64>() as f64
                / self.response_times_ms.len() as f64
        };

        let p95_response_ms = {
            let mut sorted = self.response_times_ms.clone();
            sorted.sort_unstable();
            let idx = (sorted.len() as f64 * 0.95) as usize;
            sorted.get(idx).copied().unwrap_or(0) as f64
        };

        let error_rate = if self.requests_total == 0 {
            0.0
        } else {
            self.errors_total as f64 / self.requests_total as f64
        };

        let requests_per_minute = if let Some(ws) = self.window_start {
            let elapsed_mins = ws.elapsed().as_secs_f64() / 60.0;
            if elapsed_mins > 0.0 {
                self.window_requests as f64 / elapsed_mins
            } else {
                0.0
            }
        } else {
            0.0
        };

        PerformanceMetrics {
            requests_total: self.requests_total,
            requests_per_minute,
            avg_response_ms,
            p95_response_ms,
            error_rate,
            active_ws_connections: self.active_ws_connections,
            events_indexed_total: self.events_indexed_total,
            last_ledger_processed: self.last_ledger_processed,
            uptime_seconds,
        }
    }
}

// ─── Health Monitor ───────────────────────────────────────────────────────────

pub struct HealthMonitor {
    db_pool: PgPool,
    horizon_url: String,
    http_client: Client,
    started_at: Instant,
    pub metrics: Arc<RwLock<MetricsCollector>>,
    pub alerts: Arc<RwLock<Vec<Alert>>>,
}

impl HealthMonitor {
    pub fn new(db_pool: PgPool, horizon_url: String) -> Self {
        Self {
            db_pool,
            horizon_url,
            http_client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            started_at: Instant::now(),
            metrics: Arc::new(RwLock::new(MetricsCollector::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run a full health check across all components.
    pub async fn check(&self) -> SystemHealth {
        let db = self.check_database().await;
        let horizon = self.check_horizon().await;

        let overall = if db.status == ServiceStatus::Unhealthy
            || horizon.status == ServiceStatus::Unhealthy
        {
            ServiceStatus::Unhealthy
        } else if db.status == ServiceStatus::Degraded
            || horizon.status == ServiceStatus::Degraded
        {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Healthy
        };

        let metrics = self.metrics.read().await.snapshot(self.started_at);

        // Evaluate alert conditions
        self.evaluate_alerts(&db, &horizon, &metrics).await;

        SystemHealth {
            status: overall,
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: Utc::now(),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            components: vec![db, horizon],
            metrics,
        }
    }

    async fn check_database(&self) -> ComponentHealth {
        let start = Instant::now();
        let result = sqlx::query("SELECT 1").execute(&self.db_pool).await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(_) => ComponentHealth {
                name: "database".to_string(),
                status: if latency_ms > 500 {
                    ServiceStatus::Degraded
                } else {
                    ServiceStatus::Healthy
                },
                latency_ms: Some(latency_ms),
                message: if latency_ms > 500 {
                    Some(format!("High latency: {}ms", latency_ms))
                } else {
                    None
                },
                last_checked: Utc::now(),
            },
            Err(e) => {
                error!("Database health check failed: {}", e);
                ComponentHealth {
                    name: "database".to_string(),
                    status: ServiceStatus::Unhealthy,
                    latency_ms: Some(latency_ms),
                    message: Some(e.to_string()),
                    last_checked: Utc::now(),
                }
            }
        }
    }

    async fn check_horizon(&self) -> ComponentHealth {
        let url = format!("{}/", self.horizon_url);
        let start = Instant::now();
        let result = self.http_client.get(&url).send().await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 200 => {
                ComponentHealth {
                    name: "stellar_horizon".to_string(),
                    status: if latency_ms > 2000 {
                        ServiceStatus::Degraded
                    } else {
                        ServiceStatus::Healthy
                    },
                    latency_ms: Some(latency_ms),
                    message: if latency_ms > 2000 {
                        Some(format!("Slow response: {}ms", latency_ms))
                    } else {
                        None
                    },
                    last_checked: Utc::now(),
                }
            }
            Ok(resp) => ComponentHealth {
                name: "stellar_horizon".to_string(),
                status: ServiceStatus::Degraded,
                latency_ms: Some(latency_ms),
                message: Some(format!("Unexpected status: {}", resp.status())),
                last_checked: Utc::now(),
            },
            Err(e) => {
                warn!("Horizon health check failed: {}", e);
                ComponentHealth {
                    name: "stellar_horizon".to_string(),
                    status: ServiceStatus::Unhealthy,
                    latency_ms: Some(latency_ms),
                    message: Some(e.to_string()),
                    last_checked: Utc::now(),
                }
            }
        }
    }

    async fn evaluate_alerts(
        &self,
        db: &ComponentHealth,
        horizon: &ComponentHealth,
        metrics: &PerformanceMetrics,
    ) {
        let mut alerts = self.alerts.write().await;

        let fire = |alerts: &mut Vec<Alert>,
                    component: &str,
                    severity: AlertSeverity,
                    message: &str| {
            // Avoid duplicate active alerts for the same component
            let already_active = alerts.iter().any(|a: &Alert| {
                a.component == component && a.resolved_at.is_none() && a.message == message
            });
            if !already_active {
                warn!("[ALERT] {} - {}: {}", severity_label(&severity), component, message);
                alerts.push(Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    severity,
                    component: component.to_string(),
                    message: message.to_string(),
                    triggered_at: Utc::now(),
                    resolved_at: None,
                });
            }
        };

        // Auto-resolve alerts for healthy components
        let now = Utc::now();
        for alert in alerts.iter_mut() {
            if alert.resolved_at.is_some() {
                continue;
            }
            let comp_healthy = match alert.component.as_str() {
                "database" => db.status == ServiceStatus::Healthy,
                "stellar_horizon" => horizon.status == ServiceStatus::Healthy,
                _ => false,
            };
            if comp_healthy {
                alert.resolved_at = Some(now);
                info!("Alert resolved for component: {}", alert.component);
            }
        }

        if db.status == ServiceStatus::Unhealthy {
            fire(&mut alerts, "database", AlertSeverity::Critical, "Database is unreachable");
        } else if db.status == ServiceStatus::Degraded {
            fire(&mut alerts, "database", AlertSeverity::Warning, "Database latency is high");
        }

        if horizon.status == ServiceStatus::Unhealthy {
            fire(&mut alerts, "stellar_horizon", AlertSeverity::Critical, "Stellar Horizon is unreachable");
        } else if horizon.status == ServiceStatus::Degraded {
            fire(&mut alerts, "stellar_horizon", AlertSeverity::Warning, "Stellar Horizon response is slow");
        }

        if metrics.error_rate > 0.1 {
            fire(
                &mut alerts,
                "api",
                AlertSeverity::Warning,
                "Error rate exceeds 10%",
            );
        }

        if metrics.error_rate > 0.25 {
            fire(
                &mut alerts,
                "api",
                AlertSeverity::Critical,
                "Error rate exceeds 25%",
            );
        }
    }

    /// Background loop: persist metrics snapshot to DB every 60 seconds.
    pub async fn run_metrics_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let snapshot = self.metrics.read().await.snapshot(self.started_at);
            if let Err(e) = self.persist_metrics(&snapshot).await {
                error!("Failed to persist metrics: {}", e);
            }
        }
    }

    async fn persist_metrics(&self, m: &PerformanceMetrics) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO health_metrics
                (timestamp, requests_total, requests_per_minute, avg_response_ms,
                 p95_response_ms, error_rate, active_ws_connections,
                 events_indexed_total, last_ledger_processed, uptime_seconds)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#,
        )
        .bind(Utc::now())
        .bind(m.requests_total as i64)
        .bind(m.requests_per_minute)
        .bind(m.avg_response_ms)
        .bind(m.p95_response_ms)
        .bind(m.error_rate)
        .bind(m.active_ws_connections as i64)
        .bind(m.events_indexed_total as i64)
        .bind(m.last_ledger_processed)
        .bind(m.uptime_seconds as i64)
        .execute(&self.db_pool)
        .await?;
        Ok(())
    }
}

fn severity_label(s: &AlertSeverity) -> &'static str {
    match s {
        AlertSeverity::Info => "INFO",
        AlertSeverity::Warning => "WARNING",
        AlertSeverity::Critical => "CRITICAL",
    }
}

// ─── Shared State ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct HealthState {
    pub monitor: Arc<HealthMonitor>,
}

// ─── HTTP Handlers ────────────────────────────────────────────────────────────

/// GET /health — quick liveness probe (no DB check).
pub async fn liveness() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "timestamp": Utc::now() }))
}

/// GET /health/ready — full readiness check across all components.
pub async fn readiness(State(state): State<HealthState>) -> (axum::http::StatusCode, Json<SystemHealth>) {
    let health = state.monitor.check().await;
    let code = match health.status {
        ServiceStatus::Healthy => axum::http::StatusCode::OK,
        ServiceStatus::Degraded => axum::http::StatusCode::OK,
        ServiceStatus::Unhealthy => axum::http::StatusCode::SERVICE_UNAVAILABLE,
    };
    (code, Json(health))
}

/// GET /health/metrics — current performance metrics snapshot.
pub async fn metrics(State(state): State<HealthState>) -> Json<PerformanceMetrics> {
    let m = state.monitor.metrics.read().await;
    Json(m.snapshot(state.monitor.started_at))
}

/// GET /health/alerts — active and recent alerts.
pub async fn alerts(State(state): State<HealthState>) -> Json<serde_json::Value> {
    let alerts = state.monitor.alerts.read().await;
    let active: Vec<&Alert> = alerts.iter().filter(|a| a.resolved_at.is_none()).collect();
    let resolved: Vec<&Alert> = alerts.iter().filter(|a| a.resolved_at.is_some()).collect();
    Json(json!({
        "active": active,
        "resolved": resolved,
        "total_active": active.len(),
        "total_resolved": resolved.len(),
    }))
}

/// GET /status — human-readable status page.
pub async fn status_page(State(state): State<HealthState>) -> axum::response::Html<String> {
    let health = state.monitor.check().await;
    let alerts = state.monitor.alerts.read().await;
    let active_alerts: Vec<&Alert> = alerts.iter().filter(|a| a.resolved_at.is_none()).collect();

    let status_color = match health.status {
        ServiceStatus::Healthy => "#22c55e",
        ServiceStatus::Degraded => "#f59e0b",
        ServiceStatus::Unhealthy => "#ef4444",
    };

    let status_label = match health.status {
        ServiceStatus::Healthy => "All Systems Operational",
        ServiceStatus::Degraded => "Partial Degradation",
        ServiceStatus::Unhealthy => "Service Disruption",
    };

    let components_html: String = health
        .components
        .iter()
        .map(|c| {
            let color = match c.status {
                ServiceStatus::Healthy => "#22c55e",
                ServiceStatus::Degraded => "#f59e0b",
                ServiceStatus::Unhealthy => "#ef4444",
            };
            let label = match c.status {
                ServiceStatus::Healthy => "Operational",
                ServiceStatus::Degraded => "Degraded",
                ServiceStatus::Unhealthy => "Outage",
            };
            let latency = c
                .latency_ms
                .map(|l| format!("{}ms", l))
                .unwrap_or_else(|| "—".to_string());
            let msg = c.message.as_deref().unwrap_or("");
            format!(
                r#"<tr>
                  <td style="padding:10px 16px">{name}</td>
                  <td style="padding:10px 16px"><span style="color:{color};font-weight:600">{label}</span></td>
                  <td style="padding:10px 16px">{latency}</td>
                  <td style="padding:10px 16px;color:#6b7280;font-size:0.85em">{msg}</td>
                </tr>"#,
                name = c.name,
                color = color,
                label = label,
                latency = latency,
                msg = msg
            )
        })
        .collect();

    let alerts_html: String = if active_alerts.is_empty() {
        "<p style='color:#6b7280'>No active alerts.</p>".to_string()
    } else {
        active_alerts
            .iter()
            .map(|a| {
                let color = match a.severity {
                    AlertSeverity::Info => "#3b82f6",
                    AlertSeverity::Warning => "#f59e0b",
                    AlertSeverity::Critical => "#ef4444",
                };
                format!(
                    r#"<div style="border-left:4px solid {color};padding:10px 16px;margin-bottom:8px;background:#1f2937;border-radius:4px">
                      <strong style="color:{color}">[{severity:?}]</strong> <strong>{component}</strong>: {message}
                      <div style="color:#9ca3af;font-size:0.8em;margin-top:4px">{triggered_at}</div>
                    </div>"#,
                    color = color,
                    severity = a.severity,
                    component = a.component,
                    message = a.message,
                    triggered_at = a.triggered_at.format("%Y-%m-%d %H:%M:%S UTC")
                )
            })
            .collect()
    };

    let m = &health.metrics;
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>StellarEscrow — System Status</title>
  <style>
    *{{box-sizing:border-box;margin:0;padding:0}}
    body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:#111827;color:#f9fafb;padding:32px 16px}}
    .container{{max-width:900px;margin:0 auto}}
    h1{{font-size:1.8rem;margin-bottom:4px}}
    .subtitle{{color:#9ca3af;margin-bottom:32px}}
    .card{{background:#1f2937;border-radius:8px;padding:24px;margin-bottom:24px}}
    h2{{font-size:1.1rem;margin-bottom:16px;color:#e5e7eb}}
    table{{width:100%;border-collapse:collapse}}
    th{{text-align:left;padding:8px 16px;color:#9ca3af;font-size:0.85rem;border-bottom:1px solid #374151}}
    tr:hover{{background:#374151}}
    .badge{{display:inline-block;padding:4px 12px;border-radius:9999px;font-size:0.85rem;font-weight:600}}
    .metrics-grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:16px}}
    .metric{{background:#374151;border-radius:6px;padding:16px}}
    .metric-value{{font-size:1.5rem;font-weight:700;margin-bottom:4px}}
    .metric-label{{color:#9ca3af;font-size:0.8rem}}
    footer{{color:#6b7280;font-size:0.8rem;margin-top:32px;text-align:center}}
  </style>
</head>
<body>
<div class="container">
  <h1>StellarEscrow Status</h1>
  <p class="subtitle">v{version} &nbsp;·&nbsp; Uptime: {uptime}s &nbsp;·&nbsp; {timestamp}</p>

  <div class="card">
    <h2>Overall Status</h2>
    <span class="badge" style="background:{status_color};color:#fff;font-size:1rem">{status_label}</span>
  </div>

  <div class="card">
    <h2>Components</h2>
    <table>
      <thead><tr><th>Component</th><th>Status</th><th>Latency</th><th>Notes</th></tr></thead>
      <tbody>{components_html}</tbody>
    </table>
  </div>

  <div class="card">
    <h2>Performance Metrics</h2>
    <div class="metrics-grid">
      <div class="metric"><div class="metric-value">{req_total}</div><div class="metric-label">Total Requests</div></div>
      <div class="metric"><div class="metric-value">{rpm:.1}</div><div class="metric-label">Req / Minute</div></div>
      <div class="metric"><div class="metric-value">{avg_ms:.1}ms</div><div class="metric-label">Avg Response</div></div>
      <div class="metric"><div class="metric-value">{p95_ms:.1}ms</div><div class="metric-label">p95 Response</div></div>
      <div class="metric"><div class="metric-value">{err_rate:.2}%</div><div class="metric-label">Error Rate</div></div>
      <div class="metric"><div class="metric-value">{ws_conn}</div><div class="metric-label">WS Connections</div></div>
      <div class="metric"><div class="metric-value">{events}</div><div class="metric-label">Events Indexed</div></div>
      <div class="metric"><div class="metric-value">{ledger}</div><div class="metric-label">Last Ledger</div></div>
    </div>
  </div>

  <div class="card">
    <h2>Active Alerts</h2>
    {alerts_html}
  </div>

  <footer>Auto-refreshes every 30s &nbsp;·&nbsp; <a href="/health/ready" style="color:#6b7280">/health/ready</a> &nbsp;·&nbsp; <a href="/health/metrics" style="color:#6b7280">/health/metrics</a></footer>
</div>
<script>setTimeout(()=>location.reload(),30000)</script>
</body>
</html>"#,
        version = health.version,
        uptime = health.uptime_seconds,
        timestamp = health.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        status_color = status_color,
        status_label = status_label,
        components_html = components_html,
        req_total = m.requests_total,
        rpm = m.requests_per_minute,
        avg_ms = m.avg_response_ms,
        p95_ms = m.p95_response_ms,
        err_rate = m.error_rate * 100.0,
        ws_conn = m.active_ws_connections,
        events = m.events_indexed_total,
        ledger = m
            .last_ledger_processed
            .map(|l| l.to_string())
            .unwrap_or_else(|| "—".to_string()),
        alerts_html = alerts_html,
    );

    axum::response::Html(html)
}
