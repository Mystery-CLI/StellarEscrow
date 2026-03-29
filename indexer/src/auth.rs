use crate::config::AuthConfig;
use axum::body::Body;
use axum::extract::{FromRef, State};
use axum::http::{header, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

const PUBLIC_PATHS: &[&str] = &[
    "/",
    "/health",
    "/health/live",
    "/health/ready",
    "/health/metrics",
    "/health/alerts",
    "/status",
    "/help",
    "/ws",
    "/api/v1/docs",
];

fn normalize_api_key(value: &str) -> Option<&str> {
    if let Some(stripped) = value.strip_prefix("Bearer ") {
        Some(stripped)
    } else {
        Some(value)
    }
}

pub async fn auth_middleware(
    State(auth_config): State<Arc<AuthConfig>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path();

    if PUBLIC_PATHS
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{}/", p)))
    {
        return next.run(req).await;
    }

    let api_key = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|hv| hv.to_str().ok())
        .and_then(normalize_api_key)
        .or_else(|| {
            req.headers()
                .get("x-api-key")
                .and_then(|hv| hv.to_str().ok())
        });

    let bad_key_response = (
        StatusCode::UNAUTHORIZED,
        "Unauthorized: missing or invalid API key",
    )
        .into_response();

    let key = match api_key {
        Some(k) if !k.is_empty() => k,
        _ => return bad_key_response,
    };

    let is_admin_route = path.starts_with("/admin") || path.starts_with("/api/v1/admin");

    if is_admin_route {
        if auth_config.admin_keys.iter().any(|ak| ak == key) {
            return next.run(req).await;
        } else {
            return (StatusCode::FORBIDDEN, "Forbidden: admin key required").into_response();
        }
    }

    if auth_config.api_keys.iter().any(|ak| ak == key)
        || auth_config.admin_keys.iter().any(|ak| ak == key)
    {
        next.run(req).await
    } else {
        bad_key_response
    }
}
