CREATE TABLE IF NOT EXISTS integration_deliveries (
    id           UUID PRIMARY KEY,
    connector_id TEXT        NOT NULL,
    event_id     UUID        NOT NULL,
    status       TEXT        NOT NULL CHECK (status IN ('success', 'failed')),
    status_code  INTEGER,
    error        TEXT,
    duration_ms  BIGINT      NOT NULL DEFAULT 0,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_integration_deliveries_connector
    ON integration_deliveries (connector_id, attempted_at DESC);

CREATE INDEX IF NOT EXISTS idx_integration_deliveries_event
    ON integration_deliveries (event_id);
