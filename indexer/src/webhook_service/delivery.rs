use super::{WebhookDeliveryRecord, WebhookEndpoint};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;
use chrono::Utc;

type HmacSha256 = Hmac<Sha256>;

/// Deliver a webhook payload with exponential backoff retry.
pub async fn deliver_with_retry(
    http: &reqwest::Client,
    endpoint: &WebhookEndpoint,
    payload: &serde_json::Value,
    max_attempts: u32,
) -> WebhookDeliveryRecord {
    let body = serde_json::to_string(payload).unwrap_or_default();
    let signature = sign_payload(&body, &endpoint.secret);

    let mut last_record = WebhookDeliveryRecord {
        id: Uuid::new_v4(),
        endpoint_id: endpoint.id,
        event_type: payload["event_type"].as_str().unwrap_or("unknown").to_string(),
        payload: payload.clone(),
        status_code: None,
        success: false,
        attempt: 0,
        error: None,
        delivered_at: Utc::now(),
        duration_ms: 0,
    };

    for attempt in 1..=max_attempts {
        let start = std::time::Instant::now();

        let result = http
            .post(&endpoint.url)
            .header("Content-Type", "application/json")
            .header("X-StellarEscrow-Signature", &signature)
            .header("X-StellarEscrow-Event", payload["event_type"].as_str().unwrap_or(""))
            .header("X-StellarEscrow-Delivery", last_record.id.to_string())
            .body(body.clone())
            .send()
            .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let success = resp.status().is_success();
                last_record.status_code = Some(status);
                last_record.success = success;
                last_record.attempt = attempt;
                last_record.duration_ms = duration_ms;
                last_record.delivered_at = Utc::now();

                if success {
                    return last_record;
                }
                last_record.error = Some(format!("HTTP {}", status));
            }
            Err(e) => {
                last_record.error = Some(e.to_string());
                last_record.attempt = attempt;
                last_record.duration_ms = duration_ms;
            }
        }

        if attempt < max_attempts {
            // Exponential backoff: 1s, 2s, 4s
            let delay = std::time::Duration::from_secs(2u64.pow(attempt - 1));
            tokio::time::sleep(delay).await;
        }
    }

    last_record
}

/// Sign a payload with HMAC-SHA256 using the endpoint secret.
/// Header format: sha256=<hex>
pub fn sign_payload(body: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key size");
    mac.update(body.as_bytes());
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}
