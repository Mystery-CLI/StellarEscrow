pub mod delivery;
pub mod validation;

use crate::database::Database;
use crate::models::Event;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub url: String,
    pub secret: String,
    pub event_types: Vec<String>, // empty = all events
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub failure_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliveryRecord {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status_code: Option<u16>,
    pub success: bool,
    pub attempt: u32,
    pub error: Option<String>,
    pub delivered_at: DateTime<Utc>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookStats {
    pub total_endpoints: usize,
    pub active_endpoints: usize,
    pub total_deliveries: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    pub success_rate: f64,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct WebhookService {
    db: Arc<Database>,
    endpoints: Arc<RwLock<Vec<WebhookEndpoint>>>,
    delivery_log: Arc<RwLock<Vec<WebhookDeliveryRecord>>>,
    http: reqwest::Client,
}

impl WebhookService {
    pub fn new(db: Arc<Database>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build webhook HTTP client");

        Self {
            db,
            endpoints: Arc::new(RwLock::new(Vec::new())),
            delivery_log: Arc::new(RwLock::new(Vec::new())),
            http,
        }
    }

    /// Load registered endpoints from DB at startup.
    pub async fn load_endpoints(&self) {
        match self.db.get_webhook_endpoints().await {
            Ok(endpoints) => {
                *self.endpoints.write().await = endpoints;
                tracing::info!("Loaded {} webhook endpoints", self.endpoints.read().await.len());
            }
            Err(e) => tracing::warn!("Failed to load webhook endpoints: {}", e),
        }
    }

    /// Register a new webhook endpoint.
    pub async fn register(&self, url: String, secret: String, event_types: Vec<String>) -> anyhow::Result<WebhookEndpoint> {
        // Validate URL
        validation::validate_url(&url)?;

        let endpoint = WebhookEndpoint {
            id: Uuid::new_v4(),
            url,
            secret,
            event_types,
            active: true,
            created_at: Utc::now(),
            failure_count: 0,
        };

        self.db.insert_webhook_endpoint(&endpoint).await?;
        self.endpoints.write().await.push(endpoint.clone());
        Ok(endpoint)
    }

    /// Deactivate a webhook endpoint.
    pub async fn deactivate(&self, id: Uuid) -> anyhow::Result<()> {
        self.db.deactivate_webhook_endpoint(id).await?;
        let mut endpoints = self.endpoints.write().await;
        if let Some(ep) = endpoints.iter_mut().find(|e| e.id == id) {
            ep.active = false;
        }
        Ok(())
    }

    /// Process a contract event — deliver to all matching active endpoints.
    pub async fn process_event(&self, event: &Event) {
        let endpoints = self.endpoints.read().await.clone();
        let matching: Vec<WebhookEndpoint> = endpoints.into_iter()
            .filter(|ep| ep.active && self.matches(ep, &event.event_type))
            .collect();

        if matching.is_empty() {
            return;
        }

        let payload = serde_json::json!({
            "id": Uuid::new_v4(),
            "event_type": event.event_type,
            "ledger": event.ledger,
            "timestamp": event.timestamp,
            "data": event.data,
        });

        for endpoint in matching {
            let http = self.http.clone();
            let db = self.db.clone();
            let log = self.delivery_log.clone();
            let endpoints_ref = self.endpoints.clone();
            let payload = payload.clone();

            tokio::spawn(async move {
                let record = delivery::deliver_with_retry(&http, &endpoint, &payload, 3).await;

                // Auto-disable endpoint after 10 consecutive failures
                if !record.success {
                    let mut eps = endpoints_ref.write().await;
                    if let Some(ep) = eps.iter_mut().find(|e| e.id == endpoint.id) {
                        ep.failure_count += 1;
                        if ep.failure_count >= 10 {
                            ep.active = false;
                            tracing::warn!("Webhook endpoint {} auto-disabled after 10 failures", ep.id);
                        }
                    }
                }

                log.write().await.push(record.clone());

                if let Err(e) = db.insert_webhook_delivery(&record).await {
                    tracing::warn!("Failed to persist webhook delivery: {}", e);
                }
            });
        }
    }

    pub async fn get_endpoints(&self) -> Vec<WebhookEndpoint> {
        self.endpoints.read().await.clone()
    }

    pub async fn get_delivery_log(&self, limit: usize) -> Vec<WebhookDeliveryRecord> {
        let log = self.delivery_log.read().await;
        log.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_stats(&self) -> WebhookStats {
        let endpoints = self.endpoints.read().await;
        let log = self.delivery_log.read().await;
        let total = log.len() as u64;
        let successful = log.iter().filter(|r| r.success).count() as u64;

        WebhookStats {
            total_endpoints: endpoints.len(),
            active_endpoints: endpoints.iter().filter(|e| e.active).count(),
            total_deliveries: total,
            successful_deliveries: successful,
            failed_deliveries: total.saturating_sub(successful),
            success_rate: if total > 0 { successful as f64 / total as f64 } else { 1.0 },
        }
    }

    fn matches(&self, endpoint: &WebhookEndpoint, event_type: &str) -> bool {
        endpoint.event_types.is_empty() || endpoint.event_types.iter().any(|t| t == event_type)
    }
}
