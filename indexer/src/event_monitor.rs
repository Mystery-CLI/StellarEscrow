use chrono::{DateTime, Utc};
use futures::stream::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::StellarConfig;
use crate::database::Database;
use crate::error::AppError;
use crate::fraud_service::FraudDetectionService;
use crate::job_queue::{JobQueue, types::{Job, JobType}};
use crate::models::{Event, WebSocketMessage};
use crate::websocket::WebSocketManager;

#[derive(Debug, Deserialize)]
struct HorizonResponse<T> {
    _embedded: EmbeddedRecords<T>,
    _links: Links,
}

#[derive(Debug, Deserialize)]
struct EmbeddedRecords<T> {
    records: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct Links {
    next: Option<Link>,
    prev: Option<Link>,
}

#[derive(Debug, Deserialize)]
struct Link {
    href: String,
}

#[derive(Debug, Deserialize)]
struct Ledger {
    sequence: i64,
    closed_at: String,
}

#[derive(Debug, Deserialize)]
struct Effect {
    id: String,
    #[serde(rename = "type")]
    effect_type: String,
    created_at: String,
    contract: Option<String>,
    topics: Vec<String>,
    data: serde_json::Value,
}

pub struct EventMonitor {
    config: StellarConfig,
    database: Arc<Database>,
    ws_manager: Arc<WebSocketManager>,
    client: Client,
    last_ledger: Option<i64>,
    fraud_service: Arc<FraudDetectionService>,
    notification_service: Arc<crate::notification_service::NotificationService>,
    integration_service: Arc<crate::integration_service::IntegrationService>,
    job_queue: Arc<tokio::sync::Mutex<JobQueue>>,
}

impl EventMonitor {
    pub fn new(
        config: StellarConfig,
        database: Arc<Database>,
        ws_manager: Arc<WebSocketManager>,
        fraud_service: Arc<FraudDetectionService>,
        notification_service: Arc<crate::notification_service::NotificationService>,
        integration_service: Arc<crate::integration_service::IntegrationService>,
        job_queue: Arc<tokio::sync::Mutex<JobQueue>>,
    ) -> Self {
        Self {
            config: config.clone(),
            database,
            ws_manager,
            fraud_service,
            notification_service,
            integration_service,
            job_queue,
            client: Client::new(),
            last_ledger: config.start_ledger.map(|l| l as i64),
        }
    }

    pub async fn start(&mut self) -> Result<(), AppError> {
        info!(
            "Starting event monitor for contract {}",
            self.config.contract_id
        );

        // Get the latest ledger from database if not specified in config
        if self.last_ledger.is_none() {
            match self
                .database
                .get_latest_ledger(&self.config.contract_id)
                .await?
            {
                Some(ledger) => {
                    info!("Resuming from ledger {}", ledger);
                    self.last_ledger = Some(ledger);
                }
                None => {
                    warn!("No previous events found, starting from latest ledger");
                }
            }
        }

        loop {
            if let Err(e) = self.poll_events().await {
                error!("Error polling events: {}", e);
            }

            sleep(Duration::from_secs(self.config.poll_interval_seconds)).await;
        }
    }

