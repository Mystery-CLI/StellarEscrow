-- Analytics events table
CREATE TABLE IF NOT EXISTS analytics_events (
    id          BIGSERIAL PRIMARY KEY,
    event_type  TEXT NOT NULL,
    ledger      BIGINT NOT NULL,
    data        JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_analytics_event_type ON analytics_events (event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_recorded_at ON analytics_events (recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_analytics_ledger ON analytics_events (ledger DESC);

-- Backup records table
CREATE TABLE IF NOT EXISTS backup_records (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    started_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    status       TEXT NOT NULL DEFAULT 'pending',
    size_bytes   BIGINT,
    location     TEXT,
    checksum     TEXT,
    error        TEXT,
    verified     BOOLEAN NOT NULL DEFAULT false
);
CREATE INDEX IF NOT EXISTS idx_backup_started_at ON backup_records (started_at DESC);
CREATE INDEX IF NOT EXISTS idx_backup_status ON backup_records (status);

-- Webhook endpoints table
CREATE TABLE IF NOT EXISTS webhook_endpoints (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url           TEXT NOT NULL,
    secret        TEXT NOT NULL,
    event_types   JSONB NOT NULL DEFAULT '[]',
    active        BOOLEAN NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    failure_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_webhook_active ON webhook_endpoints (active);

-- Webhook deliveries table
CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    endpoint_id UUID NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
    event_type  TEXT NOT NULL,
    payload     JSONB NOT NULL DEFAULT '{}',
    status_code INTEGER,
    success     BOOLEAN NOT NULL DEFAULT false,
    attempt     INTEGER NOT NULL DEFAULT 1,
    error       TEXT,
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_ms BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_webhook_delivery_endpoint ON webhook_deliveries (endpoint_id);
CREATE INDEX IF NOT EXISTS idx_webhook_delivery_success ON webhook_deliveries (success);
CREATE INDEX IF NOT EXISTS idx_webhook_delivery_delivered_at ON webhook_deliveries (delivered_at DESC);
