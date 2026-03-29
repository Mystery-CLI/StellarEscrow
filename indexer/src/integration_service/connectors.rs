use std::time::Instant;

use chrono::Utc;
use uuid::Uuid;

use crate::config::{ConnectorConfig, ConnectorKind};
use crate::models::Event;

use super::{DeliveryRecord, DeliveryStatus};

/// Deliver an event to a connector and return a `DeliveryRecord`.
pub async fn deliver(
    http: &reqwest::Client,
    connector: &ConnectorConfig,
    event: &Event,
) -> DeliveryRecord {
    let start = Instant::now();
    let result = match connector.kind {
        ConnectorKind::Webhook | ConnectorKind::Http => send_http(http, connector, event).await,
    };
    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(status_code) => DeliveryRecord {
            id: Uuid::new_v4(),
            connector_id: connector.id.clone(),
            event_id: event.id,
            status: DeliveryStatus::Success,
            status_code: Some(status_code),
            error: None,
            duration_ms,
            attempted_at: Utc::now(),
        },
        Err(err) => DeliveryRecord {
            id: Uuid::new_v4(),
            connector_id: connector.id.clone(),
            event_id: event.id,
            status: DeliveryStatus::Failed,
            status_code: None,
            error: Some(err),
            duration_ms,
            attempted_at: Utc::now(),
        },
    }
}

async fn send_http(
    http: &reqwest::Client,
    connector: &ConnectorConfig,
    event: &Event,
) -> Result<u16, String> {
    let mut req = http
        .post(&connector.url)
        .timeout(std::time::Duration::from_secs(connector.timeout_secs))
        .json(event);

    if let Some(token) = &connector.auth_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();

    if resp.status().is_success() {
        Ok(status)
    } else {
        Err(format!("HTTP {}", status))
    }
}
