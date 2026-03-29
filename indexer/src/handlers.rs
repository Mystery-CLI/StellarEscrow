use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Response},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::analytics_service::AnalyticsService;
use crate::backup_service::BackupService;
use crate::cache_service::CacheService;
use crate::compliance_service::{ComplianceService, ComplianceStatus};
use crate::database::Database;
use crate::error::AppError;
use crate::fraud_service::FraudDetectionService;
use crate::health::HealthState;
use crate::job_queue::types::{Job, JobPriority, JobType};
use crate::job_queue::worker::JobWorker;
use crate::job_queue::JobQueue;
use crate::webhook_service::WebhookService;
use crate::monitoring_service::{MonitoringService, dashboard};
use crate::models::{
    AuditQuery, DiscoveryQuery, Event, EventQuery, EventStats, GlobalSearchQuery,
    GlobalSearchResponse, HistoryQuery, IndexerStatus, NewAuditLog, PagedResponse,
    PaginatedResponse, ReplayRequest, RetentionRequest, RetentionResponse, StatsResponse,
    SuggestionQuery, TradeSearchQuery, WebSocketMessage,
};
use crate::websocket::WebSocketManager;

/// Default page size — kept small for mobile clients.
const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 100;

/// GET / — API discovery / navigation index.
pub async fn api_index() -> Json<serde_json::Value> {
    Json(json!({
        "name": "StellarEscrow Indexer API",
        "version": "1.0.0",
        "endpoints": {
            "health_live":     "GET  /health/live",
            "health_ready":    "GET  /health/ready",
            "health_metrics":  "GET  /health/metrics",
            "health_alerts":   "GET  /health/alerts",
            "status_page":     "GET  /status",
            "events":          "GET  /events?limit=20&offset=0&event_type=&trade_id=&from_ledger=&to_ledger=",
            "event_by_id":     "GET  /events/:id",
            "events_by_trade": "GET  /events/trade/:trade_id",
            "events_by_type":  "GET  /events/type/:event_type",
            "replay":          "POST /events/replay",
            "websocket":       "GET  /ws",
            "help":            "GET  /help",
            "audit_ingest":    "POST /audit",
            "audit_query":     "GET  /audit",
            "audit_stats":     "GET  /audit/stats",
            "audit_purge":     "DELETE /audit/purge",
            "search":          "GET  /search",
            "search_trades":   "GET  /search/trades",
            "search_discovery":"GET  /search/discovery",
            "search_suggestions":"GET /search/suggestions",
            "search_history":  "GET  /search/history",
            "search_analytics":"GET  /search/analytics",
            "cache_stats":     "GET  /cache/stats",
            "cache_invalidate":"POST /cache/invalidate",
            "cache_warm":      "POST /cache/warm",
            "jobs_stats":      "GET  /jobs/stats",
            "jobs_enqueue":    "POST /jobs/enqueue",
            "jobs_schedule":   "POST /jobs/schedule",
            "fraud_review":    "POST /fraud/review",
            "notif_prefs_get": "GET  /notifications/preferences/:address",
            "notif_prefs_put": "PUT  /notifications/preferences/:address",
            "notif_log":       "GET  /notifications/log/:address",
            "push_register":   "POST /push/register",
            "push_unregister": "DELETE /push/unregister/:device_token",
            "help":            "GET  /help"
        }
    }))
}

pub async fn api_docs() -> Json<serde_json::Value> {
    Json(json!({
        "api_name": "StellarEscrow Indexer API",
        "api_version": "v1",
        "base_url": "/api/v1",
        "authentication": {
            "scheme": "API_KEY",
            "headers": ["Authorization: Bearer <API_KEY>", "x-api-key: <API_KEY>"],
            "admin_key": "for /admin endpoints"
        },
        "rate_limit": {
            "default_rpm": "config.rate_limit.default_rpm",
            "elevated_rpm": "config.rate_limit.elevated_rpm",
            "admin_rpm": "config.rate_limit.admin_rpm"
        }
    }))
}

