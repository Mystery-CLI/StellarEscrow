use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tracing::{error, info};

use crate::models::WebSocketMessage;

pub struct WebSocketManager {
    tx: Sender<WebSocketMessage>,
}

impl WebSocketManager {
    pub fn new(tx: Sender<WebSocketMessage>) -> Self {
        Self { tx }
    }

    pub async fn handle_connection(self: Arc<Self>, socket: WebSocket) {
        let mut rx = self.tx.subscribe();
        let (mut sender, mut receiver) = socket.split();

        // Spawn task to handle incoming messages from client
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Close(_)) => break,
                    Ok(_) => {} // Handle other messages if needed
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        });

        // Send messages to client
        while let Ok(msg) = rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize WebSocket message: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }

        info!("WebSocket connection closed");
    }

    pub async fn broadcast(&self, message: WebSocketMessage) {
        if let Err(e) = self.tx.send(message) {
            error!("Failed to broadcast WebSocket message: {}", e);
        }
    }
}
