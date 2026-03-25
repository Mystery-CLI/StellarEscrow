use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Response},
    response::Json,
    response::Response,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::health::HealthState;
use crate::models::{
    Event, EventQuery, EventStats, IndexerStatus, PaginatedResponse, ReplayRequest, StatsResponse,
    WebSocketMessage,
};
use crate::websocket::WebSocketManager;
use crate::database::Database;
use crate::models::{EventQuery, PagedResponse, ReplayRequest, WebSocketMessage};
use crate::models::{
    AuditQuery, DiscoveryQuery, Event, EventQuery, GlobalSearchQuery, GlobalSearchResponse,
    HistoryQuery, NewAuditLog, PagedResponse, ReplayRequest, RetentionRequest, RetentionResponse,
    SuggestionQuery, TradeSearchQuery, WebSocketMessage,
};
use crate::websocket::WebSocketManager;
use crate::database::Database;
use crate::{database::Database, models::Event, models::PagedResponse};
use crate::fraud_service::FraudDetectionService;
use crate::{database::Database, models::Event};

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
            "replay":          "POST /events/replay  {from_ledger, to_ledger?}",
            "websocket":       "GET  /ws",
            "help":            "GET  /help",
            "audit_ingest":    "POST /audit  {actor, category, action, outcome, ...}",
            "audit_query":     "GET  /audit?actor=&category=&action=&outcome=&severity=&from=&to=&limit=&offset=",
            "audit_stats":     "GET  /audit/stats",
            "audit_purge":     "DELETE /audit/purge  {older_than_days?}"
            "search":          "GET  /search?q=&limit=",
            "search_trades":   "GET  /search/trades?q=&status=&seller=&buyer=&min_amount=&max_amount=&limit=&offset=",
            "search_discovery":"GET  /search/discovery?q=&role=&limit=",
            "search_suggestions":"GET /search/suggestions?q=&limit=",
            "search_history":  "GET  /search/history?limit=",
            "search_analytics":"GET  /search/analytics?from=&to=&search_type=",
            "fraud_review":    "POST /fraud/review  {trade_id, status, reviewer, notes}",
            "notif_prefs_get": "GET  /notifications/preferences/:address",
            "notif_prefs_put": "PUT  /notifications/preferences/:address  {email_enabled, email_address, sms_enabled, phone_number, push_enabled, push_token, on_*}",
            "notif_log":       "GET  /notifications/log/:address?limit=",
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
    let query = EventQuery { limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PaginatedResponse {
        has_more: offset + limit < total,
        items: events,
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
    let query = EventQuery { trade_id: Some(trade_id), limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse { has_more: offset + limit < total, items: events, total, limit, offset }))
}

pub async fn get_events_by_type(
    Path(event_type): Path<String>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<PagedResponse<Event>>, AppError> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);
    let query = EventQuery { event_type: Some(event_type), limit: Some(limit), offset: Some(offset), ..params };

    let (events, total) = tokio::try_join!(
        state.database.get_events(&query),
        state.database.count_events(&query),
    )?;

    Ok(Json(PagedResponse { has_more: offset + limit < total, items: events, total, limit, offset }))
}

pub async fn replay_events(
    State(state): State<AppState>,
    Json(request): Json<ReplayRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let to_ledger = request.to_ledger.unwrap_or(i64::MAX);
    let events = state.database.get_events_in_range(request.from_ledger, to_ledger, "contract_id").await?;

    for event in &events {
        let ws_message = WebSocketMessage {
            event_type: event.event_type.clone(),
            data: event.data.clone(),
            timestamp: event.timestamp,
        };
        state.ws_manager.broadcast(ws_message).await;
    }

    Ok(Json(json!({ "replayed": events.len(), "from_ledger": request.from_ledger, "to_ledger": request.to_ledger })))
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| state.ws_manager.handle_connection(socket))
}

/// GET /status — indexer sync state for loading indicators.
pub async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<IndexerStatus>, AppError> {
    let (total_events, latest) = tokio::try_join!(
        state.database.get_event_count(None),
        state.database.get_latest_ledger_global(),
    )?;

    let (latest_ledger, latest_ledger_time) = match latest {
        Some((l, t)) => (Some(l), Some(t)),
        None => (None, None),
    };

    Ok(Json(IndexerStatus {
        syncing: true, // always true while the monitor is running
        latest_ledger,
        latest_ledger_time,
        total_events,
        server_time: chrono::Utc::now(),
    }))
}

/// GET /stats — per-event-type counts for dashboard skeleton panels.
pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>, AppError> {
    let (total_events, type_counts) = tokio::try_join!(
        state.database.get_event_count(None),
        state.database.get_event_type_counts(),
    )?;

    let by_type = type_counts
        .into_iter()
        .map(|(event_type, count)| EventStats { event_type, count })
        .collect();

    Ok(Json(StatsResponse { total_events, by_type }))
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
    let rows = state.database.search_trades(&params).await?;
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
    state.database.update_fraud_review(
        payload.trade_id,
        &payload.status,
        &payload.reviewer,
        &payload.notes,
    ).await?;
    
    Ok(Json(json!({ "status": "updated", "trade_id": payload.trade_id })))
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub ws_manager: Arc<WebSocketManager>,
    pub health: HealthState,
    pub fraud_service: Arc<FraudDetectionService>,
    pub notification_service: Arc<NotificationService>,
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
    let q = AuditQuery { limit: Some(limit), offset: Some(offset), ..params };

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
    Ok(Json(RetentionResponse { deleted, older_than_days: days }))
}

// =============================================================================
// Notification Handlers
// =============================================================================

/// GET /notifications/preferences/:address
pub async fn get_notification_preferences(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<crate::models::NotificationPreferences>, AppError> {
    let prefs = state.database.get_notification_preferences(&address).await?
        .ok_or_else(|| AppError::NotFound("preferences not found".into()))?;
    Ok(Json(prefs))
}

/// PUT /notifications/preferences/:address
pub async fn upsert_notification_preferences(
    Path(address): Path<String>,
    State(state): State<AppState>,
    Json(body): Json<crate::models::UpdateNotificationPreferences>,
) -> Result<Json<crate::models::NotificationPreferences>, AppError> {
    let prefs = state.database.upsert_notification_preferences(&address, &body).await?;
    Ok(Json(prefs))
}

/// GET /notifications/log/:address
pub async fn get_notification_log(
    Path(address): Path<String>,
    Query(params): Query<crate::models::HistoryQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::NotificationLogEntry>>, AppError> {
    let entries = state.database.get_notification_log(&address, params.limit.unwrap_or(50)).await?;
    Ok(Json(entries))
}
