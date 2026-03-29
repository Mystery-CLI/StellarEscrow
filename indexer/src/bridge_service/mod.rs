//! Bridge service for cross-chain trade coordination.
//!
//! Responsibilities:
//! - Monitor bridge provider attestations
//! - Validate cross-chain deposits
//! - Coordinate with multiple bridge protocols
//! - Handle bridge failures and retries
//! - Emit bridge-related events

pub mod providers;
pub mod validator;
pub mod coordinator;

pub use coordinator::BridgeCoordinator;
pub use providers::{BridgeProvider, BridgeProviderClient};
pub use validator::BridgeValidator;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::database::Database;
use crate::error::AppError;

/// Bridge service orchestrator
pub struct BridgeService {
    database: Arc<Database>,
    coordinator: Arc<BridgeCoordinator>,
    validator: Arc<BridgeValidator>,
}

impl BridgeService {
    pub fn new(database: Arc<Database>) -> Self {
        let coordinator = Arc::new(BridgeCoordinator::new(database.clone()));
        let validator = Arc::new(BridgeValidator::new(database.clone()));

        Self {
            database,
            coordinator,
            validator,
        }
    }

    /// Process incoming bridge attestation
    pub async fn process_attestation(
        &self,
        trade_id: u64,
        attestation_data: serde_json::Value,
    ) -> Result<(), AppError> {
        info!("Processing bridge attestation for trade {}", trade_id);

        // Validate attestation format and signature
        self.validator
            .validate_attestation(&attestation_data)
            .await?;

        // Coordinate with bridge provider
        self.coordinator
            .confirm_deposit(trade_id, attestation_data)
            .await?;

        info!("Bridge attestation processed successfully for trade {}", trade_id);
        Ok(())
    }

    /// Handle bridge failure with retry logic
    pub async fn handle_bridge_failure(
        &self,
        trade_id: u64,
        error: String,
    ) -> Result<(), AppError> {
        warn!("Bridge failure for trade {}: {}", trade_id, error);

        // Attempt retry
        if self.coordinator.should_retry(trade_id).await? {
            self.coordinator.retry_attestation(trade_id).await?;
            info!("Retrying bridge attestation for trade {}", trade_id);
        } else {
            // Rollback trade
            self.coordinator.rollback_trade(trade_id).await?;
            error!("Bridge attestation failed permanently for trade {}", trade_id);
        }

        Ok(())
    }

    /// Query bridge provider status
    pub async fn get_provider_status(
        &self,
        provider_name: &str,
    ) -> Result<serde_json::Value, AppError> {
        self.coordinator.get_provider_status(provider_name).await
    }

    /// List all supported bridge providers
    pub async fn list_providers(&self) -> Result<Vec<String>, AppError> {
        self.coordinator.list_providers().await
    }
}
