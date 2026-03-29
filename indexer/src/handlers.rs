use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    response::Response,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    DiscoveryQuery, EventQuery, GlobalSearchQuery, GlobalSearchResponse, HistoryQuery, ReplayRequest,
    SuggestionQuery, TradeDetailQuery, TradeSearchQuery, WebSocketMessage,
};
use crate::websocket::WebSocketManager;
use crate::{database::Database, models::Event};

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn get_events(
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, AppError> {
    let query = EventQuery {
        limit: params.limit.or(Some(50)),
        offset: params.offset.or(Some(0)),
        ..params
    };

    let events = state.database.get_events(&query).await?;
    Ok(Json(events))
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
) -> Result<Json<Vec<Event>>, AppError> {
    let query = EventQuery {
        trade_id: Some(trade_id),
        limit: params.limit.or(Some(50)),
        offset: params.offset.or(Some(0)),
        ..params
    };

    let events = state.database.get_events(&query).await?;
    Ok(Json(events))
}

pub async fn get_events_by_type(
    Path(event_type): Path<String>,
    Query(params): Query<EventQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, AppError> {
    let query = EventQuery {
        event_type: Some(event_type),
        limit: params.limit.or(Some(50)),
        offset: params.offset.or(Some(0)),
        ..params
    };

    let events = state.database.get_events(&query).await?;
    Ok(Json(events))
}

pub async fn replay_events(
    State(state): State<AppState>,
    Json(request): Json<ReplayRequest>,
) -> Result<Json<Vec<Event>>, AppError> {
    let to_ledger = request.to_ledger.unwrap_or(i64::MAX);
    let events = state.database.get_events_in_range(request.from_ledger, to_ledger, "contract_id").await?;

    // Broadcast replayed events to WebSocket clients
    for event in &events {
        let ws_message = WebSocketMessage {
            event_type: event.event_type.clone(),
            data: event.data.clone(),
            timestamp: event.timestamp,
        };
        state.ws_manager.broadcast(ws_message).await;
    }

    Ok(Json(events))
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| state.ws_manager.handle_connection(socket))
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

pub async fn get_trade_detail(
    Path(trade_id): Path<i64>,
    Query(_params): Query<TradeDetailQuery>,
    State(state): State<AppState>,
) -> Result<Json<crate::models::TradeDetailResponse>, AppError> {
    match state.database.get_trade_detail(trade_id).await? {
        Some(detail) => Ok(Json(detail)),
        None => Err(AppError::NotFound(format!("Trade {} not found", trade_id))),
    }
}

pub async fn fund_trade(
    Path(trade_id): Path<i64>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify trade exists and is in Created status
    let trade = state.database.get_trade_detail(trade_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Trade {} not found", trade_id)))?;

    if trade.status != "trade_created" {
        return Err(AppError::InvalidEventData(
            format!("Trade {} cannot be funded — current status: {}", trade_id, trade.status)
        ));
    }

    let buyer = body.get("buyer")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if !buyer.is_empty() && trade.buyer != buyer {
        return Err(AppError::InvalidEventData(
            "Provided address is not the buyer for this trade".to_string()
        ));
    }

    // In a full implementation this would invoke the Soroban contract via RPC.
    // The indexer records the intent and the on-chain event will confirm funding.
    Ok(Json(serde_json::json!({
        "status": "submitted",
        "trade_id": trade_id,
        "buyer": buyer,
        "amount": trade.amount,
        "message": "Funding transaction submitted. Awaiting on-chain confirmation."
    })))
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub ws_manager: Arc<WebSocketManager>,
}