use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::{ConnectorConfig, IntegrationConfig};
use crate::database::Database;
use crate::models::Event;

pub mod connectors;

#[cfg(test)]
mod test;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRecord {
    pub id: Uuid,
    pub connector_id: String,
    pub event_id: Uuid,
    pub status: DeliveryStatus,
    pub status_code: Option<u16>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub attempted_at: chrono::DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Monitoring state
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, Serialize)]
pub struct ConnectorStats {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub last_error: Option<String>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct IntegrationService {
    db: Option<Arc<Database>>,
    connectors: Vec<ConnectorConfig>,
    stats: Arc<RwLock<HashMap<String, ConnectorStats>>>,
    http: reqwest::Client,
}

impl IntegrationService {
    pub fn new(db: Arc<Database>, cfg: IntegrationConfig) -> Self {
        Self::build(Some(db), cfg)
    }

    #[cfg(test)]
    pub fn new_without_db(cfg: IntegrationConfig) -> Self {
        Self::build(None, cfg)
    }

    fn build(db: Option<Arc<Database>>, cfg: IntegrationConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");

        let mut stats = HashMap::new();
        for c in &cfg.connectors {
            stats.insert(c.id.clone(), ConnectorStats::default());
        }

        Self {
            db,
            connectors: cfg.connectors,
            stats: Arc::new(RwLock::new(stats)),
            http,
        }
    }

    /// Called by the event monitor for every new contract event.
    pub async fn process_event(&self, event: &Event) {
        for connector in &self.connectors {
            if !self.should_forward(connector, event) {
                continue;
            }
            let record = connectors::deliver(&self.http, connector, event).await;
            self.record_delivery(&record).await;
        }
    }

    /// Returns monitoring stats for all connectors.
    pub async fn get_stats(&self) -> HashMap<String, ConnectorStats> {
        self.stats.read().await.clone()
    }

    /// Returns recent delivery records from the database.
    pub async fn get_delivery_log(
        &self,
        connector_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<DeliveryRecord>, sqlx::Error> {
        let db = self.db.as_ref().expect("db required");
        db.get_integration_deliveries(connector_id, limit).await
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn should_forward(&self, connector: &ConnectorConfig, event: &Event) -> bool {
        connector.event_filter.is_empty() || connector.event_filter.contains(&event.event_type)
    }

    async fn record_delivery(&self, record: &DeliveryRecord) {
        {
            let mut stats = self.stats.write().await;
            let entry = stats.entry(record.connector_id.clone()).or_default();
            entry.total += 1;
            if record.status == DeliveryStatus::Success {
                entry.success += 1;
                info!(
                    connector = %record.connector_id,
                    event = %record.event_id,
                    duration_ms = record.duration_ms,
                    "Integration delivery succeeded"
                );
            } else {
                entry.failed += 1;
                entry.last_error = record.error.clone();
                warn!(
                    connector = %record.connector_id,
                    event = %record.event_id,
                    error = ?record.error,
                    "Integration delivery failed"
                );
            }
        }

        if let Some(db) = &self.db {
            if let Err(e) = db.insert_integration_delivery(record).await {
                error!("Failed to persist delivery record: {}", e);
            }
        }
    }
}
