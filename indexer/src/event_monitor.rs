use chrono::{DateTime, Utc};
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
use crate::job_queue::{JobQueue, types::{Job, JobPriority, JobType}};
use crate::models::{Event, WebSocketMessage};
use crate::websocket::WebSocketManager;

// ---------------------------------------------------------------------------
// Horizon API response types
// ---------------------------------------------------------------------------

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
}

#[derive(Debug, Deserialize)]
struct Link {
    href: String,
}

#[derive(Debug, Deserialize)]
struct Ledger {
    sequence: i64,
}

/// Raw contract effect from Horizon.
#[derive(Debug, Deserialize)]
struct Effect {
    id: String,
    #[serde(rename = "type")]
    effect_type: String,
    created_at: String,
    contract: Option<String>,
    /// topics[0] = category symbol, topics[1] = event_name symbol
    topics: Vec<String>,
    data: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Topic → (category, event_type) mapping
// ---------------------------------------------------------------------------

/// Derive the canonical event_type string and its category from the two-topic
/// layout emitted by the contract: `(category, event_name)`.
///
/// Returns `None` for unknown/unhandled combinations so the caller can skip them.
fn resolve_event_type(category: &str, event_name: &str) -> Option<(&'static str, &'static str)> {
    match (category, event_name) {
        // trade category
        ("trade", "created")   => Some(("trade", "trade_created")),
        ("trade", "funded")    => Some(("trade", "trade_funded")),
        ("trade", "complete")  => Some(("trade", "trade_completed")),
        ("trade", "confirm")   => Some(("trade", "trade_confirmed")),
        ("trade", "cancel")    => Some(("trade", "trade_cancelled")),
        ("trade", "time_rel")  => Some(("trade", "time_released")),
        ("trade", "meta_upd")  => Some(("trade", "metadata_updated")),
        ("trade", "dispute")   => Some(("trade", "dispute_raised")),
        ("trade", "resolved")  => Some(("trade", "dispute_resolved")),
        ("trade", "part_res")  => Some(("trade", "partial_resolved")),
        // arb category
        ("arb", "arb_reg")     => Some(("arb", "arbitrator_registered")),
        ("arb", "arb_rem")     => Some(("arb", "arbitrator_removed")),
        ("arb", "arb_rate")    => Some(("arb", "arbitrator_rated")),
        ("arb", "arb_rep")     => Some(("arb", "arbitrator_rep_updated")),
        // fee category
        ("fee", "fee_upd")     => Some(("fee", "fee_updated")),
        ("fee", "fees_out")    => Some(("fee", "fees_withdrawn")),
        ("fee", "fee_dst")     => Some(("fee", "fees_distributed")),
        ("fee", "cust_fee")    => Some(("fee", "custom_fee_set")),
        ("fee", "tier_up")     => Some(("fee", "tier_upgraded")),
        ("fee", "tier_dn")     => Some(("fee", "tier_downgraded")),
        ("fee", "tier_cfg")    => Some(("fee", "tier_config_updated")),
        // tmpl category
        ("tmpl", "tmpl_cr")    => Some(("tmpl", "template_created")),
        ("tmpl", "tmpl_up")    => Some(("tmpl", "template_updated")),
        ("tmpl", "tmpl_off")   => Some(("tmpl", "template_deactivated")),
        ("tmpl", "tmpl_tr")    => Some(("tmpl", "template_trade")),
        // sub category
        ("sub", "sub_new")     => Some(("sub", "subscribed")),
        ("sub", "sub_ren")     => Some(("sub", "subscription_renewed")),
        ("sub", "sub_can")     => Some(("sub", "subscription_cancelled")),
        // gov category
        ("gov", "prop_cr")     => Some(("gov", "proposal_created")),
        ("gov", "voted")       => Some(("gov", "vote_cast")),
        ("gov", "prop_ex")     => Some(("gov", "proposal_executed")),
        ("gov", "delegat")     => Some(("gov", "delegated")),
        // sys category
        ("sys", "paused")      => Some(("sys", "paused")),
        ("sys", "unpaused")    => Some(("sys", "unpaused")),
        ("sys", "emrg_wd")     => Some(("sys", "emergency_withdraw")),
        ("sys", "upgraded")    => Some(("sys", "upgraded")),
        ("sys", "migrated")    => Some(("sys", "migrated")),
        ("sys", "priv_set")    => Some(("sys", "privacy_set")),
        ("sys", "disc_gr")     => Some(("sys", "disclosure_granted")),
        ("sys", "disc_rv")     => Some(("sys", "disclosure_revoked")),
        ("sys", "brg_set")     => Some(("sys", "bridge_oracle_set")),
        ("sys", "brg_cr")      => Some(("sys", "bridge_trade_created")),
        ("sys", "brg_ok")      => Some(("sys", "bridge_deposit_confirmed")),
        ("sys", "brg_exp")     => Some(("sys", "bridge_trade_expired")),
        ("sys", "compl_fail")  => Some(("sys", "compliance_failed")),
        ("sys", "compl_pass")  => Some(("sys", "compliance_passed")),
        ("sys", "up_prop")     => Some(("sys", "upgrade_proposed")),
        ("sys", "up_can")      => Some(("sys", "upgrade_cancelled")),
        ("sys", "up_rb")       => Some(("sys", "upgrade_rolled_back")),
        // ins category
        ("ins", "ins_reg")     => Some(("ins", "insurance_provider_registered")),
        ("ins", "ins_rem")     => Some(("ins", "insurance_provider_removed")),
        ("ins", "ins_buy")     => Some(("ins", "insurance_purchased")),
        ("ins", "ins_pay")     => Some(("ins", "insurance_claimed")),
        // oracle category
        ("oracle", "orc_reg")  => Some(("oracle", "oracle_registered")),
        ("oracle", "orc_rem")  => Some(("oracle", "oracle_removed")),
        ("oracle", "orc_px")   => Some(("oracle", "oracle_price_fetched")),
        ("oracle", "orc_err")  => Some(("oracle", "oracle_unavailable")),
        _ => None,
    }
}

/// Extract the schema version from the event data payload (`v` field).
fn extract_schema_version(data: &serde_json::Value) -> i32 {
    data.get("v")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32
}

// ---------------------------------------------------------------------------
// EventMonitor
// ---------------------------------------------------------------------------

/// How many effects to accumulate before flushing to the database.
const BATCH_SIZE: usize = 50;

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
        let latest_ledger = self.get_latest_ledger().await?;
        let start_ledger = self.last_ledger.unwrap_or(latest_ledger - 100);

