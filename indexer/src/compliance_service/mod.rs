pub mod kyc;
pub mod aml;
pub mod workflow;
pub mod reporting;

use crate::config::ComplianceConfig;
use crate::database::Database;
use crate::models::Event;
use aml::{AmlScreener, AmlResult};
use kyc::{KycProvider, KycResult, KycStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Pending,
    Approved,
    Rejected,
    RequiresReview,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub id: Uuid,
    pub address: String,
    pub trade_id: Option<u64>,
    pub kyc_result: KycResult,
    pub aml_result: AmlResult,
    pub status: ComplianceStatus,
    pub risk_score: u8,
    pub notes: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

impl ComplianceCheck {
    pub fn is_approved(&self) -> bool {
        self.status == ComplianceStatus::Approved
    }
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct ComplianceService {
    db: Arc<Database>,
    kyc: KycProvider,
    aml: AmlScreener,
    config: ComplianceConfig,
}

impl ComplianceService {
    pub fn new(db: Arc<Database>, config: ComplianceConfig) -> Self {
        let kyc = KycProvider::new(&config.kyc_provider_url, &config.kyc_api_key);
        let aml = AmlScreener::new(
            &config.aml_provider_url,
            &config.aml_api_key,
            config.aml_risk_threshold,
            config.blocked_jurisdictions.clone(),
        );
        Self { db, kyc, aml, config }
    }

    /// Run full KYC + AML compliance check for an address.
    pub async fn check_address(&self, address: &str, trade_id: Option<u64>) -> ComplianceCheck {
        let kyc_result = self.kyc.verify(address).await;
        let aml_result = self.aml.screen(address).await;

        let status = self.determine_status(&kyc_result, &aml_result);
        let risk_score = self.calculate_risk_score(&kyc_result, &aml_result);

        let check = ComplianceCheck {
            id: Uuid::new_v4(),
            address: address.to_string(),
            trade_id,
            kyc_result,
            aml_result,
            status,
            risk_score,
            notes: None,
            checked_at: Utc::now(),
            reviewed_by: None,
            reviewed_at: None,
        };

        // Persist to DB
        if let Err(e) = self.db.insert_compliance_check(&check).await {
            tracing::error!("Failed to persist compliance check: {}", e);
        }

        check
    }

    /// Process a trade_created event — check both buyer and seller.
    pub async fn process_trade_event(&self, event: &Event) -> Option<Vec<ComplianceCheck>> {
        if event.event_type != "trade_created" {
            return None;
        }

        let data: serde_json::Value = event.data.clone();
        let trade_id = data.get("trade_id")?.as_u64();
        let seller = data.get("seller")?.as_str()?.to_string();
        let buyer = data.get("buyer")?.as_str()?.to_string();
        let amount = data.get("amount").and_then(|v| v.as_u64()).unwrap_or(0);

        let (seller_check, buyer_check) = tokio::join!(
            self.check_address(&seller, trade_id),
            self.check_address(&buyer, trade_id),
        );

        // Enforce trade amount limits
        if amount > self.config.max_trade_amount && self.config.max_trade_amount > 0 {
            tracing::warn!(
                trade_id = ?trade_id,
                amount,
                limit = self.config.max_trade_amount,
                "Trade amount exceeds configured limit"
            );
        }

        // Emit compliance event if either party is blocked
        if seller_check.status == ComplianceStatus::Blocked
            || buyer_check.status == ComplianceStatus::Blocked
        {
            tracing::warn!(
                trade_id = ?trade_id,
                seller_status = ?seller_check.status,
                buyer_status = ?buyer_check.status,
                "Trade blocked by compliance"
            );
        }

        Some(vec![seller_check, buyer_check])
    }

    /// Generate a compliance report for a date range.
    pub async fn generate_report(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> anyhow::Result<ComplianceReport> {
        let checks = self.db.get_compliance_checks_in_range(from, to).await?;
        Ok(reporting::build_report(checks, from, to))
    }

    /// Get compliance status for a specific address.
    pub async fn get_address_status(&self, address: &str) -> Option<ComplianceCheck> {
        self.db.get_latest_compliance_check(address).await.ok().flatten()
    }

    /// Manual review override — update a compliance check status.
    pub async fn review_check(
        &self,
        check_id: Uuid,
        status: ComplianceStatus,
        reviewer: &str,
        notes: &str,
    ) -> anyhow::Result<()> {
        self.db
            .update_compliance_review(check_id, &status, reviewer, notes)
            .await
    }
    fn determine_status(&self, kyc: &KycResult, aml: &AmlResult) -> ComplianceStatus {
        if aml.is_blocked || kyc.status == KycStatus::Rejected {
            return ComplianceStatus::Blocked;
        }
        if aml.risk_score >= self.config.aml_risk_threshold {
            return ComplianceStatus::RequiresReview;
        }
        if kyc.status == KycStatus::Pending {
            return ComplianceStatus::Pending;
        }
        if kyc.level < self.config.required_kyc_level {
            return ComplianceStatus::RequiresReview;
        }
        ComplianceStatus::Approved
    }

    fn calculate_risk_score(&self, kyc: &KycResult, aml: &AmlResult) -> u8 {
        let kyc_penalty: u8 = match kyc.status {
            KycStatus::Verified => 0,
            KycStatus::Pending => 20,
            KycStatus::Rejected => 50,
            KycStatus::Unverified => 30,
        };
        aml.risk_score.saturating_add(kyc_penalty).min(100)
    }
}
