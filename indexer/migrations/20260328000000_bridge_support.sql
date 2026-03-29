-- Bridge support tables for cross-chain trades

-- Bridge provider configurations
CREATE TABLE IF NOT EXISTS bridge_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_name VARCHAR(50) NOT NULL UNIQUE,
    oracle_address VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    fee_bps INTEGER NOT NULL DEFAULT 50,
    supported_chains TEXT NOT NULL,
    max_trade_amount BIGINT NOT NULL,
    min_trade_amount BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cross-chain trade metadata
CREATE TABLE IF NOT EXISTS cross_chain_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id BIGINT NOT NULL UNIQUE,
    source_chain VARCHAR(50) NOT NULL,
    dest_chain VARCHAR(50) NOT NULL DEFAULT 'stellar',
    source_tx_hash VARCHAR(100) NOT NULL,
    bridge_provider VARCHAR(50) NOT NULL,
    attestation_id VARCHAR(100) NOT NULL UNIQUE,
    attestation_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    attestation_timestamp TIMESTAMPTZ NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    min_confirmations INTEGER NOT NULL DEFAULT 12,
    current_confirmations INTEGER NOT NULL DEFAULT 0,
    bridge_fee BIGINT NOT NULL,
    bridge_metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (bridge_provider) REFERENCES bridge_providers(provider_name)
);

-- Bridge attestations
CREATE TABLE IF NOT EXISTS bridge_attestations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attestation_id VARCHAR(100) NOT NULL UNIQUE,
    trade_id BIGINT NOT NULL,
    source_chain VARCHAR(50) NOT NULL,
    source_tx_hash VARCHAR(100) NOT NULL,
    amount BIGINT NOT NULL,
    recipient VARCHAR(100) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    signature BYTEA NOT NULL,
    provider VARCHAR(50) NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT false,
    verification_timestamp TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (trade_id) REFERENCES cross_chain_trades(trade_id)
);

-- Bridge retry history
CREATE TABLE IF NOT EXISTS bridge_retry_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id BIGINT NOT NULL,
    retry_count INTEGER NOT NULL,
    error_message TEXT,
    retry_timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (trade_id) REFERENCES cross_chain_trades(trade_id)
);

-- Bridge provider health checks
CREATE TABLE IF NOT EXISTS bridge_provider_health (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_name VARCHAR(50) NOT NULL,
    healthy BOOLEAN NOT NULL,
    latency_ms BIGINT NOT NULL,
    error_message TEXT,
    check_timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (provider_name) REFERENCES bridge_providers(provider_name)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_cross_chain_trades_trade_id ON cross_chain_trades(trade_id);
CREATE INDEX IF NOT EXISTS idx_cross_chain_trades_status ON cross_chain_trades(attestation_status);
CREATE INDEX IF NOT EXISTS idx_cross_chain_trades_source_chain ON cross_chain_trades(source_chain);
CREATE INDEX IF NOT EXISTS idx_cross_chain_trades_provider ON cross_chain_trades(bridge_provider);
CREATE INDEX IF NOT EXISTS idx_cross_chain_trades_created ON cross_chain_trades(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_bridge_attestations_trade_id ON bridge_attestations(trade_id);
CREATE INDEX IF NOT EXISTS idx_bridge_attestations_provider ON bridge_attestations(provider);
CREATE INDEX IF NOT EXISTS idx_bridge_attestations_verified ON bridge_attestations(verified);
CREATE INDEX IF NOT EXISTS idx_bridge_attestations_created ON bridge_attestations(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_bridge_retry_history_trade_id ON bridge_retry_history(trade_id);
CREATE INDEX IF NOT EXISTS idx_bridge_retry_history_created ON bridge_retry_history(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_bridge_provider_health_provider ON bridge_provider_health(provider_name);
CREATE INDEX IF NOT EXISTS idx_bridge_provider_health_created ON bridge_provider_health(created_at DESC);
