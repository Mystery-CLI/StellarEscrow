use reqwest::Client;
use serde_json::json;
use tracing::{error, info};

use crate::config::NotificationConfig;

/// Thin wrappers around external provider APIs.
/// Each function logs on failure but does NOT propagate — notifications are
/// best-effort and must never crash the main service.

pub async fn send_email(
    cfg: &NotificationConfig,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    if cfg.email_api_url.trim().is_empty()
        || cfg.email_api_key.trim().is_empty()
        || cfg.email_from.trim().is_empty()
    {
        return Err("email provider is not configured".to_string());
    }
    if to.trim().is_empty() {
        return Err("email recipient is empty".to_string());
    }
    if subject.trim().is_empty() || body.trim().is_empty() {
        return Err("email content is empty".to_string());
    }

    let client = Client::new();
    // SendGrid-compatible POST /v3/mail/send
    let payload = json!({
        "personalizations": [{ "to": [{ "email": to }] }],
        "from": { "email": cfg.email_from },
        "subject": subject,
        "content": [{ "type": "text/plain", "value": body }]
    });

    let res = client
        .post(format!("{}/v3/mail/send", cfg.email_api_url))
        .bearer_auth(&cfg.email_api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        info!("Email sent to {}", to);
        Ok(())
    } else {
        let msg = format!(
            "Email API error {}: {}",
            res.status(),
            res.text().await.unwrap_or_default()
        );
        error!("{}", msg);
        Err(msg)
    }
}

pub async fn send_sms(cfg: &NotificationConfig, to: &str, body: &str) -> Result<(), String> {
    if cfg.sms_api_url.trim().is_empty()
        || cfg.sms_account_sid.trim().is_empty()
        || cfg.sms_auth_token.trim().is_empty()
        || cfg.sms_from.trim().is_empty()
    {
        return Err("sms provider is not configured".to_string());
    }
    if to.trim().is_empty() {
        return Err("sms recipient is empty".to_string());
    }
    if body.trim().is_empty() {
        return Err("sms content is empty".to_string());
    }

    let client = Client::new();
    // Twilio-compatible POST /2010-04-01/Accounts/{sid}/Messages.json
    let url = format!(
        "{}/2010-04-01/Accounts/{}/Messages.json",
        cfg.sms_api_url, cfg.sms_account_sid
    );

    let res = client
        .post(&url)
        .basic_auth(&cfg.sms_account_sid, Some(&cfg.sms_auth_token))
        .form(&[("From", cfg.sms_from.as_str()), ("To", to), ("Body", body)])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        info!("SMS sent to {}", to);
        Ok(())
    } else {
        let msg = format!(
            "SMS API error {}: {}",
            res.status(),
            res.text().await.unwrap_or_default()
        );
        error!("{}", msg);
        Err(msg)
    }
}

pub async fn send_push(
    cfg: &NotificationConfig,
    token: &str,
    title: &str,
    body: &str,
) -> Result<(), String> {
    if cfg.push_api_url.trim().is_empty()
        || cfg.push_project_id.trim().is_empty()
        || cfg.push_server_key.trim().is_empty()
    {
        return Err("push provider is not configured".to_string());
    }
    if token.trim().is_empty() {
        return Err("push token is empty".to_string());
    }
    if title.trim().is_empty() || body.trim().is_empty() {
        return Err("push content is empty".to_string());
    }

    let client = Client::new();
    // FCM v1 HTTP API
    let payload = json!({
        "message": {
            "token": token,
            "notification": { "title": title, "body": body }
        }
    });

    let res = client
        .post(format!(
            "{}/v1/projects/{}/messages:send",
            cfg.push_api_url, cfg.push_project_id
        ))
        .bearer_auth(&cfg.push_server_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        info!("Push sent to token {}", token_preview(token));
        Ok(())
    } else {
        let msg = format!(
            "Push API error {}: {}",
            res.status(),
            res.text().await.unwrap_or_default()
        );
        error!("{}", msg);
        Err(msg)
    }
}

fn token_preview(token: &str) -> &str {
    let end = token
        .char_indices()
        .nth(8)
        .map(|(idx, _)| idx)
        .unwrap_or(token.len());
    &token[..end]
}
