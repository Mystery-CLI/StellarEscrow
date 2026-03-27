use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlResult {
    /// Risk score 0–100 (higher = riskier)
    pub risk_score: u8,
    /// Whether the address is on a sanctions/blocklist
    pub is_blocked: bool,
    /// Matched sanctions lists (e.g. "OFAC", "EU", "UN")
    pub sanctions_matches: Vec<String>,
    /// Exposure categories (e.g. "darknet", "mixer", "exchange")
    pub exposure_categories: Vec<String>,
    pub provider: String,
    pub reference_id: Option<String>,
}

/// AML screening service (Chainalysis / Elliptic compatible interface).
pub struct AmlScreener {
    api_url: String,
    api_key: String,
    risk_threshold: u8,
    blocked_jurisdictions: Vec<String>,
    client: reqwest::Client,
}

impl AmlScreener {
    pub fn new(
        api_url: &str,
        api_key: &str,
        risk_threshold: u8,
        blocked_jurisdictions: Vec<String>,
    ) -> Self {
        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            risk_threshold,
            blocked_jurisdictions,
            client: reqwest::Client::new(),
        }
    }

    /// Screen an address for AML risk.
    pub async fn screen(&self, address: &str) -> AmlResult {
        if self.api_url.is_empty() || self.api_key.is_empty() {
            return AmlResult {
                risk_score: 0,
                is_blocked: false,
                sanctions_matches: vec![],
                exposure_categories: vec![],
                provider: "none".to_string(),
                reference_id: None,
            };
        }

        match self.call_provider(address).await {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("AML provider error for {}: {}", address, e);
                // Fail open with a medium risk score to flag for review
                AmlResult {
                    risk_score: 50,
                    is_blocked: false,
                    sanctions_matches: vec![],
                    exposure_categories: vec!["provider_error".to_string()],
                    provider: "error".to_string(),
                    reference_id: None,
                }
            }
        }
    }

    async fn call_provider(&self, address: &str) -> anyhow::Result<AmlResult> {
        let url = format!("{}/v1/screen", self.api_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({ "address": address, "asset": "XLM" }))
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("AML provider returned {}", resp.status());
        }

        let body: serde_json::Value = resp.json().await?;
        let risk_score = body["risk_score"].as_u64().unwrap_or(0).min(100) as u8;
        let sanctions_matches: Vec<String> = body["sanctions_matches"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let is_blocked = !sanctions_matches.is_empty() || risk_score >= self.risk_threshold;

        let exposure_categories: Vec<String> = body["exposure_categories"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Ok(AmlResult {
            risk_score,
            is_blocked,
            sanctions_matches,
            exposure_categories,
            provider: body["provider"].as_str().unwrap_or("unknown").to_string(),
            reference_id: body["reference_id"].as_str().map(|s| s.to_string()),
        })
    }
}
