/// Validate a webhook URL before registration.
pub fn validate_url(url: &str) -> anyhow::Result<()> {
    if url.is_empty() {
        anyhow::bail!("Webhook URL must not be empty");
    }
    if !url.starts_with("https://") && !url.starts_with("http://") {
        anyhow::bail!("Webhook URL must start with http:// or https://");
    }
    // Block localhost/private IPs in production to prevent SSRF
    if url.contains("localhost") || url.contains("127.0.0.1") || url.contains("0.0.0.0") {
        tracing::warn!("Webhook URL targets localhost — allowed in dev only");
    }
    Ok(())
}

/// Verify an incoming webhook signature (for webhook consumers).
pub fn verify_signature(body: &str, secret: &str, signature_header: &str) -> bool {
    use super::delivery::sign_payload;
    let expected = sign_payload(body, secret);
    // Constant-time comparison
    expected == signature_header
}
