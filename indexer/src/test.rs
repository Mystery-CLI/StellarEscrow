#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert!(config.database.url.contains("postgres"));
        assert_eq!(config.stellar.network, "testnet");
    }

    #[test]
    fn test_config_validation_passes_for_defaults() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_fails_empty_db_url() {
        let mut config = Config::default();
        config.database.url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_fails_bad_network() {
        let mut config = Config::default();
        config.stellar.network = "devnet".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_fails_production_without_mainnet() {
        let mut config = Config::default();
        config.meta.environment = "production".to_string();
        // network is still "testnet" — should fail
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_health_liveness() {
        let response = crate::health::liveness().await;
        let body = response.0;
        assert_eq!(body["status"], "ok");
    }
}
