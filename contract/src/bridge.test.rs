#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    #[test]
    fn test_register_bridge_provider() {
        let env = Env::default();
        let provider_addr = Address::random(&env);

        let config = BridgeProviderConfig {
            provider: BridgeProvider::Wormhole,
            oracle_address: provider_addr.clone(),
            is_active: true,
            fee_bps: 50,
            supported_chains: String::from_small_copy(&env, "ethereum,polygon"),
            max_trade_amount: 1_000_000_000_000,
            min_trade_amount: 1_000_000,
        };

        let result = register_bridge_provider(&env, config);
        assert!(result.is_ok());

        // Verify provider is registered
        let providers = get_bridge_providers(&env);
        assert_eq!(providers.len(), 1);
    }

    #[test]
    fn test_duplicate_provider_registration_fails() {
        let env = Env::default();
        let provider_addr = Address::random(&env);

        let config = BridgeProviderConfig {
            provider: BridgeProvider::Wormhole,
            oracle_address: provider_addr.clone(),
            is_active: true,
            fee_bps: 50,
            supported_chains: String::from_small_copy(&env, "ethereum"),
            max_trade_amount: 1_000_000_000_000,
            min_trade_amount: 1_000_000,
        };

        register_bridge_provider(&env, config.clone()).unwrap();

        // Try to register same provider again
        let result = register_bridge_provider(&env, config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_cross_chain_trade() {
        let env = Env::default();
        let provider_addr = Address::random(&env);

        // Register provider first
        let config = BridgeProviderConfig {
            provider: BridgeProvider::Wormhole,
            oracle_address: provider_addr.clone(),
            is_active: true,
            fee_bps: 50,
            supported_chains: String::from_small_copy(&env, "ethereum"),
            max_trade_amount: 1_000_000_000_000,
            min_trade_amount: 1_000_000,
        };

        register_bridge_provider(&env, config).unwrap();

        // Create cross-chain trade
        let trade = create_cross_chain_trade(
            &env,
            1,
            String::from_small_copy(&env, "ethereum"),
            String::from_small_copy(&env, "0x1234"),
            BridgeProvider::Wormhole,
            String::from_small_copy(&env, "att-123"),
            50_000_000,
        );

        assert!(trade.is_ok());
        let trade = trade.unwrap();
        assert_eq!(trade.trade_id, 1);
        assert_eq!(trade.attestation_status, AttestationStatus::Pending);
    }

    #[test]
    fn test_update_attestation_status() {
        let env = Env::default();
        let provider_addr = Address::random(&env);

        // Setup
        let config = BridgeProviderConfig {
            provider: BridgeProvider::Wormhole,
            oracle_address: provider_addr.clone(),
            is_active: true,
            fee_bps: 50,
            supported_chains: String::from_small_copy(&env, "ethereum"),
            max_trade_amount: 1_000_000_000_000,
            min_trade_amount: 1_000_000,
        };

        register_bridge_provider(&env, config).unwrap();

        let _trade = create_cross_chain_trade(
            &env,
            1,
            String::from_small_copy(&env, "ethereum"),
            String::from_small_copy(&env, "0x1234"),
            BridgeProvider::Wormhole,
            String::from_small_copy(&env, "att-123"),
            50_000_000,
        )
        .unwrap();

        // Update status
        let result = update_attestation_status(
            &env,
            1,
            AttestationStatus::Confirmed,
            15,
        );

        assert!(result.is_ok());

        // Verify status updated
        let trade = get_cross_chain_trade(&env, 1).unwrap();
        assert_eq!(trade.attestation_status, AttestationStatus::Confirmed);
        assert_eq!(trade.current_confirmations, 15);
    }

    #[test]
    fn test_validate_bridge_attestation() {
        let env = Env::default();
        let provider_addr = Address::random(&env);
        let recipient = Address::random(&env);

        // Register provider
        let config = BridgeProviderConfig {
            provider: BridgeProvider::Wormhole,
            oracle_address: provider_addr.clone(),
            is_active: true,
            fee_bps: 50,
            supported_chains: String::from_small_copy(&env, "ethereum"),
            max_trade_amount: 1_000_000_000_000,
            min_trade_amount: 1_000_000,
        };

        register_bridge_provider(&env, config).unwrap();

        // Create attestation
        let attestation = BridgeAttestation {
            attestation_id: String::from_small_copy(&env, "att-123"),
            trade_id: 1,
            source_chain: String::from_small_copy(&env, "ethereum"),
            source_tx_hash: String::from_small_copy(&env, "0x1234"),
            amount: 1_000_000_000,
            recipient: recipient.clone(),
            timestamp: env.ledger().timestamp(),
            signature: soroban_sdk::Vec::new(&env),
            provider: BridgeProvider::Wormhole,
        };

        // Validate (should fail - empty signature)
        let result = validate_bridge_attestation(&env, &attestation);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.valid);
    }

    #[test]
    fn test_retry_bridge_attestation() {
        let env = Env::default();
        let provider_addr = Address::random(&env);

        // Setup
        let config = BridgeProviderConfig {
            provider: BridgeProvider::Wormhole,
            oracle_address: provider_addr.clone(),
            is_active: true,
            fee_bps: 50,
            supported_chains: String::from_small_copy(&env, "ethereum"),
            max_trade_amount: 1_000_000_000_000,
            min_trade_amount: 1_000_000,
        };

        register_bridge_provider(&env, config).unwrap();

        let _trade = create_cross_chain_trade(
            &env,
            1,
            String::from_small_copy(&env, "ethereum"),
            String::from_small_copy(&env, "0x1234"),
            BridgeProvider::Wormhole,
            String::from_small_copy(&env, "att-123"),
            50_000_000,
        )
        .unwrap();

        // Retry
        let result = retry_bridge_attestation(&env, 1);
        assert!(result.is_ok());

        // Verify retry count incremented
        let trade = get_cross_chain_trade(&env, 1).unwrap();
        assert_eq!(trade.retry_count, 1);
        assert_eq!(trade.attestation_status, AttestationStatus::Pending);
    }

    #[test]
    fn test_retry_limit_exceeded() {
        let env = Env::default();
        let provider_addr = Address::random(&env);

        // Setup
        let config = BridgeProviderConfig {
            provider: BridgeProvider::Wormhole,
            oracle_address: provider_addr.clone(),
            is_active: true,
            fee_bps: 50,
            supported_chains: String::from_small_copy(&env, "ethereum"),
            max_trade_amount: 1_000_000_000_000,
            min_trade_amount: 1_000_000,
        };

        register_bridge_provider(&env, config).unwrap();

        let _trade = create_cross_chain_trade(
            &env,
            1,
            String::from_small_copy(&env, "ethereum"),
            String::from_small_copy(&env, "0x1234"),
            BridgeProvider::Wormhole,
            String::from_small_copy(&env, "att-123"),
            50_000_000,
        )
        .unwrap();

        // Retry 3 times
        for _ in 0..3 {
            retry_bridge_attestation(&env, 1).unwrap();
        }

        // Fourth retry should fail
        let result = retry_bridge_attestation(&env, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_pause_and_resume_bridge() {
        let env = Env::default();

        assert!(!is_bridge_paused(&env));

        pause_bridge(&env);
        assert!(is_bridge_paused(&env));

        resume_bridge(&env);
        assert!(!is_bridge_paused(&env));
    }

    #[test]
    fn test_deactivate_bridge_provider() {
        let env = Env::default();
        let provider_addr = Address::random(&env);

        let config = BridgeProviderConfig {
            provider: BridgeProvider::Wormhole,
            oracle_address: provider_addr.clone(),
            is_active: true,
            fee_bps: 50,
            supported_chains: String::from_small_copy(&env, "ethereum"),
            max_trade_amount: 1_000_000_000_000,
            min_trade_amount: 1_000_000,
        };

        register_bridge_provider(&env, config).unwrap();

        // Deactivate
        let result = deactivate_bridge_provider(&env, &BridgeProvider::Wormhole);
        assert!(result.is_ok());

        // Verify deactivated
        let result = get_bridge_provider(&env, &BridgeProvider::Wormhole);
        assert!(result.is_err());
    }

    #[test]
    fn test_bridge_nonce_increment() {
        let env = Env::default();

        let nonce1 = get_next_bridge_nonce(&env).unwrap();
        let nonce2 = get_next_bridge_nonce(&env).unwrap();

        assert_eq!(nonce1, 1);
        assert_eq!(nonce2, 2);
    }
}
