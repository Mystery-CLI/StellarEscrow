use crate::error::MobileSdkError;
use crate::types::{Platform, PushRegistration};

/// Register a device for push notifications about trade events.
/// Sends the device token + Stellar address to the indexer's notification endpoint.
pub fn register_push(
    indexer_url: &str,
    registration: &PushRegistration,
) -> Result<(), MobileSdkError> {
    let payload = serde_json::to_string(registration)
        .map_err(|e| MobileSdkError::Serialization(e.to_string()))?;

    // In a real mobile app this would be an async HTTP POST.
    // Kept sync here so the SDK has no async runtime dependency.
    let url = format!("{}/push/register", indexer_url.trim_end_matches('/'));
    let _ = (url, payload); // placeholder — replace with platform HTTP client
    Ok(())
}

/// Unregister a device token.
pub fn unregister_push(indexer_url: &str, device_token: &str) -> Result<(), MobileSdkError> {
    let url = format!(
        "{}/push/unregister/{}",
        indexer_url.trim_end_matches('/'),
        device_token
    );
    let _ = url; // placeholder
    Ok(())
}

/// Map a trade event type to a human-readable push notification body.
pub fn notification_body(event_type: &str, trade_id: u64) -> String {
    match event_type {
        "funded" => format!("Trade #{trade_id} has been funded by the buyer."),
        "complete" => format!("Trade #{trade_id} marked complete — confirm receipt."),
        "confirm" => format!("Trade #{trade_id} settled. Funds released."),
        "dispute" => format!("Trade #{trade_id} is under dispute."),
        "resolved" => format!("Trade #{trade_id} dispute resolved."),
        "cancel" => format!("Trade #{trade_id} was cancelled."),
        _ => format!("Trade #{trade_id} updated."),
    }
}

/// Returns the platform-specific push service name for logging/debugging.
pub fn push_service_name(platform: &Platform) -> &'static str {
    match platform {
        Platform::Ios => "APNs",
        Platform::Android => "FCM",
    }
}
