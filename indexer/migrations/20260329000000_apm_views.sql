-- APM query helpers: materialized view + index for fast dashboard queries

-- Hourly rollup view for performance metrics
CREATE MATERIALIZED VIEW IF NOT EXISTS perf_metrics_hourly AS
SELECT
    date_trunc('hour', recorded_at)          AS hour,
    route,
    method,
    COUNT(*)                                  AS requests,
    SUM(CASE WHEN is_error THEN 1 ELSE 0 END) AS errors,
    AVG(duration_ms)                          AS avg_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY duration_ms) AS p95_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY duration_ms) AS p99_ms
FROM performance_metrics
GROUP BY 1, 2, 3
WITH NO DATA;

CREATE UNIQUE INDEX IF NOT EXISTS idx_perf_hourly_pk
    ON perf_metrics_hourly (hour DESC, route, method);

-- Refresh function (called by background job or cron)
CREATE OR REPLACE FUNCTION refresh_perf_hourly()
RETURNS void LANGUAGE sql AS $$
    REFRESH MATERIALIZED VIEW CONCURRENTLY perf_metrics_hourly;
$$;

-- Index to speed up alert history queries
CREATE INDEX IF NOT EXISTS idx_perf_alerts_unresolved
    ON performance_alerts (triggered_at DESC)
    WHERE resolved_at IS NULL;