pub async fn get_events(
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery {
        limit: Some(limit),
        offset: Some(offset),
        ..params
    };
    let cache_key = CacheService::events_query_key(&query);
    let events = state
        .cache_service
        .get_or_load(&cache_key, state.cache_service.ttl_events(), || async {
            state.database.get_events(&query).await
        })
        .await?;
    let total = state.database.count_events(&query).await?;

    Ok(Json(PaginatedResponse {
        has_more: offset + limit < total,
        data: events,
        total,
        limit,
        offset,
    }))
}

pub async fn get_event_by_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Event>, AppError> {
    let event = state.database.get_event_by_id(id).await?;
    Ok(Json(event))
}

pub async fn get_events_by_trade_id(
    Path(trade_id): Path<u64>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery {
        trade_id: Some(trade_id),
        limit: Some(limit),
        offset: Some(offset),
        ..params
    };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse {
        has_more: offset + limit < total,
        items: events,
        total,
        limit,
        offset,
    }))
}

pub async fn get_events_by_type(
    Path(event_type): Path<String>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery {
        event_type: Some(event_type),
        limit: Some(limit),
        offset: Some(offset),
        ..params
    };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse {
        has_more: offset + limit < total,
        items: events,
        total,
        limit,
        offset,
    }))
}

pub async fn replay_events(
    State(state): State<AppState>,
    Json(request): Json<ReplayRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let to_ledger = request.to_ledger.unwrap_or(i64::MAX);
    let events = state
        .database
        .get_events_in_range(request.from_ledger, to_ledger, "contract_id")
        .await?;

    for event in &events {
        let ws_message = WebSocketMessage {
            event_type: event.event_type.clone(),
            category: event.category.clone(),
            version: event.schema_version as u32,
            data: event.data.clone(),
            timestamp: event.timestamp,
        };
        state.ws_manager.broadcast(ws_message).await;
    }

    state
        .cache_service
        .invalidate_pattern("events:list:*")
        .await;
    state.cache_service.invalidate(crate::cache_service::KEY_STATS).await;
    state
        .cache_service
        .invalidate(crate::cache_service::KEY_INDEXER_STATUS)
        .await;

    Ok(Json(
        json!({ "replayed": events.len(), "from_ledger": request.from_ledger, "to_ledger": request.to_ledger }),
    ))
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| state.ws_manager.handle_connection(socket))
}

/// GET /status — indexer sync state for loading indicators.
pub async fn get_status(State(state): State<AppState>) -> Result<Json<IndexerStatus>, AppError> {
    let status = state
        .cache_service
        .get_or_load(
            crate::cache_service::KEY_INDEXER_STATUS,
            state.cache_service.ttl_default(),
            || async {
                let (total_events, latest) = tokio::try_join!(
                    state.database.get_event_count(None),
                    state.database.get_latest_ledger_global(),
                )?;

                let (latest_ledger, latest_ledger_time) = match latest {
                    Some((l, t)) => (Some(l), Some(t)),
                    None => (None, None),
                };

                Ok(IndexerStatus {
                    syncing: true,
                    latest_ledger,
                    latest_ledger_time,
                    total_events,
                    server_time: chrono::Utc::now(),
                })
            },
        )
        .await?;

    Ok(Json(status))
}

/// GET /stats — per-event-type counts for dashboard skeleton panels.
pub async fn get_stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, AppError> {
    let stats = state
        .cache_service
        .get_or_load(crate::cache_service::KEY_STATS, state.cache_service.ttl_default(), || async {
            let (total_events, type_counts) = tokio::try_join!(
                state.database.get_event_count(None),
                state.database.get_event_type_counts(),
            )?;

            let by_type = type_counts
                .into_iter()
                .map(|(event_type, count)| EventStats { event_type, count })
                .collect();

            Ok(StatsResponse {
                total_events,
                by_type,
            })
        })
        .await?;

    Ok(Json(stats))
}

