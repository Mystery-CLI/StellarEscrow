//! Bridge attestation validation logic.

use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::database::Database;
use crate::error::AppError;

/// Bridge attestation validator
pub struct BridgeValidator {
    database: Arc<Database>,
}

impl BridgeValidator {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Validate attestation structure and content
    pub async fn validate_attestation(
        &self,
        attestation: &serde_json::Value,
    ) -> Result<(), AppError> {
        debug!("Validating bridge attestation");

        // Check required fields
        let required_fields = vec![
            "attestation_id",
            "trade_id",
            "source_chain",
            "source_tx_hash",
            "amount",
            "recipient",
            "timestamp",
            "signature",
            "provider",
        ];

        for field in required_fields {
            if attestation.get(field).is_none() {
                return Err(AppError::BridgeError(format!(
                    "Missing required field: {}",
                    field
                )));
            }
        }

        // Validate field types
        self.validate_field_types(attestation)?;

        // Validate amount is positive
        let amount = attestation["amount"]
            .as_u64()
            .ok_or_else(|| AppError::BridgeError("Invalid amount".to_string()))?;

        if amount == 0 {
            return Err(AppError::BridgeError("Amount must be positive".to_string()));
        }

        // Validate timestamp is reasonable (not too far in past/future)
        self.validate_timestamp(attestation)?;

        // Check for duplicate attestation
        self.check_duplicate_attestation(attestation).await?;

        debug!("Attestation validation passed");
        Ok(())
    }

    /// Validate field types
    fn validate_field_types(&self, attestation: &serde_json::Value) -> Result<(), AppError> {
        if !attestation["attestation_id"].is_string() {
            return Err(AppError::BridgeError("attestation_id must be string".to_string()));
        }

        if !attestation["trade_id"].is_u64() {
            return Err(AppError::BridgeError("trade_id must be u64".to_string()));
        }

        if !attestation["source_chain"].is_string() {
            return Err(AppError::BridgeError("source_chain must be string".to_string()));
        }

        if !attestation["source_tx_hash"].is_string() {
            return Err(AppError::BridgeError("source_tx_hash must be string".to_string()));
        }

        if !attestation["amount"].is_u64() {
            return Err(AppError::BridgeError("amount must be u64".to_string()));
        }

        if !attestation["recipient"].is_string() {
            return Err(AppError::BridgeError("recipient must be string".to_string()));
        }

        if !attestation["timestamp"].is_u64() {
            return Err(AppError::BridgeError("timestamp must be u64".to_string()));
        }

        if !attestation["signature"].is_array() {
            return Err(AppError::BridgeError("signature must be array".to_string()));
        }

        if !attestation["provider"].is_string() {
            return Err(AppError::BridgeError("provider must be string".to_string()));
        }

        Ok(())
    }

    /// Validate timestamp is within acceptable range
    fn validate_timestamp(&self, attestation: &serde_json::Value) -> Result<(), AppError> {
        let timestamp = attestation["timestamp"]
            .as_u64()
            .ok_or_else(|| AppError::BridgeError("Invalid timestamp".to_string()))?;

        let now = chrono::Utc::now().timestamp() as u64;
        let max_age_secs = 3600; // 1 hour
        let max_future_secs = 300; // 5 minutes

        if now > timestamp + max_age_secs {
            return Err(AppError::BridgeError("Attestation too old".to_string()));
        }

        if timestamp > now + max_future_secs {
            return Err(AppError::BridgeError("Attestation timestamp in future".to_string()));
        }

        Ok(())
    }

    /// Check if attestation already processed
    async fn check_duplicate_attestation(
        &self,
        attestation: &serde_json::Value,
    ) -> Result<(), AppError> {
        let attestation_id = attestation["attestation_id"]
            .as_str()
            .ok_or_else(|| AppError::BridgeError("Invalid attestation_id".to_string()))?;

        // Query database for existing attestation
        let query = "SELECT id FROM events WHERE data->>'attestation_id' = $1 AND event_type = 'BridgeAttestationReceived' LIMIT 1";

        let result = sqlx::query(query)
            .bind(attestation_id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.is_some() {
            return Err(AppError::BridgeError(
                "Attestation already processed".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate cross-chain amount against limits
    pub async fn validate_amount_limits(
        &self,
        amount: u64,
        provider: &str,
    ) -> Result<(), AppError> {
        // Query provider configuration from database
        let query = "SELECT data FROM events WHERE event_type = 'BridgeProviderRegistered' AND data->>'provider' = $1 ORDER BY created_at DESC LIMIT 1";

        let result = sqlx::query_as::<_, (serde_json::Value,)>(query)
            .bind(provider)
            .fetch_optional(self.database.pool())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some((config,)) = result {
            let min_amount = config["min_trade_amount"]
                .as_u64()
                .unwrap_or(1_000_000); // 0.1 USDC default
            let max_amount = config["max_trade_amount"]
                .as_u64()
                .unwrap_or(1_000_000_000_000); // 100M USDC default

            if amount < min_amount || amount > max_amount {
                return Err(AppError::BridgeError(format!(
                    "Amount {} outside limits [{}, {}]",
                    amount, min_amount, max_amount
                )));
            }
        }

        Ok(())
    }

    /// Validate source chain is supported
    pub async fn validate_source_chain(
        &self,
        source_chain: &str,
        provider: &str,
    ) -> Result<(), AppError> {
        // Query provider configuration
        let query = "SELECT data FROM events WHERE event_type = 'BridgeProviderRegistered' AND data->>'provider' = $1 ORDER BY created_at DESC LIMIT 1";

        let result = sqlx::query_as::<_, (serde_json::Value,)>(query)
            .bind(provider)
            .fetch_optional(self.database.pool())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if let Some((config,)) = result {
            let supported_chains = config["supported_chains"]
                .as_str()
                .unwrap_or("")
                .split(',')
                .collect::<Vec<_>>();

            if !supported_chains.contains(&source_chain) {
                return Err(AppError::BridgeError(format!(
                    "Source chain {} not supported by provider {}",
                    source_chain, provider
                )));
            }
        }

        Ok(())
    }
}
