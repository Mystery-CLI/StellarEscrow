-- Health metrics time-series table
CREATE TABLE IF NOT EXISTS health_metrics (
    id                      BIGSERIAL PRIMARY KEY,
    timestamp               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    requests_total          BIGINT NOT NULL DEFAULT 0,
    requests_per_minute     DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_response_ms         DOUBLE PRECISION NOT NULL DEFAULT 0,
    p95_response_ms         DOUBLE PRECISION NOT NULL DEFAULT 0,
    error_rate              DOUBLE PRECISION NOT NULL DEFAULT 0,
    active_ws_connections   BIGINT NOT NULL DEFAULT 0,
    events_indexed_total    BIGINT NOT NULL DEFAULT 0,
    last_ledger_processed   BIGINT,
    uptime_seconds          BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_health_metrics_timestamp ON health_metrics (timestamp DESC);

-- Automatically prune rows older than 7 days (run via pg_cron or a periodic DELETE)
-- This comment documents intent; actual pruning is handled by the application.
