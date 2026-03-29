#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::config::{ConnectorConfig, ConnectorKind, IntegrationConfig};
    use crate::integration_service::IntegrationService;
    use crate::models::Event;

    fn make_event(event_type: &str) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            category: "trade".to_string(),
            schema_version: 2,
            contract_id: "CTEST".to_string(),
            ledger: 1000,
            transaction_hash: "abc123".to_string(),
            timestamp: Utc::now(),
            data: json!({"trade_id": 1}),
            created_at: Utc::now(),
        }
    }

    fn make_service(url: &str, filter: Vec<String>) -> IntegrationService {
        IntegrationService::new_without_db(IntegrationConfig {
            connectors: vec![ConnectorConfig {
                id: "test-connector".to_string(),
                name: "Test".to_string(),
                kind: ConnectorKind::Webhook,
                url: url.to_string(),
                auth_token: None,
                event_filter: filter,
                timeout_secs: 5,
            }],
        })
    }

    #[tokio::test]
    async fn test_successful_delivery() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/hook"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let svc = make_service(&format!("{}/hook", server.uri()), vec![]);
        svc.process_event(&make_event("trade_created")).await;

        let stats = svc.get_stats().await;
        let s = &stats["test-connector"];
        assert_eq!(s.success, 1);
        assert_eq!(s.failed, 0);
    }

    #[tokio::test]
    async fn test_failed_delivery_increments_failed_count() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/hook"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let svc = make_service(&format!("{}/hook", server.uri()), vec![]);
        svc.process_event(&make_event("trade_funded")).await;

        let stats = svc.get_stats().await;
        let s = &stats["test-connector"];
        assert_eq!(s.failed, 1);
        assert!(s.last_error.is_some());
    }

    #[tokio::test]
    async fn test_event_filter_skips_unmatched_events() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/hook"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;

        let svc = make_service(
            &format!("{}/hook", server.uri()),
            vec!["trade_created".to_string()],
        );
        svc.process_event(&make_event("trade_funded")).await; // not in filter

        let stats = svc.get_stats().await;
        assert_eq!(stats["test-connector"].total, 0);
    }

    #[tokio::test]
    async fn test_auth_token_sent_in_header() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/secure"))
            .and(header("Authorization", "Bearer secret-token"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let svc = IntegrationService::new_without_db(IntegrationConfig {
            connectors: vec![ConnectorConfig {
                id: "auth-connector".to_string(),
                name: "Auth Test".to_string(),
                kind: ConnectorKind::Http,
                url: format!("{}/secure", server.uri()),
                auth_token: Some("secret-token".to_string()),
                event_filter: vec![],
                timeout_secs: 5,
            }],
        });
        svc.process_event(&make_event("trade_created")).await;

        let stats = svc.get_stats().await;
        assert_eq!(stats["auth-connector"].success, 1);
    }
}