    async fn poll_events(&mut self) -> Result<(), AppError> {
        // Get latest ledger
        let latest_ledger = self.get_latest_ledger().await?;
        let start_ledger = self.last_ledger.unwrap_or(latest_ledger - 100); // Look back 100 ledgers if no start point

        if start_ledger >= latest_ledger {
            return Ok(()); // No new ledgers
        }

        info!(
            "Polling events from ledger {} to {}",
            start_ledger, latest_ledger
        );

        // Get effects for the contract
        let effects = self
            .get_contract_effects(start_ledger, latest_ledger)
            .await?;

        for effect in effects {
            if let Some(event) = self.parse_effect_to_event(effect).await? {
                // Broadcast to WebSocket clients
                let ws_message = WebSocketMessage {
                    event_type: event.event_type.clone(),
                    data: event.data.clone(),
                    timestamp: event.timestamp,
                };
                self.ws_manager.broadcast(ws_message).await;

                // Process fraud detection
                let report = if event.event_type == "trade_created" {
                    self.fraud_service.process_event(&event).await
                } else if event.event_type == "trade_confirmed" {
                    self.fraud_service.process_confirmed_event(&event).await
                } else {
                    None
                };

                if let Some(report) = report {
                    if let Err(e) = self.database.insert_fraud_alert(&report).await {
                        error!("Error inserting fraud alert: {}", e);
                    }
                    if report.status == "high_risk" {
                        warn!("!!! HIGH RISK TRANSACTION DETECTED !!!");
                        warn!("Trade ID: {}", report.trade_id);
                        warn!("Score: {}/100", report.risk_score);
                        warn!("Rules: {:?}", report.rules_triggered);

                        // Emit a special "fraud_alert" websocket message for real-time dashboard updates
                        self.ws_manager
                            .broadcast(WebSocketMessage {
                                event_type: "fraud_alert".to_string(),
                                data: serde_json::to_value(&report).unwrap_or_default(),
                                timestamp: Utc::now(),
                            })
                            .await;
                    }
                }

                // Enqueue background job for event processing
                let job = Job {
                    job_type: JobType::Event,
                    event_id: event.id.to_string(),
                    trade_id: event.data.get("trade_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    payload: event.data.clone(),
                };
                
                let mut queue = self.job_queue.lock().await;
                if let Err(e) = queue.enqueue(job).await {
                    error!("Failed to enqueue job: {}", e);
                }
            }
        }

        self.last_ledger = Some(latest_ledger);
        Ok(())
    }

    async fn get_latest_ledger(&self) -> Result<i64, AppError> {
        let url = format!("{}/ledgers?order=desc&limit=1", self.config.horizon_url);
        let response: HorizonResponse<Ledger> = self.client.get(&url).send().await?.json().await?;
        Ok(response._embedded.records[0].sequence)
    }

    async fn get_contract_effects(
        &self,
        from_ledger: i64,
        to_ledger: i64,
    ) -> Result<Vec<Effect>, AppError> {
        let mut all_effects = Vec::new();
        let mut cursor = None;

        loop {
            let mut url = format!(
                "{}/effects?contract={}&ledger.ge={}&ledger.le={}",
                self.config.horizon_url, self.config.contract_id, from_ledger, to_ledger
            );

            if let Some(ref c) = cursor {
                url.push_str(&format!("&cursor={}", c));
            }

            let response: HorizonResponse<Effect> =
                self.client.get(&url).send().await?.json().await?;

            let next = response._links.next.is_none();
            let last_id = response._embedded.records.last().map(|r| r.id.clone());
            all_effects.extend(response._embedded.records);

            if next {
                break;
            }

            cursor = last_id;
        }

        Ok(all_effects)
    }

    async fn parse_effect_to_event(&self, effect: Effect) -> Result<Option<Event>, AppError> {
        // Only process contract events
        if effect.contract.as_ref() != Some(&self.config.contract_id) {
            return Ok(None);
        }

        // Parse topics to determine event type
        if effect.topics.is_empty() {
            return Ok(None);
        }

        let event_type = match effect.topics[0].as_str() {
            "created" => "trade_created",
            "funded" => "trade_funded",
            "complete" => "trade_completed",
            "confirm" => "trade_confirmed",
            "dispute" => "dispute_raised",
            "resolved" => "dispute_resolved",
            "cancel" => "trade_cancelled",
            "arb_reg" => "arbitrator_registered",
            "arb_rem" => "arbitrator_removed",
            "fee_upd" => "fee_updated",
            "fees_out" => "fees_withdrawn",
            _ => return Ok(None), // Unknown event type
        };

        // Parse timestamp
        let timestamp = DateTime::parse_from_rfc3339(&effect.created_at)
            .map_err(|e| AppError::InvalidEventData(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);

        // Get ledger from effect ID (effects are ordered by ledger)
        let ledger = self.extract_ledger_from_effect_id(&effect.id)?;

        // Get transaction hash from effect
        let transaction_hash = self.get_transaction_hash_for_effect(&effect.id).await?;

        let event = Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            contract_id: self.config.contract_id.clone(),
            ledger,
            transaction_hash,
            timestamp,
            data: effect.data,
            created_at: Utc::now(),
        };

        Ok(Some(event))
    }

    fn extract_ledger_from_effect_id(&self, effect_id: &str) -> Result<i64, AppError> {
        // Effect IDs are in format: <ledger>-<transaction>-<operation>-<effect>
        let parts: Vec<&str> = effect_id.split('-').collect();
        if parts.len() < 1 {
            return Err(AppError::InvalidEventData(
                "Invalid effect ID format".to_string(),
            ));
        }

        parts[0]
            .parse()
            .map_err(|_| AppError::InvalidEventData("Invalid ledger in effect ID".to_string()))
    }

    async fn get_transaction_hash_for_effect(&self, effect_id: &str) -> Result<String, AppError> {
        // For simplicity, we'll use the effect ID as transaction hash
        // In a real implementation, you'd query the effect's transaction
        Ok(effect_id.to_string())
    }
}
