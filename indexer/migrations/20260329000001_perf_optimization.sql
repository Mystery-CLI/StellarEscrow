-- Enable pg_stat_statements for slow query analysis
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Slow query log table: populated by the application when a query exceeds the threshold
CREATE TABLE IF NOT EXISTS slow_query_log (
    id          BIGSERIAL PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    query_hash  TEXT        NOT NULL,
    query_text  TEXT        NOT NULL,
    duration_ms DOUBLE PRECISION NOT NULL,
    rows_returned INT
);

CREATE INDEX IF NOT EXISTS idx_slow_query_recorded ON slow_query_log (recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_slow_query_duration ON slow_query_log (duration_ms DESC);

-- Partial index on events.data for trade_id lookups (already exists, kept for reference)
-- Composite covering index for the most common paginated event query
CREATE INDEX IF NOT EXISTS idx_events_type_ledger_id
    ON events (event_type, ledger DESC, id)
    WHERE event_type IS NOT NULL;

-- Covering index for search_trades hot path
CREATE INDEX IF NOT EXISTS idx_events_trade_search
    ON events (ledger DESC, timestamp DESC, event_type)
    INCLUDE (id, data)
    WHERE category = 'trade';