        if start_ledger >= latest_ledger {
            return Ok(());
        }

        info!(
            "Polling events from ledger {} to {}",
            start_ledger, latest_ledger
        );

        let effects = self
            .get_contract_effects(start_ledger, latest_ledger)
            .await?;

        // Parse all effects into structured events first
        let mut batch: Vec<Event> = Vec::with_capacity(effects.len());
        for effect in effects {
            if let Some(event) = self.parse_effect_to_event(effect).await? {
                batch.push(event);
            }
        }

        // Flush in chunks to avoid oversized transactions
        for chunk in batch.chunks(BATCH_SIZE) {
            match self.database.insert_events_batch(chunk).await {
                Ok(result) => {
                    info!(
                        "Batch inserted {} events ({} skipped as duplicates)",
                        result.inserted, result.skipped
                    );
                }
                Err(e) => {
                    error!("Batch insert failed: {}", e);
                }
            }

            // Post-insert processing per event
            for event in chunk {
                self.process_event(event).await;
            }
        }

        self.last_ledger = Some(latest_ledger);
        Ok(())
    }

    /// Run fraud detection, WebSocket broadcast, and job enqueue for a single event.
    async fn process_event(&self, event: &Event) {
        // Broadcast to WebSocket subscribers with structured metadata
        let ws_message = WebSocketMessage {
            event_type: event.event_type.clone(),
            category: event.category.clone(),
            version: event.schema_version as u32,
            data: event.data.clone(),
            timestamp: event.timestamp,
        };
        self.ws_manager.broadcast(ws_message).await;

        // Notifications are best-effort and should not block the rest of event processing.
        self.notification_service.process_event(event).await;

        // Fraud detection for high-value trade events
        let report = match event.event_type.as_str() {
            "trade_created" => self.fraud_service.process_event(event).await,
            "trade_confirmed" => self.fraud_service.process_confirmed_event(event).await,
            _ => None,
        };

        if let Some(report) = report {
            if let Err(e) = self.database.insert_fraud_alert(&report).await {
                error!("Error inserting fraud alert: {}", e);
            }
            if report.status == "high_risk" {
                warn!(
                    "HIGH RISK TRANSACTION: trade_id={} score={}/100 rules={:?}",
                    report.trade_id, report.risk_score, report.rules_triggered
                );
                self.ws_manager
                    .broadcast(WebSocketMessage {
                        event_type: "fraud_alert".to_string(),
                        category: "sys".to_string(),
                        version: 0,
                        data: serde_json::to_value(&report).unwrap_or_default(),
                        timestamp: Utc::now(),
                    })
                    .await;
            }
        }

        let trade_id = event
            .data
            .get("trade_id")
            .map(|value| {
                value
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| value.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let event_priority = match event.event_type.as_str() {
            "dispute_raised" | "dispute_resolved" => JobPriority::Critical,
            "trade_created" | "trade_funded" | "trade_confirmed" => JobPriority::High,
            _ => JobPriority::Normal,
        };
        let event_job = Job::new(
            JobType::Event,
            event.id.to_string(),
            trade_id.clone(),
            event.data.clone(),
            event_priority,
        );

        let notification_job = Job::new(
            JobType::Notification,
            event.id.to_string(),
            trade_id,
            serde_json::json!({
                "event_type": event.event_type,
                "timestamp": event.timestamp,
                "data": event.data,
            }),
            JobPriority::High,
        );

        let mut queue = self.job_queue.lock().await;
        if let Err(e) = queue.enqueue(event_job).await {
            error!("Failed to enqueue event job for event {}: {}", event.id, e);
        }
        if let Err(e) = queue.enqueue(notification_job).await {
            error!("Failed to enqueue notification job for event {}: {}", event.id, e);
        }
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
        let mut cursor: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/effects?contract={}&ledger.ge={}&ledger.le={}&limit=200",
                self.config.horizon_url, self.config.contract_id, from_ledger, to_ledger
            );

            if let Some(ref c) = cursor {
                url.push_str(&format!("&cursor={}", c));
            }

            let response: HorizonResponse<Effect> =
                self.client.get(&url).send().await?.json().await?;

            let has_next = response._links.next.is_some();
            let last_id = response._embedded.records.last().map(|r| r.id.clone());
            all_effects.extend(response._embedded.records);

            if !has_next {
                break;
            }
            cursor = last_id;
        }

        Ok(all_effects)
    }

    async fn parse_effect_to_event(&self, effect: Effect) -> Result<Option<Event>, AppError> {
        if effect.contract.as_ref() != Some(&self.config.contract_id) {
            return Ok(None);
        }

        // Contract events use a two-topic layout: (category, event_name)
        if effect.topics.len() < 2 {
            return Ok(None);
        }

        let category_raw = &effect.topics[0];
        let event_name_raw = &effect.topics[1];

        let (category, event_type) = match resolve_event_type(category_raw, event_name_raw) {
            Some(pair) => pair,
            None => {
                // Log unknown events at debug level rather than silently dropping
                tracing::debug!(
                    "Unhandled event topic ({}, {}), skipping",
                    category_raw,
                    event_name_raw
                );
                return Ok(None);
            }
        };

        let timestamp = DateTime::parse_from_rfc3339(&effect.created_at)
            .map_err(|e| AppError::InvalidEventData(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&Utc);

        let ledger = self.extract_ledger_from_effect_id(&effect.id)?;
        let transaction_hash = self.get_transaction_hash_for_effect(&effect.id).await?;
        let schema_version = extract_schema_version(&effect.data);

        Ok(Some(Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            category: category.to_string(),
            schema_version,
            contract_id: self.config.contract_id.clone(),
            ledger,
            transaction_hash,
            timestamp,
            data: effect.data,
            created_at: Utc::now(),
        }))
    }

    fn extract_ledger_from_effect_id(&self, effect_id: &str) -> Result<i64, AppError> {
        effect_id
            .split('-')
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| AppError::InvalidEventData("Invalid effect ID format".to_string()))
    }

    async fn get_transaction_hash_for_effect(&self, effect_id: &str) -> Result<String, AppError> {
        Ok(effect_id.to_string())
    }
}
