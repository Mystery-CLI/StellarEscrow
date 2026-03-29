-- Search analytics: aggregated daily stats per search type
CREATE TABLE IF NOT EXISTS search_analytics (
    id          BIGSERIAL PRIMARY KEY,
    date        DATE        NOT NULL,
    search_type VARCHAR(32) NOT NULL,
    query_count BIGINT      NOT NULL DEFAULT 0,
    unique_terms BIGINT     NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (date, search_type)
);

CREATE INDEX IF NOT EXISTS idx_search_analytics_date
    ON search_analytics (date DESC);
