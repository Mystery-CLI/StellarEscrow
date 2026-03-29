use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KycStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycResult {
    pub status: KycStatus,
    /// KYC level achieved: 0 = none, 1 = basic (ID), 2 = enhanced (ID + liveness)
    pub level: u8,
    pub provider: String,
    pub reference_id: Option<String>,
    pub jurisdiction: Option<String>,
    pub failure_reason: Option<String>,
}

/// KYC provider integration (Jumio / Onfido compatible interface).
pub struct KycProvider {
    api_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl KycProvider {
    pub fn new(api_url: &str, api_key: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Verify an address/identity against the KYC provider.
    /// In production this would call the real provider API.
    pub async fn verify(&self, address: &str) -> KycResult {
        if self.api_url.is_empty() || self.api_key.is_empty() {
            // No provider configured — return unverified
            return KycResult {
                status: KycStatus::Unverified,
                level: 0,
                provider: "none".to_string(),
                reference_id: None,
                jurisdiction: None,
                failure_reason: Some("KYC provider not configured".to_string()),
            };
        }

        match self.call_provider(address).await {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("KYC provider error for {}: {}", address, e);
                KycResult {
                    status: KycStatus::Pending,
                    level: 0,
                    provider: "error".to_string(),
                    reference_id: None,
                    jurisdiction: None,
                    failure_reason: Some(e.to_string()),
                }
            }
        }
    }

    async fn call_provider(&self, address: &str) -> anyhow::Result<KycResult> {
        let url = format!("{}/v1/verify", self.api_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({ "address": address }))
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("KYC provider returned {}", resp.status());
        }

        let body: serde_json::Value = resp.json().await?;
        let status = match body["status"].as_str().unwrap_or("pending") {
            "verified" => KycStatus::Verified,
            "rejected" => KycStatus::Rejected,
            _ => KycStatus::Pending,
        };

        Ok(KycResult {
            status,
            level: body["level"].as_u64().unwrap_or(0) as u8,
            provider: body["provider"].as_str().unwrap_or("unknown").to_string(),
            reference_id: body["reference_id"].as_str().map(|s| s.to_string()),
            jurisdiction: body["jurisdiction"].as_str().map(|s| s.to_string()),
            failure_reason: body["failure_reason"].as_str().map(|s| s.to_string()),
        })
    }
}