pub async fn global_search(
    Query(params): Query<GlobalSearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<GlobalSearchResponse>, AppError> {
    let limit = params.limit.unwrap_or(10).clamp(1, 50);
    let trade_query = TradeSearchQuery {
        q: Some(params.q.clone()),
        status: None,
        seller: None,
        buyer: None,
        min_amount: None,
        max_amount: None,
        limit: Some(limit),
        offset: Some(0),
    };

    let user_query = DiscoveryQuery {
        q: Some(params.q.clone()),
        role: Some("user".to_string()),
        limit: Some(limit),
    };
    let arb_query = DiscoveryQuery {
        q: Some(params.q.clone()),
        role: Some("arbitrator".to_string()),
        limit: Some(limit),
    };

    let trades = state.database.search_trades(&trade_query).await?;
    let users = state.database.discover_entities(&user_query).await?;
    let arbitrators = state.database.discover_entities(&arb_query).await?;
    let suggestions = state.database.get_search_suggestions(&params.q, 10).await?;

    state.database.record_search(&params.q, "global").await?;

    Ok(Json(GlobalSearchResponse {
        trades,
        users,
        arbitrators,
        suggestions,
    }))
}

pub async fn search_trades(
    Query(params): Query<TradeSearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::TradeSearchResult>>, AppError> {
    let cache_key = CacheService::search_trades_key(&params);
    let rows = state
        .cache_service
        .get_or_load(&cache_key, state.cache_service.ttl_default(), || async {
            state.database.search_trades(&params).await
        })
        .await?;
    if let Some(q) = params.q {
        if !q.is_empty() {
            state.database.record_search(&q, "trades").await?;
        }
    }
    Ok(Json(rows))
}

pub async fn discover_entities(
    Query(params): Query<DiscoveryQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::DiscoveryResult>>, AppError> {
    let rows = state.database.discover_entities(&params).await?;
    if let Some(q) = params.q {
        if !q.is_empty() {
            state.database.record_search(&q, "discovery").await?;
        }
    }
    Ok(Json(rows))
}

pub async fn search_suggestions(
    Query(params): Query<SuggestionQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::SearchSuggestion>>, AppError> {
    let rows = state
        .database
        .get_search_suggestions(&params.q, params.limit.unwrap_or(10))
        .await?;
    Ok(Json(rows))
}

pub async fn search_history(
    Query(params): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::SearchHistoryEntry>>, AppError> {
    let rows = state
        .database
        .get_search_history(params.limit.unwrap_or(20))
        .await?;
    Ok(Json(rows))
}

pub async fn search_analytics(
    Query(params): Query<crate::models::SearchAnalyticsQuery>,
    State(state): State<AppState>,
) -> Result<Json<crate::models::SearchAnalyticsResponse>, AppError> {
    let result = state.database.get_search_analytics(&params).await?;
    Ok(Json(result))
}

pub async fn get_fraud_alerts(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let alerts = state.database.get_fraud_alerts().await?;
    Ok(Json(alerts))
}

#[derive(Deserialize)]
pub struct FraudReviewRequest {
    pub trade_id: u64,
    pub status: String,
    pub reviewer: String,
    pub notes: String,
}

pub async fn update_fraud_review(
    State(state): State<AppState>,
    Json(payload): Json<FraudReviewRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    state
        .database
        .update_fraud_review(
            payload.trade_id,
            &payload.status,
            &payload.reviewer,
            &payload.notes,
        )
        .await?;

    Ok(Json(
        json!({ "status": "updated", "trade_id": payload.trade_id }),
    ))
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub stellar_contract_id: String,
    pub job_queue: Arc<tokio::sync::Mutex<JobQueue>>,
    pub job_worker: Arc<JobWorker>,
    pub ws_manager: Arc<WebSocketManager>,
    pub health: HealthState,
    pub fraud_service: Arc<FraudDetectionService>,
    pub notification_service: Arc<crate::notification_service::NotificationService>,
    pub gateway: Arc<crate::gateway::GatewayState>,
    pub performance_service: Arc<crate::performance_service::PerformanceService>,
    pub integration_service: Arc<crate::integration_service::IntegrationService>,
    pub compliance_service: Arc<crate::compliance_service::ComplianceService>,
    pub monitoring_service: Arc<crate::monitoring_service::MonitoringService>,
    pub analytics_service: Arc<AnalyticsService>,
    pub cache_service: Arc<CacheService>,
    pub backup_service: Arc<BackupService>,
    pub webhook_service: Arc<WebhookService>,
    pub compliance_service: Arc<ComplianceService>,
    pub monitoring_service: Arc<MonitoringService>,
}

// =============================================================================
// Audit Log Handlers
// =============================================================================

/// POST /audit — ingest a new audit log entry.
pub async fn create_audit_log(
    State(state): State<AppState>,
    Json(body): Json<NewAuditLog>,
) -> Result<Json<crate::models::AuditLog>, AppError> {
    let log = state.database.insert_audit_log(&body).await?;
    Ok(Json(log))
}

/// GET /audit — query audit logs with optional filters.
pub async fn query_audit_logs(
    Query(params): Query<AuditQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<crate::models::AuditLog>>, AppError> {
    let limit = params.limit.unwrap_or(50).clamp(1, 500);
    let offset = params.offset.unwrap_or(0).max(0);
    let q = AuditQuery {
        limit: Some(limit),
        offset: Some(offset),
        ..params
    };

    let (logs, total) = tokio::try_join!(
        state.database.query_audit_logs(&q),
        state.database.count_audit_logs(&q),
    )?;

    Ok(Json(PagedResponse {
        has_more: offset + limit < total,
        items: logs,
        total,
        limit,
        offset,
    }))
}

/// GET /audit/stats — aggregated analysis (counts by category, outcome, severity, top actors/actions).
pub async fn audit_stats(
    State(state): State<AppState>,
) -> Result<Json<crate::models::AuditStats>, AppError> {
    let stats = state.database.audit_stats().await?;
    Ok(Json(stats))
}

/// DELETE /audit/purge — apply retention policy, deleting logs older than N days.
pub async fn purge_audit_logs(
    State(state): State<AppState>,
    Json(body): Json<RetentionRequest>,
) -> Result<Json<RetentionResponse>, AppError> {
    let days = body.older_than_days.unwrap_or(90).clamp(1, 365);
    let deleted = state.database.purge_old_audit_logs(days).await?;
    Ok(Json(RetentionResponse {
        deleted,
        older_than_days: days,
    }))
}

// =============================================================================
// Notification Handlers
// =============================================================================

/// GET /notifications/preferences/:address
pub async fn get_notification_preferences(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::models::NotificationPreferences>, AppError> {
    let prefs = state
        .database
        .get_notification_preferences(&address)
        .await?
        .unwrap_or_else(|| crate::models::NotificationPreferences::default_for_address(address));
    Ok(Json(prefs))
}

/// PUT /notifications/preferences/:address
pub async fn upsert_notification_preferences(
    Path(address): Path<String>,
    State(state): State<AppState>,
    Json(body): Json<crate::models::UpdateNotificationPreferences>,
) -> Result<Json<crate::models::NotificationPreferences>, AppError> {
    let prefs = state
        .database
        .upsert_notification_preferences(&address, &body)
        .await?;
    Ok(Json(prefs))
}

/// GET /notifications/log/:address
pub async fn get_notification_log(
    Path(address): Path<String>,
    Query(params): Query<crate::models::HistoryQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::NotificationLogEntry>>, AppError> {
    let entries = state
        .database
        .get_notification_log(&address, params.limit.unwrap_or(50))
        .await?;
    Ok(Json(entries))
}

/// POST /push/register
pub async fn register_push_token(
    State(state): State<AppState>,
    Json(body): Json<crate::models::PushRegistrationRequest>,
) -> Result<Json<crate::models::NotificationPreferences>, AppError> {
    let token = body.device_token.trim();
    let address = body.address.trim();

    if token.is_empty() || address.is_empty() {
        return Err(AppError::BadRequest(
            "address and device_token are required".to_string(),
        ));
    }

    let prefs = state
        .database
        .upsert_notification_preferences(
            address,
            &crate::models::UpdateNotificationPreferences {
                email_enabled: None,
                email_address: None,
                sms_enabled: None,
                phone_number: None,
                push_enabled: Some(true),
                push_token: Some(token.to_string()),
                on_trade_created: None,
                on_trade_funded: None,
                on_trade_completed: None,
                on_trade_confirmed: None,
                on_dispute_raised: None,
                on_dispute_resolved: None,
                on_trade_cancelled: None,
            },
        )
        .await?;

    Ok(Json(prefs))
}

/// DELETE /push/unregister/:device_token
pub async fn unregister_push_token(
    Path(device_token): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let removed = state
        .database
        .unregister_push_token(device_token.trim())
        .await?;

    Ok(Json(json!({
        "status": "ok",
        "removed": removed,
    })))
}

// =============================================================================
// Gateway Handlers
// =============================================================================

/// GET /api/v1/gateway/stats - Return gateway statistics and status
pub async fn gateway_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = crate::gateway::get_gateway_stats(&state.gateway);
    Ok(Json(
        serde_json::to_value(stats).map_err(|_| AppError::InternalServerError)?,
    ))
}

// =============================================================================
// Integration Service Handlers
// =============================================================================

/// GET /integrations/stats — connector monitoring stats
pub async fn get_integration_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let stats = state.integration_service.get_stats().await;
    Json(serde_json::to_value(stats).unwrap_or_default())
}

/// GET /integrations/log?connector_id=&limit= — delivery log
pub async fn get_integration_log(
    Query(params): Query<IntegrationLogQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::integration_service::DeliveryRecord>>, AppError> {
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let records = state
        .integration_service
        .get_delivery_log(params.connector_id.as_deref(), limit)
        .await
        .map_err(AppError::Database)?;
    Ok(Json(records))
}

#[derive(serde::Deserialize)]
pub struct IntegrationLogQuery {
    pub connector_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct EnqueueJobRequest {
    pub job_type: String,
    pub event_id: Option<String>,
    pub trade_id: Option<String>,
    pub priority: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub run_at: Option<i64>,
    pub max_attempts: Option<u32>,
}

/// GET /jobs/stats — job queue depths and worker status.
pub async fn get_job_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let snapshot = state.job_worker.snapshot().await?;
    Ok(Json(serde_json::to_value(&snapshot).unwrap_or_default()))
}

/// POST /jobs/enqueue — enqueue an immediate background job.
pub async fn enqueue_job(
    State(state): State<AppState>,
    Json(body): Json<EnqueueJobRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let job_type = parse_job_type(&body.job_type)?;
    let priority = parse_job_priority(body.priority.as_deref())?;
    let mut job = Job::new(
        job_type,
        body.event_id.unwrap_or_else(|| "manual".to_string()),
        body.trade_id.unwrap_or_else(|| "manual".to_string()),
        body.payload.unwrap_or_default(),
        priority,
    );
    if let Some(max_attempts) = body.max_attempts {
        job = job.with_max_attempts(max_attempts.max(1));
    }

    let mut queue = state.job_queue.lock().await;
    queue.enqueue(job.clone()).await?;

    Ok(Json(json!({
        "status": "queued",
        "job_id": job.id,
        "priority": job.priority.as_str(),
    })))
}

/// POST /jobs/schedule — enqueue a delayed background job.
pub async fn schedule_job(
    State(state): State<AppState>,
    Json(body): Json<EnqueueJobRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let run_at = body
        .run_at
        .ok_or_else(|| AppError::BadRequest("run_at is required".to_string()))?;
    let job_type = parse_job_type(&body.job_type)?;
    let priority = parse_job_priority(body.priority.as_deref())?;
    let mut job = Job::new(
        job_type,
        body.event_id.unwrap_or_else(|| "scheduled".to_string()),
        body.trade_id.unwrap_or_else(|| "scheduled".to_string()),
        body.payload.unwrap_or_default(),
        priority,
    )
    .scheduled_at(run_at);
    if let Some(max_attempts) = body.max_attempts {
        job = job.with_max_attempts(max_attempts.max(1));
    }

    let mut queue = state.job_queue.lock().await;
    queue.enqueue_at(job.clone(), run_at).await?;

    Ok(Json(json!({
        "status": "scheduled",
        "job_id": job.id,
        "run_at": run_at,
        "priority": job.priority.as_str(),
    })))
}

// =============================================================================
// Performance Monitoring Handlers
// =============================================================================

/// GET /performance/dashboard — full APM dashboard (routes, stats, alerts).
pub async fn get_performance_dashboard(
    State(state): State<AppState>,
) -> Result<Json<crate::performance_service::PerformanceDashboard>, AppError> {
    Ok(Json(state.performance_service.dashboard().await))
}

/// GET /performance/alerts — active performance alerts.
pub async fn get_performance_alerts(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let dashboard = state.performance_service.dashboard().await;
    Json(serde_json::json!({
        "active": dashboard.active_alerts,
        "total": dashboard.active_alerts.len(),
    }))
}

// =============================================================================
// Analytics Handlers
// =============================================================================

/// GET /analytics/dashboard — full analytics dashboard.
pub async fn get_analytics_dashboard(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let dash = state.analytics_service.get_dashboard().await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(Json(serde_json::to_value(&dash).unwrap_or_default()))
}

/// GET /analytics/realtime — real-time in-memory stats (no DB).
pub async fn get_analytics_realtime(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let window = state.analytics_service.get_realtime().await;
    Json(serde_json::to_value(&window).unwrap_or_default())
}

#[derive(Deserialize)]
pub struct ExportQuery {
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub format: Option<String>,
}

/// GET /analytics/export — export analytics data as CSV or JSON.
pub async fn export_analytics(
    Query(params): Query<ExportQuery>,
    State(state): State<AppState>,
) -> Result<axum::response::Response<String>, AppError> {
    use crate::analytics_service::export::ExportFormat;
    use std::str::FromStr;

    let from = params.from.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to   = params.to.unwrap_or_else(chrono::Utc::now);
    let fmt  = params.format.as_deref().and_then(|s| ExportFormat::from_str(s).ok())
        .unwrap_or(ExportFormat::Json);

    let is_csv = fmt == ExportFormat::Csv;
    let body = state.analytics_service.export(from, to, fmt).await
        .map_err(|_| AppError::InternalServerError)?;

    let content_type = if is_csv { "text/csv" } else { "application/json" };
    Ok(axum::response::Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Content-Disposition", if is_csv { "attachment; filename=analytics.csv" } else { "inline" })
        .body(body)
        .unwrap())
}

// =============================================================================
// Cache Handlers
// =============================================================================

/// GET /cache/stats — cache hit/miss statistics.
pub async fn get_cache_stats(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let snapshot = state.cache_service.snapshot().await;
    Json(serde_json::to_value(&snapshot).unwrap_or_default())
}

#[derive(Deserialize)]
pub struct CacheInvalidateRequest {
    pub key: Option<String>,
    pub pattern: Option<String>,
}

/// POST /cache/invalidate — manually invalidate a cache key or pattern.
pub async fn invalidate_cache(
    State(state): State<AppState>,
    Json(body): Json<CacheInvalidateRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(key) = body.key {
        state.cache_service.invalidate(&key).await;
        return Ok(Json(json!({ "invalidated": key })));
    }
    if let Some(pattern) = body.pattern {
        state.cache_service.invalidate_pattern(&pattern).await;
        return Ok(Json(json!({ "invalidated_pattern": pattern })));
    }
    Err(AppError::NotFound("Provide key or pattern".to_string()))
}

/// POST /cache/warm - trigger cache warming for core hot keys.
pub async fn warm_cache(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let report = state
        .cache_service
        .warm_core(&state.database, &state.stellar_contract_id)
        .await?;
    Ok(Json(serde_json::to_value(&report).unwrap_or_default()))
}

// =============================================================================
// Backup Handlers
// =============================================================================

/// POST /backup/trigger — trigger an immediate backup.
pub async fn trigger_backup(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let record = state.backup_service.run_backup().await;
    Ok(Json(serde_json::to_value(&record).unwrap_or_default()))
}

/// GET /backup/status — backup monitoring snapshot.
pub async fn get_backup_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let snapshot = state.backup_service.get_monitor_snapshot().await;
    Json(serde_json::to_value(&snapshot).unwrap_or_default())
}

/// GET /backup/history — list recent backup records.
pub async fn get_backup_history(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let records = state.backup_service.list_backups(20).await;
    Json(serde_json::to_value(&records).unwrap_or_default())
}

/// GET /backup/recovery-plan — generate a recovery procedure document.
pub async fn get_recovery_plan(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let snapshot = state.backup_service.get_monitor_snapshot().await;
    let location = snapshot.last_backup.as_ref().and_then(|b| b.location.as_deref());
    let plan = crate::backup_service::recovery::generate_recovery_plan(location);
    Json(serde_json::to_value(&plan).unwrap_or_default())
}

// =============================================================================
// Webhook Handlers
// =============================================================================

#[derive(Deserialize)]
pub struct RegisterWebhookRequest {
    pub url: String,
    pub secret: String,
    pub event_types: Option<Vec<String>>,
}

/// POST /webhooks — register a new webhook endpoint.
pub async fn register_webhook(
    State(state): State<AppState>,
    Json(body): Json<RegisterWebhookRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let endpoint = state.webhook_service
        .register(body.url, body.secret, body.event_types.unwrap_or_default())
        .await
        .map_err(|e| AppError::NotFound(e.to_string()))?;
    Ok(Json(serde_json::to_value(&endpoint).unwrap_or_default()))
}

/// GET /webhooks — list all registered webhook endpoints.
pub async fn list_webhooks(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let endpoints = state.webhook_service.get_endpoints().await;
    Json(serde_json::to_value(&endpoints).unwrap_or_default())
}

/// DELETE /webhooks/:id — deactivate a webhook endpoint.
pub async fn deactivate_webhook(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.webhook_service.deactivate(id).await
        .map_err(|e| AppError::NotFound(e.to_string()))?;
    Ok(Json(json!({ "deactivated": id })))
}

/// GET /webhooks/deliveries — recent delivery log.
pub async fn get_webhook_deliveries(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let log = state.webhook_service.get_delivery_log(50).await;
    Json(serde_json::to_value(&log).unwrap_or_default())
}

/// GET /webhooks/stats — webhook delivery statistics.
pub async fn get_webhook_stats(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let stats = state.webhook_service.get_stats().await;
    Json(serde_json::to_value(&stats).unwrap_or_default())
// Compliance Handlers
// =============================================================================

#[derive(Deserialize)]
pub struct ComplianceCheckQuery {
    pub address: String,
    pub trade_id: Option<u64>,
}

#[derive(Deserialize)]
pub struct ComplianceReviewRequest {
    pub check_id: Uuid,
    pub status: String,
    pub reviewer: String,
    pub notes: String,
}

#[derive(Deserialize)]
pub struct ComplianceReportQuery {
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
}

/// POST /compliance/check — run KYC + AML check for an address.
pub async fn run_compliance_check(
    State(state): State<AppState>,
    Json(payload): Json<ComplianceCheckQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let check = state
        .compliance_service
        .check_address(&payload.address, payload.trade_id)
        .await;
    let cache_key = CacheService::compliance_status_key(&payload.address);
    let value = serde_json::to_value(&check).unwrap_or_default();
    state
        .cache_service
        .set(&cache_key, &value, state.cache_service.ttl_default())
        .await;
    Ok(Json(value))
}

/// GET /compliance/status/:address — get latest compliance status for an address.
pub async fn get_compliance_status(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let cache_key = CacheService::compliance_status_key(&address);
    let response = state
        .cache_service
        .get_or_load(&cache_key, state.cache_service.ttl_default(), || async {
            let value = match state.compliance_service.get_address_status(&address).await {
                Some(check) => serde_json::to_value(&check).unwrap_or_default(),
                None => json!({ "status": "not_found", "address": address }),
            };
            Ok(value)
        })
        .await?;

    Ok(Json(response))
}

/// POST /compliance/review — manually review a compliance check.
pub async fn review_compliance_check(
    State(state): State<AppState>,
    Json(payload): Json<ComplianceReviewRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let status = match payload.status.as_str() {
        "approved" => ComplianceStatus::Approved,
        "rejected" => ComplianceStatus::Rejected,
        "blocked" => ComplianceStatus::Blocked,
        _ => ComplianceStatus::RequiresReview,
    };
    state
        .compliance_service
        .review_check(payload.check_id, status, &payload.reviewer, &payload.notes)
        .await
        .map_err(|e| AppError::NotFound(e.to_string()))?;
    state
        .cache_service
        .invalidate_pattern("compliance:*")
        .await;
    Ok(Json(json!({ "status": "updated", "check_id": payload.check_id })))
}

/// GET /compliance/report — generate compliance report for a date range.
pub async fn get_compliance_report(
    Query(params): Query<ComplianceReportQuery>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let from = params.from.unwrap_or_else(|| {
        chrono::Utc::now() - chrono::Duration::days(30)
    });
    let to = params.to.unwrap_or_else(chrono::Utc::now);
    let report = state
        .compliance_service
        .generate_report(from, to)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(Json(serde_json::to_value(&report).unwrap_or_default()))
}

// =============================================================================
// Monitoring Handlers
// =============================================================================

/// GET /monitoring/dashboard — full monitoring dashboard snapshot.
pub async fn get_monitoring_dashboard(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let snapshot = state.monitoring_service.get_snapshot().await;
    let grafana_url = None; // populated from config in production
    let dash = dashboard::build_dashboard(&snapshot, grafana_url);
    Ok(Json(serde_json::to_value(&dash).unwrap_or_default()))
}

/// GET /monitoring/alerts — list active alerts.
pub async fn get_monitoring_alerts(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let alerts = state.monitoring_service.get_active_alerts().await;
    Ok(Json(serde_json::to_value(&alerts).unwrap_or_default()))
}

/// GET /monitoring/metrics — Prometheus-format metrics.
pub async fn get_prometheus_metrics(
    State(state): State<AppState>,
) -> axum::response::Response<String> {
    let body = state.monitoring_service.prometheus_metrics();
    axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(body)
        .unwrap()
}

fn parse_job_type(value: &str) -> Result<JobType, AppError> {
    match value {
        "event" => Ok(JobType::Event),
        "notification" => Ok(JobType::Notification),
        "cache_warm" => Ok(JobType::CacheWarm),
        "compliance" => Ok(JobType::Compliance),
        _ => Err(AppError::BadRequest(format!("unsupported job_type '{}'", value))),
    }
}

fn parse_job_priority(value: Option<&str>) -> Result<JobPriority, AppError> {
    match value.unwrap_or("normal") {
        "critical" => Ok(JobPriority::Critical),
        "high" => Ok(JobPriority::High),
        "normal" => Ok(JobPriority::Normal),
        "low" => Ok(JobPriority::Low),
        other => Err(AppError::BadRequest(format!("unsupported priority '{}'", other))),
    }
}
