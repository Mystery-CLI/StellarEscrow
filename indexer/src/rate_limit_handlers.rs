use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::json;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use crate::rate_limit::{RateLimitError, RateLimiter, RateTier};

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

/// Tower middleware that enforces rate limits on every request.
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let ip = addr.ip();

    match limiter.check(ip).await {
        Ok(headers) => {
            let mut response = next.run(req).await;
            let h = response.headers_mut();
            set_header(h, "X-RateLimit-Limit", headers.limit);
            set_header(h, "X-RateLimit-Remaining", headers.remaining);
            set_header(h, "X-RateLimit-Reset", headers.reset_secs);
            response
        }
        Err(RateLimitError::LimitExceeded { limit, reset_secs }) => {
            let mut resp = Json(json!({
                "error": "Too Many Requests",
                "message": "Rate limit exceeded. Please slow down.",
                "retry_after_seconds": reset_secs
            }))
            .into_response();
            *resp.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            let h = resp.headers_mut();
            set_header(h, "X-RateLimit-Limit", limit);
            set_header(h, "X-RateLimit-Remaining", 0u64);
            set_header(h, "X-RateLimit-Reset", reset_secs);
            set_header(h, "Retry-After", reset_secs);
            resp
        }
        Err(RateLimitError::Blacklisted) => {
            let mut resp = Json(json!({
                "error": "Forbidden",
                "message": "Your IP address has been blocked."
            }))
            .into_response();
            *resp.status_mut() = StatusCode::FORBIDDEN;
            resp
        }
    }
}

fn set_header(headers: &mut axum::http::HeaderMap, name: &'static str, value: u64) {
    if let (Ok(n), Ok(v)) = (
        HeaderName::from_static(name),
        HeaderValue::from_str(&value.to_string()),
    ) {
        headers.insert(n, v);
    }
}

// ---------------------------------------------------------------------------
// Admin handlers
// ---------------------------------------------------------------------------

/// GET /admin/rate-limits — monitoring snapshot
pub async fn get_rate_limit_stats(
    State(limiter): State<Arc<RateLimiter>>,
) -> impl IntoResponse {
    Json(limiter.snapshot().await)
}

#[derive(Deserialize)]
pub struct IpPayload {
    pub ip: IpAddr,
}

#[derive(Deserialize)]
pub struct TierPayload {
    pub ip: IpAddr,
    pub tier: RateTier,
}

/// POST /admin/rate-limits/whitelist — add IP to whitelist
pub async fn add_to_whitelist(
    State(limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<IpPayload>,
) -> impl IntoResponse {
    limiter.add_whitelist(payload.ip).await;
    Json(json!({ "whitelisted": payload.ip.to_string() }))
}

/// DELETE /admin/rate-limits/whitelist — remove IP from whitelist
pub async fn remove_from_whitelist(
    State(limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<IpPayload>,
) -> impl IntoResponse {
    limiter.remove_whitelist(payload.ip).await;
    Json(json!({ "removed_from_whitelist": payload.ip.to_string() }))
}

/// POST /admin/rate-limits/blacklist — add IP to blacklist
pub async fn add_to_blacklist(
    State(limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<IpPayload>,
) -> impl IntoResponse {
    limiter.add_blacklist(payload.ip).await;
    Json(json!({ "blacklisted": payload.ip.to_string() }))
}

/// DELETE /admin/rate-limits/blacklist — remove IP from blacklist
pub async fn remove_from_blacklist(
    State(limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<IpPayload>,
) -> impl IntoResponse {
    limiter.remove_blacklist(payload.ip).await;
    Json(json!({ "removed_from_blacklist": payload.ip.to_string() }))
}

/// POST /admin/rate-limits/tier — set tier override for an IP
pub async fn set_ip_tier(
    State(limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<TierPayload>,
) -> impl IntoResponse {
    limiter.set_tier(payload.ip, payload.tier).await;
    Json(json!({ "updated": payload.ip.to_string() }))
}
