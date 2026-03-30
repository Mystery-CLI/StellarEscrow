use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::models::WebSocketMessage;

// ---------------------------------------------------------------------------
// JWT Claims
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct WsClaims {
    pub sub: String,
    pub exp: usize,
}

// ---------------------------------------------------------------------------
// Client subscription state
// ---------------------------------------------------------------------------

/// A connected WebSocket client with its subscribed trade IDs.
#[derive(Debug)]
struct WsClient {
    id: Uuid,
    /// Empty set = subscribed to all events; non-empty = only matching trade_ids.
    trade_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// Missed-event buffer
// ---------------------------------------------------------------------------

/// Ring-buffer of recent messages for reconnecting clients.
const REPLAY_BUFFER_SIZE: usize = 200;

// ---------------------------------------------------------------------------
// WebSocketManager
// ---------------------------------------------------------------------------

pub struct WebSocketManager {
    tx: broadcast::Sender<WebSocketMessage>,
    /// Recent messages for catch-up on reconnect.
    replay_buffer: Arc<RwLock<Vec<WebSocketMessage>>>,
    /// JWT secret for authenticating connections.
    jwt_secret: String,
}

impl WebSocketManager {
    pub fn new(tx: broadcast::Sender<WebSocketMessage>, jwt_secret: impl Into<String>) -> Self {
        Self {
            tx,
            replay_buffer: Arc::new(RwLock::new(Vec::with_capacity(REPLAY_BUFFER_SIZE))),
            jwt_secret: jwt_secret.into(),
        }
    }

    /// Validate a JWT token and return the subject claim.
    pub fn validate_token(&self, token: &str) -> Option<String> {
        let key = DecodingKey::from_secret(self.jwt_secret.as_bytes());
        decode::<WsClaims>(token, &key, &Validation::default())
            .ok()
            .map(|data| data.claims.sub)
    }

    /// Handle an authenticated WebSocket connection.
    pub async fn handle_connection(self: Arc<Self>, socket: WebSocket, subject: String) {
        let client_id = Uuid::new_v4();
        info!("WebSocket client {} connected (sub={})", client_id, subject);

        let mut rx = self.tx.subscribe();
        let (mut sender, mut receiver) = socket.split();

        // Send missed events from replay buffer
        {
            let buf = self.replay_buffer.read().await;
            for msg in buf.iter() {
                if let Ok(json) = serde_json::to_string(msg) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        return;
                    }
                }
            }
        }

        // Track which trade IDs this client wants (populated via subscribe messages)
        let subscribed_trades: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
        let sub_trades_recv = subscribed_trades.clone();

        // Spawn task to handle incoming control messages from client
        let recv_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(ctrl) = serde_json::from_str::<ClientControl>(&text) {
                            match ctrl.action.as_str() {
                                "subscribe" => {
                                    if let Some(trade_id) = ctrl.trade_id {
                                        let mut trades = sub_trades_recv.write().await;
                                        if !trades.contains(&trade_id) {
                                            trades.push(trade_id);
                                        }
                                    }
                                }
                                "unsubscribe" => {
                                    if let Some(trade_id) = ctrl.trade_id {
                                        let mut trades = sub_trades_recv.write().await;
                                        trades.retain(|t| t != &trade_id);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        // pong is handled automatically by axum
                        let _ = data;
                    }
                    Ok(Message::Close(_)) | Err(_) => break,
                    _ => {}
                }
            }
        });

        // Forward broadcast messages to this client, filtered by subscription
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let trades = subscribed_trades.read().await;
                    // If client subscribed to specific trades, filter
                    if !trades.is_empty() {
                        let trade_id = msg.data.get("trade_id")
                            .and_then(|v| v.as_str().map(String::from)
                                .or_else(|| v.as_u64().map(|n| n.to_string())));
                        if let Some(tid) = trade_id {
                            if !trades.contains(&tid) {
                                continue;
                            }
                        }
                    }
                    drop(trades);

                    let json = match serde_json::to_string(&msg) {
                        Ok(j) => j,
                        Err(e) => { error!("WS serialize error: {}", e); continue; }
                    };
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("WS client {} lagged {} messages", client_id, n);
                    // Send a lag notice so the client knows to re-poll
                    let notice = serde_json::json!({
                        "event_type": "lag_notice",
                        "missed": n,
                    });
                    let _ = sender.send(Message::Text(notice.to_string())).await;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }

        recv_task.abort();
        info!("WebSocket client {} disconnected", client_id);
    }

    /// Broadcast a message to all connected clients and store in replay buffer.
    pub async fn broadcast(&self, message: WebSocketMessage) {
        // Store in replay buffer (ring-buffer behaviour)
        {
            let mut buf = self.replay_buffer.write().await;
            if buf.len() >= REPLAY_BUFFER_SIZE {
                buf.remove(0);
            }
            buf.push(message.clone());
        }

        if let Err(e) = self.tx.send(message) {
            // No receivers is fine (no clients connected)
            if e.to_string().contains("channel closed") {
                error!("WS broadcast channel closed: {}", e);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Client control message schema
// ---------------------------------------------------------------------------

/// Messages sent from the client to the server over the WebSocket.
///
/// ```json
/// { "action": "subscribe",   "trade_id": "42" }
/// { "action": "unsubscribe", "trade_id": "42" }
/// ```
#[derive(Debug, Deserialize)]
struct ClientControl {
    action: String,
    trade_id: Option<String>,
}
