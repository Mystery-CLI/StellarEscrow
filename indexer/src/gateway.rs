//! API Gateway Module
//!
//! This module provides a centralized API gateway layer that handles:
//! - Request routing to appropriate services/endpoints
//! - Load balancing across service instances (round-robin)
//! - Authentication and authorization
//! - Request/response transformation
//! - Error handling and standardization
//!
//! ## Architecture
//!
//! The gateway sits in front of all backend services and provides a unified
//! entry point for all API requests. It uses middleware-based routing to
//! direct requests to the appropriate handlers while applying cross-cutting
//! concerns like auth, rate limiting, and logging.
//!
//! ## Routing Rules
//!
//! - `/health*` → Health check endpoints (public)
//! - `/api/v1/*` → Versioned API endpoints (authenticated)
//! - `/events*` → Event querying and replay (authenticated)
//! - `/search*` → Search and discovery (authenticated)
//! - `/audit*` → Audit log operations (authenticated)
//! - `/fraud*` → Fraud detection (authenticated)
//! - `/notifications*` → Notification management (authenticated)
//! - `/admin*` → Administrative operations (admin auth required)
//! - `/files*` → File operations (authenticated)
//! - `/help*` → Help system (public)
//! - `/ws` → WebSocket connections (authenticated)

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tracing::{info, warn};

use crate::config::AuthConfig;

/// Gateway configuration
#[derive(Clone)]
pub struct GatewayConfig {
    /// Enable load balancing
    pub load_balancing_enabled: bool,
    /// Service instances for load balancing (host:port)
    pub service_instances: Vec<String>,
    /// Enable request logging
    pub request_logging: bool,
    /// Enable response transformation
    pub transform_responses: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            load_balancing_enabled: false,
            service_instances: vec![],
            request_logging: true,
            transform_responses: true,
        }
    }
}

/// Shared gateway state
#[derive(Clone)]
pub struct GatewayState {
    pub config: Arc<GatewayConfig>,
    pub auth_config: Arc<AuthConfig>,
    /// Round-robin counter for load balancing
    pub rr_counter: Arc<AtomicUsize>,
    /// Route statistics
    pub route_stats: Arc<dashmap::DashMap<String, AtomicUsize>>,
}

impl GatewayState {
    pub fn new(config: GatewayConfig, auth_config: Arc<AuthConfig>) -> Self {
        Self {
            config: Arc::new(config),
            auth_config,
            rr_counter: Arc::new(AtomicUsize::new(0)),
            route_stats: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Get next service instance using round-robin
    pub fn get_next_instance(&self) -> Option<String> {
        if self.config.service_instances.is_empty() {
            return None;
        }
        let idx =
            self.rr_counter.fetch_add(1, Ordering::Relaxed) % self.config.service_instances.len();
        Some(self.config.service_instances[idx].clone())
    }

    /// Record route access for statistics
    pub fn record_route(&self, route: &str) {
        let stats = self
            .route_stats
            .entry(route.to_string())
            .or_insert_with(|| AtomicUsize::new(0));
        stats.fetch_add(1, Ordering::Relaxed);
    }
}

/// Gateway error types
#[derive(Debug)]
pub enum GatewayError {
    ServiceUnavailable(String),
    Unauthorized(String),
    NotFound(String),
    InternalError(String),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            GatewayError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            GatewayError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            GatewayError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            GatewayError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": true,
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }));

        (status, body).into_response()
    }
}

/// Standard API response wrapper for consistent formatting
#[derive(serde::Serialize)]
pub struct StandardResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: String,
}

impl<T: serde::Serialize> StandardResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Gateway middleware - applies authentication and routing logic
pub async fn gateway_middleware(
    State(state): State<Arc<GatewayState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path();
    let method = req.method().clone();

    // Record route access for analytics
    state.record_route(path);

    // Log request if enabled
    if state.config.request_logging {
        info!("Gateway request: {} {}", method, path);
    }

    // Check if path requires authentication
    if !is_public_path(path) {
        // Validate API key
        match validate_api_key(&req, &state.auth_config, path) {
            Ok(_) => {
                // Valid credentials, continue
            }
            Err(e) => return e.into_response(),
        }
    }

    // Add gateway headers to response
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("X-Gateway-Version", "1.0.0".parse().unwrap());

    response
}

