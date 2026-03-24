-- Search history for global/discovery/trade searches
CREATE TABLE IF NOT EXISTS search_history (
    id BIGSERIAL PRIMARY KEY,
    query_text TEXT NOT NULL,
    search_type VARCHAR(32) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_search_history_query_text
    ON search_history (query_text);

CREATE INDEX IF NOT EXISTS idx_search_history_created_at
    ON search_history (created_at DESC);
