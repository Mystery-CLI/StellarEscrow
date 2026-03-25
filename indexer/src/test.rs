#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert!(config.database.url.contains("postgres"));
        assert_eq!(config.stellar.network, "testnet");
    }

    #[tokio::test]
    async fn test_health_liveness() {
        // liveness is a simple no-dep handler; just verify it returns ok status
        let response = crate::health::liveness().await;
        let body = response.0;
        assert_eq!(body["status"], "ok");
    }
}