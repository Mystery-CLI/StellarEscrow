use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, Response},
    middleware::Next,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;

use crate::database::Database;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStats {
    pub route: String,
    pub requests: u64,
    pub errors: u64,
    pub avg_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: String,
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub threshold: f64,
    pub observed: f64,
    pub triggered_at: chrono::DateTime<Utc>,
    pub resolved_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDashboard {
    pub generated_at: chrono::DateTime<Utc>,
    pub overall: OverallStats,
    pub routes: Vec<RouteStats>,
    pub active_alerts: Vec<PerformanceAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallStats {
    pub total_requests: u64,
    pub total_errors: u64,
    pub error_rate: f64,
    pub avg_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub requests_per_minute: f64,
}

// ---------------------------------------------------------------------------
// Alert rules
// ---------------------------------------------------------------------------

struct AlertRule {
    name: &'static str,
    severity: &'static str,
    threshold: f64,
    check: fn(&OverallStats) -> Option<f64>, // returns observed value if breached
    message: &'static str,
}

const ALERT_RULES: &[AlertRule] = &[
    AlertRule {
        name: "high_error_rate",
        severity: "warning",
        threshold: 0.05,
        check: |s| (s.error_rate > 0.05).then_some(s.error_rate),
        message: "Error rate exceeds 5%",
    },
    AlertRule {
        name: "critical_error_rate",
        severity: "critical",
        threshold: 0.20,
        check: |s| (s.error_rate > 0.20).then_some(s.error_rate),
        message: "Error rate exceeds 20%",
    },
    AlertRule {
        name: "high_p95_latency",
        severity: "warning",
        threshold: 1000.0,
        check: |s| (s.p95_ms > 1000.0).then_some(s.p95_ms),
        message: "P95 latency exceeds 1000ms",
    },
    AlertRule {
        name: "critical_p95_latency",
        severity: "critical",
        threshold: 5000.0,
        check: |s| (s.p95_ms > 5000.0).then_some(s.p95_ms),
        message: "P95 latency exceeds 5000ms",
    },
];

// ---------------------------------------------------------------------------
// In-memory sample store
// ---------------------------------------------------------------------------

#[derive(Default)]
struct RouteWindow {
    durations_ms: Vec<u64>,
    errors: u64,
}

impl RouteWindow {
    fn record(&mut self, duration_ms: u64, is_error: bool) {
        if self.durations_ms.len() >= 2000 {
            self.durations_ms.remove(0);
        }
        self.durations_ms.push(duration_ms);
        if is_error {
            self.errors += 1;
        }
    }

    fn stats(&self, route: &str) -> RouteStats {
        let requests = self.durations_ms.len() as u64;
        let avg_ms = if requests == 0 {
            0.0
        } else {
            self.durations_ms.iter().sum::<u64>() as f64 / requests as f64
        };
        let (p95_ms, p99_ms) = percentiles(&self.durations_ms);
        RouteStats {
            route: route.to_string(),
            requests,
            errors: self.errors,
            avg_ms,
            p95_ms,
            p99_ms,
        }
    }
}

fn percentiles(samples: &[u64]) -> (f64, f64) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let p95 = sorted[(sorted.len() as f64 * 0.95) as usize].min(*sorted.last().unwrap()) as f64;
    let p99 = sorted[(sorted.len() as f64 * 0.99) as usize].min(*sorted.last().unwrap()) as f64;
    (p95, p99)
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct PerformanceService {
    db: Arc<Database>,
    windows: Arc<RwLock<HashMap<String, RouteWindow>>>,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    started_at: Instant,
    total_requests: Arc<RwLock<u64>>,
}

impl PerformanceService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            windows: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            started_at: Instant::now(),
            total_requests: Arc::new(RwLock::new(0)),
        }
    }

    /// Record a single request observation.
    pub async fn record(&self, route: &str, method: &str, status: u16, duration_ms: u64) {
        let is_error = status >= 500;
        {
            let mut windows = self.windows.write().await;
            windows
                .entry(format!("{} {}", method, route))
                .or_default()
                .record(duration_ms, is_error);
            *self.total_requests.write().await += 1;
        }
        // Persist to DB (fire-and-forget)
        let db = self.db.clone();
        let route = route.to_string();
        let method = method.to_string();
        tokio::spawn(async move {
            if let Err(e) = db
                .insert_perf_sample(&route, &method, status, duration_ms, is_error)
                .await
            {
                warn!("Failed to persist perf sample: {}", e);
            }
        });
    }

    /// Build the full performance dashboard.
    pub async fn dashboard(&self) -> PerformanceDashboard {
        let windows = self.windows.read().await;
        let mut routes: Vec<RouteStats> = windows
            .iter()
            .map(|(k, w)| w.stats(k))
            .collect();
        routes.sort_by(|a, b| b.requests.cmp(&a.requests));

        let all_durations: Vec<u64> = windows
            .values()
            .flat_map(|w| w.durations_ms.iter().copied())
            .collect();
        let total_requests = all_durations.len() as u64;
        let total_errors: u64 = windows.values().map(|w| w.errors).sum();
        let avg_ms = if total_requests == 0 {
            0.0
        } else {
            all_durations.iter().sum::<u64>() as f64 / total_requests as f64
        };
        let (p95_ms, p99_ms) = percentiles(&all_durations);
        let elapsed_mins = self.started_at.elapsed().as_secs_f64() / 60.0;
        let requests_per_minute = if elapsed_mins > 0.0 {
            total_requests as f64 / elapsed_mins
        } else {
            0.0
        };
        let error_rate = if total_requests == 0 {
            0.0
        } else {
            total_errors as f64 / total_requests as f64
        };

        let overall = OverallStats {
            total_requests,
            total_errors,
            error_rate,
            avg_ms,
            p95_ms,
            p99_ms,
            requests_per_minute,
        };

        // Evaluate alert rules
        self.evaluate_alerts(&overall).await;

        let active_alerts = self
            .alerts
            .read()
            .await
            .iter()
            .filter(|a| a.resolved_at.is_none())
            .cloned()
            .collect();

        PerformanceDashboard {
            generated_at: Utc::now(),
            overall,
            routes,
            active_alerts,
        }
    }

    async fn evaluate_alerts(&self, stats: &OverallStats) {
        let mut alerts = self.alerts.write().await;

        for rule in ALERT_RULES {
            if let Some(observed) = (rule.check)(stats) {
                let already = alerts
                    .iter()
                    .any(|a| a.rule_name == rule.name && a.resolved_at.is_none());
                if !already {
                    warn!(
                        rule = rule.name,
                        severity = rule.severity,
                        observed,
                        threshold = rule.threshold,
                        "Performance alert triggered"
                    );
                    alerts.push(PerformanceAlert {
                        id: Uuid::new_v4().to_string(),
                        rule_name: rule.name.to_string(),
                        severity: rule.severity.to_string(),
                        message: rule.message.to_string(),
                        threshold: rule.threshold,
                        observed,
                        triggered_at: Utc::now(),
                        resolved_at: None,
                    });
                    // Persist alert
                    let db = self.db.clone();
                    let rule_name = rule.name.to_string();
                    let severity = rule.severity.to_string();
                    let message = rule.message.to_string();
                    let threshold = rule.threshold;
                    tokio::spawn(async move {
                        if let Err(e) = db
                            .insert_perf_alert(&rule_name, &severity, &message, threshold, observed)
                            .await
                        {
                            warn!("Failed to persist perf alert: {}", e);
                        }
                    });
                }
            } else {
                // Auto-resolve
                for alert in alerts.iter_mut() {
                    if alert.rule_name == rule.name && alert.resolved_at.is_none() {
                        alert.resolved_at = Some(Utc::now());
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Axum middleware
// ---------------------------------------------------------------------------

pub async fn perf_middleware(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let method = req.method().to_string();
    let start = Instant::now();

    let response = next.run(req).await;

    let duration_ms = start.elapsed().as_millis() as u64;
    let status = response.status().as_u16();

    // Log slow requests
    if duration_ms > 1000 {
        warn!(route = %route, method = %method, duration_ms, status, "Slow request");
    }

    // Attach timing header
    let mut response = response;
    if let Ok(v) = duration_ms.to_string().parse() {
        response.headers_mut().insert("x-response-time-ms", v);
    }

    // We can't easily access AppState here without cloning Arc into extensions.
    // Recording is done via the state-aware handler wrapper instead.
    let _ = (route, method, status, duration_ms);

    response
}
