//! Bridge coordination and state management.

use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::database::Database;
use crate::error::AppError;

use super::providers::{BridgeProvider, BridgeProviderFactory};

/// Bridge trade state tracker
#[derive(Debug, Clone)]
struct BridgeTradeState {
    trade_id: u64,
    retry_count: u32,
    last_retry_time: u64,
    status: String,
}

/// Bridge coordinator for managing cross-chain trades
pub struct BridgeCoordinator {
    database: Arc<Database>,
    trade_states: Arc<RwLock<HashMap<u64, BridgeTradeState>>>,
    provider_clients: Arc<RwLock<HashMap<String, Box<dyn super::providers::BridgeProviderClient>>>>,
}

impl BridgeCoordinator {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            trade_states: Arc::new(RwLock::new(HashMap::new())),
            provider_clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize bridge provider client
    pub async fn init_provider(
        &self,
        provider: BridgeProvider,
        config: HashMap<String, String>,
    ) -> Result<(), AppError> {
        debug!("Initializing bridge provider: {:?}", provider);

        let client = BridgeProviderFactory::create_client(&provider, &config)?;

        let mut clients = self.provider_clients.write().await;
        clients.insert(provider.as_str().to_string(), client);

        info!("Bridge provider initialized: {:?}", provider);
        Ok(())
    }

    /// Confirm deposit from bridge
    pub async fn confirm_deposit(
        &self,
        trade_id: u64,
        attestation_data: serde_json::Value,
    ) -> Result<(), AppError> {
        debug!("Confirming bridge deposit for trade {}", trade_id);

        let provider_name = attestation_data["provider"]
            .as_str()
            .ok_or_else(|| AppError::BridgeError("Missing provider".to_string()))?;

        let clients = self.provider_clients.read().await;
        let client = clients
            .get(provider_name)
            .ok_or_else(|| AppError::BridgeError(format!("Provider {} not initialized", provider_name)))?;

        // Verify attestation signature
        let attestation = serde_json::from_value(attestation_data.clone())
            .map_err(|e| AppError::BridgeError(format!("Invalid attestation: {}", e)))?;

        let valid = client.verify_attestation(&attestation).await?;
        if !valid {
            return Err(AppError::BridgeError("Attestation signature invalid".to_string()));
        }

        // Get confirmation status
        let source_tx_hash = attestation_data["source_tx_hash"]
            .as_str()
            .ok_or_else(|| AppError::BridgeError("Missing source_tx_hash".to_string()))?;

        let confirmation = client.get_confirmation_status(source_tx_hash).await?;

        if !confirmation.finalized {
            warn!(
                "Bridge deposit not finalized for trade {}: {} confirmations",
                trade_id, confirmation.confirmations
            );
            return Err(AppError::BridgeError(
                "Deposit not finalized".to_string(),
            ));
        }

        // Store attestation event
        self.store_attestation_event(trade_id, attestation_data, "confirmed")
            .await?;

        // Update trade state
        let mut states = self.trade_states.write().await;
        states.insert(
            trade_id,
            BridgeTradeState {
                trade_id,
                retry_count: 0,
                last_retry_time: chrono::Utc::now().timestamp() as u64,
                status: "confirmed".to_string(),
            },
        );

        info!("Bridge deposit confirmed for trade {}", trade_id);
        Ok(())
    }

    /// Check if trade should be retried
    pub async fn should_retry(&self, trade_id: u64) -> Result<bool, AppError> {
        let states = self.trade_states.read().await;

        if let Some(state) = states.get(&trade_id) {
            let max_retries = 3;
            let retry_delay_secs = 300; // 5 minutes
            let now = chrono::Utc::now().timestamp() as u64;

            if state.retry_count < max_retries
                && now > state.last_retry_time + retry_delay_secs
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Retry failed attestation
    pub async fn retry_attestation(&self, trade_id: u64) -> Result<(), AppError> {
        debug!("Retrying bridge attestation for trade {}", trade_id);

        let mut states = self.trade_states.write().await;

        if let Some(state) = states.get_mut(&trade_id) {
            state.retry_count += 1;
            state.last_retry_time = chrono::Utc::now().timestamp() as u64;
            state.status = "retrying".to_string();

            // Store retry event
            let query = "INSERT INTO events (id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

            let event_data = json!({
                "trade_id": trade_id,
                "retry_count": state.retry_count,
                "timestamp": chrono::Utc::now().timestamp()
            });

            sqlx::query(query)
                .bind(uuid::Uuid::new_v4())
                .bind("BridgeAttestationRetried")
                .bind("stellar-escrow")
                .bind(0i64)
                .bind("")
                .bind(chrono::Utc::now())
                .bind(event_data)
                .bind(chrono::Utc::now())
                .execute(self.database.pool())
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            info!("Bridge attestation retry #{} for trade {}", state.retry_count, trade_id);
        }

        Ok(())
    }

    /// Rollback failed trade
    pub async fn rollback_trade(&self, trade_id: u64) -> Result<(), AppError> {
        error!("Rolling back bridge trade {}", trade_id);

        let mut states = self.trade_states.write().await;
        states.remove(&trade_id);

        // Store rollback event
        let query = "INSERT INTO events (id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

        let event_data = json!({
            "trade_id": trade_id,
            "reason": "bridge_attestation_failed",
            "timestamp": chrono::Utc::now().timestamp()
        });

        sqlx::query(query)
            .bind(uuid::Uuid::new_v4())
            .bind("BridgeTradeRolledBack")
            .bind("stellar-escrow")
            .bind(0i64)
            .bind("")
            .bind(chrono::Utc::now())
            .bind(event_data)
            .bind(chrono::Utc::now())
            .execute(self.database.pool())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get provider status
    pub async fn get_provider_status(
        &self,
        provider_name: &str,
    ) -> Result<serde_json::Value, AppError> {
        let clients = self.provider_clients.read().await;
        let client = clients
            .get(provider_name)
            .ok_or_else(|| AppError::BridgeError(format!("Provider {} not found", provider_name)))?;

        let status = client.health_check().await?;

        Ok(json!({
            "provider": status.provider,
            "healthy": status.healthy,
            "latency_ms": status.latency_ms,
            "last_check": status.last_check,
            "error": status.error_message
        }))
    }

    /// List all providers
    pub async fn list_providers(&self) -> Result<Vec<String>, AppError> {
        let clients = self.provider_clients.read().await;
        Ok(clients.keys().cloned().collect())
    }

    /// Store attestation event in database
    async fn store_attestation_event(
        &self,
        trade_id: u64,
        attestation_data: serde_json::Value,
        status: &str,
    ) -> Result<(), AppError> {
        let query = "INSERT INTO events (id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

        let event_data = json!({
            "trade_id": trade_id,
            "attestation_id": attestation_data.get("attestation_id"),
            "status": status,
            "timestamp": chrono::Utc::now().timestamp()
        });

        sqlx::query(query)
            .bind(uuid::Uuid::new_v4())
            .bind("BridgeAttestationConfirmed")
            .bind("stellar-escrow")
            .bind(0i64)
            .bind("")
            .bind(chrono::Utc::now())
            .bind(event_data)
            .bind(chrono::Utc::now())
            .execute(self.database.pool())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