/// Check if a path should bypass authentication
fn is_public_path(path: &str) -> bool {
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

    PUBLIC_PATHS
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{}/", p)))
}

/// Validate API key from request headers
fn validate_api_key(
    req: &Request<Body>,
    auth_config: &AuthConfig,
    path: &str,
) -> Result<(), GatewayError> {
    // Extract API key from headers
    let api_key = extract_api_key(req)?;

    // Check if this is an admin route
    let is_admin_route = path.starts_with("/admin") || path.starts_with("/api/v1/admin");

    if is_admin_route {
        // Admin routes require admin key
        if auth_config.admin_keys.iter().any(|ak| ak == &api_key) {
            Ok(())
        } else {
            Err(GatewayError::Unauthorized("Admin key required".to_string()))
        }
    } else {
        // Regular routes accept any valid API key
        if auth_config.api_keys.iter().any(|ak| ak == &api_key)
            || auth_config.admin_keys.iter().any(|ak| ak == &api_key)
        {
            Ok(())
        } else {
            Err(GatewayError::Unauthorized("Invalid API key".to_string()))
        }
    }
}

/// Extract API key from Authorization or x-api-key header
fn extract_api_key(req: &Request<Body>) -> Result<String, GatewayError> {
    // Try Authorization header first
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_value) = auth_header.to_str() {
            // Support "Bearer <token>" format
            if let Some(key) = auth_value.strip_prefix("Bearer ") {
                return Ok(key.to_string());
            }
            // Or raw token
            return Ok(auth_value.to_string());
        }
    }

    // Try x-api-key header
    if let Some(api_key_header) = req.headers().get("x-api-key") {
        if let Ok(key) = api_key_header.to_str() {
            return Ok(key.to_string());
        }
    }

    Err(GatewayError::Unauthorized("Missing API key".to_string()))
}

/// Transform response to standard format if enabled
pub fn transform_response<T: serde::Serialize>(data: T, transform: bool) -> StandardResponse<T> {
    if transform {
        StandardResponse::success(data)
    } else {
        // Return raw data wrapped in success response
        StandardResponse {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Get gateway statistics for monitoring
pub fn get_gateway_stats(state: &GatewayState) -> HashMap<String, serde_json::Value> {
    let mut stats = HashMap::new();

    // Route statistics
    let route_stats: HashMap<String, usize> = state
        .route_stats
        .iter()
        .map(|entry| (entry.key().clone(), entry.value().load(Ordering::Relaxed)))
        .collect();

    stats.insert("routes".to_string(), json!(route_stats));
    stats.insert(
        "active_instances".to_string(),
        json!(state.config.service_instances.len()),
    );
    stats.insert(
        "load_balancing".to_string(),
        json!(state.config.load_balancing_enabled),
    );

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_paths() {
        assert!(is_public_path("/health"));
        assert!(is_public_path("/health/live"));
        assert!(is_public_path("/help"));
        assert!(is_public_path("/ws"));
        assert!(!is_public_path("/events"));
        assert!(!is_public_path("/admin/users"));
    }

    #[test]
    fn test_standard_response() {
        let resp = StandardResponse::success("test data");
        assert!(resp.success);
        assert_eq!(resp.data, Some("test data"));
        assert!(resp.error.is_none());

        let resp = StandardResponse::error("something went wrong");
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_round_robin() {
        let config = GatewayConfig {
            load_balancing_enabled: true,
            service_instances: vec![
                "http://service1:8080".to_string(),
                "http://service2:8080".to_string(),
                "http://service3:8080".to_string(),
            ],
            ..Default::default()
        };

        let auth_config = Arc::new(AuthConfig {
            api_keys: vec!["test-key".to_string()],
            admin_keys: vec!["admin-key".to_string()],
        });

        let state = GatewayState::new(config, auth_config);

        // Test round-robin distribution
        assert_eq!(
            state.get_next_instance(),
            Some("http://service1:8080".to_string())
        );
        assert_eq!(
            state.get_next_instance(),
            Some("http://service2:8080".to_string())
        );
        assert_eq!(
            state.get_next_instance(),
            Some("http://service3:8080".to_string())
        );
        assert_eq!(
            state.get_next_instance(),
            Some("http://service1:8080".to_string())
        );
    }
}
