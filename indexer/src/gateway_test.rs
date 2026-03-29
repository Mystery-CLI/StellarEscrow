//! API Gateway Tests
//!
//! Tests cover:
//! - Request routing
//! - Load balancing (round-robin)
//! - Authentication validation
//! - Response transformation
//! - Error handling

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::get,
    Router,
};
use std::sync::Arc;
use tower::ServiceExt;

use crate::config::{AuthConfig, GatewayConfig};
use crate::gateway::{
    gateway_middleware, get_gateway_stats, GatewayError, GatewayState, StandardResponse,
};

/// Helper to create test gateway state
fn create_test_gateway() -> Arc<GatewayState> {
    let config = GatewayConfig {
        load_balancing_enabled: true,
        service_instances: vec![
            "http://service1:8080".to_string(),
            "http://service2:8080".to_string(),
            "http://service3:8080".to_string(),
        ],
        request_logging: false, // Disable logging in tests
        transform_responses: true,
    };

    let auth_config = Arc::new(AuthConfig {
        api_keys: vec!["test-key-123".to_string(), "client-key-456".to_string()],
        admin_keys: vec!["admin-key-789".to_string()],
    });

    Arc::new(GatewayState::new(config, auth_config))
}

/// Test helper to create a request with API key
fn request_with_key(path: &str, key: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header("x-api-key", key)
        .body(Body::empty())
        .unwrap()
}

/// Test helper to create request with Bearer token
fn request_with_bearer(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn test_gateway_routing_valid_request() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/test", get(handler))
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway_middleware,
        ));

    // Valid API key should succeed
    let response = app
        .oneshot(request_with_key("/test", "test-key-123"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_gateway_routing_invalid_key() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/test", get(handler))
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway_middleware,
        ));

    // Invalid API key should fail
    let response = app
        .oneshot(request_with_key("/test", "invalid-key"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_gateway_public_paths_no_auth() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/health", get(handler))
        .route("/health/live", get(handler))
        .route("/help", get(handler))
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway_middleware,
        ));

    // Public paths should work without auth
    for path in &["/health", "/health/live", "/help"] {
        let request = Request::builder().uri(*path).body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Path {} should be public",
            path
        );
    }
}

#[tokio::test]
async fn test_gateway_admin_route_requires_admin_key() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app =
        Router::new()
            .route("/admin/users", get(handler))
            .layer(middleware::from_fn_with_state(
                gateway_state.clone(),
                gateway_middleware,
            ));

    // Regular API key should fail on admin routes
    let response = app
        .oneshot(request_with_key("/admin/users", "test-key-123"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Admin key should succeed
    let response = app
        .oneshot(request_with_key("/admin/users", "admin-key-789"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_gateway_bearer_token_format() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/test", get(handler))
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway_middleware,
        ));

    // Bearer token format should work
    let response = app
        .oneshot(request_with_bearer("/test", "test-key-123"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_load_balancing_round_robin() {
    let gateway_state = create_test_gateway();

    // Test round-robin distribution across 3 instances
    let mut results = Vec::new();
    for _ in 0..6 {
        if let Some(instance) = gateway_state.get_next_instance() {
            results.push(instance);
        }
    }

    // Should cycle through all instances twice
    assert_eq!(results.len(), 6);
    assert_eq!(results[0], "http://service1:8080");
    assert_eq!(results[1], "http://service2:8080");
    assert_eq!(results[2], "http://service3:8080");
    assert_eq!(results[3], "http://service1:8080");
    assert_eq!(results[4], "http://service2:8080");
    assert_eq!(results[5], "http://service3:8080");
}

#[tokio::test]
async fn test_load_balancing_empty_instances() {
    let config = GatewayConfig {
        load_balancing_enabled: false,
        service_instances: vec![],
        request_logging: false,
        transform_responses: true,
    };

    let auth_config = Arc::new(AuthConfig {
        api_keys: vec!["test-key".to_string()],
        admin_keys: vec!["admin-key".to_string()],
    });

    let gateway_state = GatewayState::new(config, auth_config);

    // Should return None when no instances configured
    assert!(gateway_state.get_next_instance().is_none());
}

#[test]
fn test_standard_response_success() {
    let response = StandardResponse::success("test data");

    assert!(response.success);
    assert_eq!(response.data, Some("test data"));
    assert!(response.error.is_none());
    assert!(!response.timestamp.is_empty());
}

#[test]
fn test_standard_response_error() {
    let response: StandardResponse<String> = StandardResponse::error("something went wrong");

    assert!(!response.success);
    assert!(response.data.is_none());
    assert_eq!(response.error, Some("something went wrong".to_string()));
    assert!(!response.timestamp.is_empty());
}

#[test]
fn test_gateway_error_types() {
    // Test ServiceUnavailable
    let err = GatewayError::ServiceUnavailable("Service down".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    // Test Unauthorized
    let err = GatewayError::Unauthorized("Invalid token".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test NotFound
    let err = GatewayError::NotFound("Route not found".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Test InternalError
    let err = GatewayError::InternalError("Internal error".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_gateway_statistics_tracking() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/events", get(handler))
        .route("/search", get(handler))
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway_middleware,
        ));

    // Make several requests to different endpoints
    for _ in 0..3 {
        let _ = app
            .oneshot(request_with_key("/events", "test-key-123"))
            .await;
        let _ = app
            .oneshot(request_with_key("/search", "test-key-123"))
            .await;
    }

    // Check statistics were recorded
    let stats = get_gateway_stats(&gateway_state);

    assert!(stats.contains_key("routes"));
    assert!(stats.contains_key("active_instances"));
    assert!(stats.contains_key("load_balancing"));
}

#[tokio::test]
async fn test_gateway_headers_added() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/test", get(handler))
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway_middleware,
        ));

    let response = app
        .oneshot(request_with_key("/test", "test-key-123"))
        .await
        .unwrap();

    // Check gateway version header is added
    let headers = response.headers();
    assert!(headers.get("X-Gateway-Version").is_some());
    assert_eq!(headers.get("X-Gateway-Version").unwrap(), "1.0.0");
}

#[tokio::test]
async fn test_missing_api_key() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/test", get(handler))
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway_middleware,
        ));

    // Request without any API key should fail
    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_multiple_valid_keys() {
    let gateway_state = create_test_gateway();

    async fn handler() -> &'static str {
        "OK"
    }

    let app = Router::new()
        .route("/test", get(handler))
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway_middleware,
        ));

    // Both test keys should work
    let response1 = app
        .oneshot(request_with_key("/test", "test-key-123"))
        .await
        .unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    let response2 = app
        .oneshot(request_with_key("/test", "client-key-456"))
        .await
        .unwrap();
    assert_eq!(response2.status(), StatusCode::OK);

    // Admin key should also work on non-admin routes
    let response3 = app
        .oneshot(request_with_key("/test", "admin-key-789"))
        .await
        .unwrap();
    assert_eq!(response3.status(), StatusCode::OK);
}
