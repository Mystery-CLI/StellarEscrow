CREATE TABLE IF NOT EXISTS performance_metrics (
    id              BIGSERIAL PRIMARY KEY,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    route           TEXT NOT NULL,
    method          TEXT NOT NULL,
    status_code     SMALLINT NOT NULL,
    duration_ms     BIGINT NOT NULL,
    is_error        BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_perf_metrics_recorded_at ON performance_metrics (recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_perf_metrics_route ON performance_metrics (route, recorded_at DESC);

CREATE TABLE IF NOT EXISTS performance_alerts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_name       TEXT NOT NULL,
    severity        TEXT NOT NULL,
    message         TEXT NOT NULL,
    threshold       DOUBLE PRECISION NOT NULL,
    observed        DOUBLE PRECISION NOT NULL,
    triggered_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_perf_alerts_triggered ON performance_alerts (triggered_at DESC);
