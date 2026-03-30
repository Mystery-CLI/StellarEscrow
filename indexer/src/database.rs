use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AppError;
use crate::fraud_service::FraudReport;
use crate::integration_service::{DeliveryRecord, DeliveryStatus};
use crate::models::{
    AuditBucket, AuditLog, AuditQuery, AuditStats, DiscoveryQuery, DiscoveryResult, Event,
    EventQuery, NewAuditLog, SearchHistoryEntry, SearchSuggestion, TradeSearchQuery,
    TradeSearchResult,
};

// ---------------------------------------------------------------------------
// Row helper for integration_deliveries
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct IntegrationDeliveryRow {
    id: Uuid,
    connector_id: String,
    event_id: Uuid,
    status: String,
    status_code: Option<i32>,
    error: Option<String>,
    duration_ms: i64,
    attempted_at: DateTime<Utc>,
}

impl From<IntegrationDeliveryRow> for DeliveryRecord {
    fn from(r: IntegrationDeliveryRow) -> Self {
        DeliveryRecord {
            id: r.id,
            connector_id: r.connector_id,
            event_id: r.event_id,
            status: if r.status == "success" {
                DeliveryStatus::Success
            } else {
                DeliveryStatus::Failed
            },
            status_code: r.status_code.map(|c| c as u16),
            error: r.error,
            duration_ms: r.duration_ms as u64,
            attempted_at: r.attempted_at,
        }
    }
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
(&self, event: &Event) -> Result<(), AppError> {
    /// Record a slow query to the slow_query_log table (fire-and-forget).
    /// Only logs queries exceeding `threshold_ms`.
    pub fn log_slow_query(
        pool: PgPool,
        query_hash: String,
        query_text: String,
        duration_ms: f64,
        rows_returned: Option<i32>,
    ) {
        const THRESHOLD_MS: f64 = 100.0;
        if duration_ms < THRESHOLD_MS {
            return;
        }
        tokio::spawn(async move {
            let _ = sqlx::query(
                "INSERT INTO slow_query_log (query_hash, query_text, duration_ms, rows_returned) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(&query_hash)
            .bind(&query_text)
            .bind(duration_ms)
            .bind(rows_returned)
            .execute(&pool)
            .await;
        });
    }

    /// Fetch the top slow queries from pg_stat_statements.
    pub async fn get_slow_queries(&self, limit: i64) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT query,
                   calls,
                   round(mean_exec_time::numeric, 2)  AS mean_ms,
                   round(total_exec_time::numeric, 2) AS total_ms,
                   round(stddev_exec_time::numeric, 2) AS stddev_ms
            FROM pg_stat_statements
            ORDER BY mean_exec_time DESC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 50))
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "query":    r.get::<String, _>("query"),
                    "calls":    r.get::<i64, _>("calls"),
                    "mean_ms":  r.get::<f64, _>("mean_ms"),
                    "total_ms": r.get::<f64, _>("total_ms"),
                    "stddev_ms":r.get::<f64, _>("stddev_ms"),
                })
            })
            .collect())
    }

    /// Fetch tables with low index-scan ratio (candidates for new indexes).
    pub async fn get_index_usage(&self) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT relname AS table,
                   seq_scan, idx_scan,
                   CASE WHEN seq_scan + idx_scan = 0 THEN 0
                        ELSE round(100.0 * idx_scan / (seq_scan + idx_scan), 1)
                   END AS idx_pct
            FROM pg_stat_user_tables
            ORDER BY idx_pct ASC
            LIMIT 20
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "table":    r.get::<String, _>("table"),
                    "seq_scan": r.get::<i64, _>("seq_scan"),
                    "idx_scan": r.get::<i64, _>("idx_scan"),
                    "idx_pct":  r.get::<f64, _>("idx_pct"),
                })
            })
            .collect())
    }

    pub async fn insert_event(&self, event: &Event) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            INSERT INTO events (id, event_type, category, schema_version, contract_id, ledger, transaction_hash, timestamp, data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(event.id)
        .bind(&event.event_type)
        .bind(&event.category)
        .bind(event.schema_version)
        .bind(&event.contract_id)
        .bind(event.ledger)
        .bind(&event.transaction_hash)
        .bind(event.timestamp)
        .bind(&event.data)
        .bind(event.created_at)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            self.insert_analytics_event(&event.event_type, &event.data, event.ledger)
                .await?;
        }

        Ok(())
    }

    /// Batch-insert up to `events.len()` events in a single transaction.
    /// Conflicts on `id` are silently skipped (idempotent replay).
    pub async fn insert_events_batch(&self, events: &[Event]) -> Result<crate::models::BatchInsertResult, AppError> {
        if events.is_empty() {
            return Ok(crate::models::BatchInsertResult { inserted: 0, skipped: 0 });
        }

        let mut tx = self.pool.begin().await?;
        let mut inserted = 0usize;

        for event in events {
            let result = sqlx::query(
                r#"
                INSERT INTO events (id, event_type, category, schema_version, contract_id, ledger, transaction_hash, timestamp, data, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(event.id)
            .bind(&event.event_type)
            .bind(&event.category)
            .bind(event.schema_version)
            .bind(&event.contract_id)
            .bind(event.ledger)
            .bind(&event.transaction_hash)
            .bind(event.timestamp)
            .bind(&event.data)
            .bind(event.created_at)
            .execute(&mut *tx)
            .await?;

            if result.rows_affected() > 0 {
                inserted += 1;

                sqlx::query(
                    "INSERT INTO analytics_events (event_type, data, ledger) VALUES ($1, $2, $3)"
                )
                .bind(&event.event_type)
                .bind(&event.data)
                .bind(event.ledger)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        let skipped = events.len() - inserted;
        Ok(crate::models::BatchInsertResult { inserted, skipped })
    }

    pub async fn get_events(&self, query: &EventQuery) -> Result<Vec<Event>, AppError> {
        let mut sql = "SELECT id, event_type, category, schema_version, contract_id, ledger, transaction_hash, timestamp, data, created_at FROM events WHERE 1=1".to_string();
        let mut owned: Vec<String> = vec![];

        if let Some(event_type) = &query.event_type {
            sql.push_str(&format!(" AND event_type = ${}", owned.len() + 1));
            owned.push(event_type.clone());
        }

        if let Some(category) = &query.category {
            sql.push_str(&format!(" AND category = ${}", owned.len() + 1));
            owned.push(category.clone());
        }

        if let Some(contract_id) = &query.contract_id {
            sql.push_str(&format!(" AND contract_id = ${}", owned.len() + 1));
            owned.push(contract_id.clone());
        }

        if let Some(trade_id) = query.trade_id {
            sql.push_str(&format!(" AND data->>'trade_id' = ${}", owned.len() + 1));
            owned.push(trade_id.to_string());
        }

        if let Some(from_ledger) = query.from_ledger {
            sql.push_str(&format!(" AND ledger >= ${}", owned.len() + 1));
            owned.push(from_ledger.to_string());
        }

        if let Some(to_ledger) = query.to_ledger {
            sql.push_str(&format!(" AND ledger <= ${}", owned.len() + 1));
            owned.push(to_ledger.to_string());
        }

        // Timestamp range filters use positional params bound separately below
        let mut time_params: Vec<chrono::DateTime<chrono::Utc>> = vec![];
        if let Some(from_time) = query.from_time {
            sql.push_str(&format!(" AND timestamp >= ${}", owned.len() + time_params.len() + 1));
            time_params.push(from_time);
        }
        if let Some(to_time) = query.to_time {
            sql.push_str(&format!(" AND timestamp <= ${}", owned.len() + time_params.len() + 1));
            time_params.push(to_time);
        }

        sql.push_str(" ORDER BY ledger DESC, timestamp DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut query_builder = sqlx::query(&sql);
        for s in &owned {
            query_builder = query_builder.bind(s.as_str());
        }
        for t in &time_params {
            query_builder = query_builder.bind(*t);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        let events = rows
            .into_iter()
            .map(|row| Event {
                id: row.get("id"),
                event_type: row.get("event_type"),
                category: row.get("category"),
                schema_version: row.get("schema_version"),
                contract_id: row.get("contract_id"),
                ledger: row.get("ledger"),
                transaction_hash: row.get("transaction_hash"),
                timestamp: row.get("timestamp"),
                data: row.get("data"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(events)
    }

    pub async fn get_event_by_id(&self, id: Uuid) -> Result<Event, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, event_type, category, schema_version, contract_id, ledger, transaction_hash, timestamp, data, created_at
            FROM events WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::EventNotFound)?;

        Ok(Event {
            id: row.get("id"),
            event_type: row.get("event_type"),
            category: row.get("category"),
            schema_version: row.get("schema_version"),
            contract_id: row.get("contract_id"),
            ledger: row.get("ledger"),
            transaction_hash: row.get("transaction_hash"),
            timestamp: row.get("timestamp"),
            data: row.get("data"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn get_latest_ledger(&self, contract_id: &str) -> Result<Option<i64>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT MAX(ledger) as latest_ledger FROM events WHERE contract_id = $1
            "#,
        )
        .bind(contract_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.get("latest_ledger")))
    }

    pub async fn count_events(&self, query: &EventQuery) -> Result<i64, AppError> {
        let mut b = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM events WHERE 1=1");
        if let Some(ref event_type) = query.event_type {
            b.push(" AND event_type = ");
            b.push_bind(event_type);
        }
        if let Some(ref category) = query.category {
            b.push(" AND category = ");
            b.push_bind(category);
        }
        if let Some(ref contract_id) = query.contract_id {
            b.push(" AND contract_id = ");
            b.push_bind(contract_id);
        }
        if let Some(trade_id) = query.trade_id {
            b.push(" AND data->>'trade_id' = ");
            b.push_bind(trade_id.to_string());
        }
        if let Some(from_ledger) = query.from_ledger {
            b.push(" AND ledger >= ");
            b.push_bind(from_ledger);
        }
        if let Some(to_ledger) = query.to_ledger {
            b.push(" AND ledger <= ");
            b.push_bind(to_ledger);
        }
        if let Some(from_time) = query.from_time {
            b.push(" AND timestamp >= ");
            b.push_bind(from_time);
        }
        if let Some(to_time) = query.to_time {
            b.push(" AND timestamp <= ");
            b.push_bind(to_time);
        }
        let total: i64 = b.build_query_scalar().fetch_one(&self.pool).await?;
        Ok(total)
    }

    pub async fn get_events_in_range(
        &self,
        from_ledger: i64,
        to_ledger: i64,
        contract_id: &str,
    ) -> Result<Vec<Event>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_type, category, schema_version, contract_id, ledger, transaction_hash, timestamp, data, created_at
            FROM events
            WHERE contract_id = $1 AND ledger >= $2 AND ledger <= $3
            ORDER BY ledger ASC, timestamp ASC
            "#,
        )
        .bind(contract_id)
        .bind(from_ledger)
        .bind(to_ledger)
        .fetch_all(&self.pool)
        .await?;

        let events = rows
            .into_iter()
            .map(|row| Event {
                id: row.get("id"),
                event_type: row.get("event_type"),
                category: row.get("category"),
                schema_version: row.get("schema_version"),
                contract_id: row.get("contract_id"),
                ledger: row.get("ledger"),
                transaction_hash: row.get("transaction_hash"),
                timestamp: row.get("timestamp"),
                data: row.get("data"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(events)
    }

    /// Total number of indexed events, optionally filtered by contract.
    pub async fn get_event_count(&self, contract_id: Option<&str>) -> Result<i64, AppError> {
        let count: i64 = if let Some(cid) = contract_id {
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE contract_id = $1")
                .bind(cid)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM events")
                .fetch_one(&self.pool)
                .await?
        };
        Ok(count)
    }

    /// Event counts grouped by event_type — used for stats/dashboard.
    pub async fn get_event_type_counts(&self) -> Result<Vec<(String, i64)>, AppError> {
        let rows = sqlx::query(
            "SELECT event_type, COUNT(*) as cnt FROM events GROUP BY event_type ORDER BY cnt DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("event_type"), r.get::<i64, _>("cnt")))
            .collect())
    }

    /// Latest indexed ledger and its timestamp across all contracts.
    pub async fn get_latest_ledger_global(&self) -> Result<Option<(i64, DateTime<Utc>)>, AppError> {
        let row = sqlx::query("SELECT ledger, timestamp FROM events ORDER BY ledger DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| {
            (
                r.get::<i64, _>("ledger"),
                r.get::<DateTime<Utc>, _>("timestamp"),
            )
        }))
    }

    pub async fn record_search(&self, query_text: &str, search_type: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO search_history (query_text, search_type)
            VALUES ($1, $2)
            "#,
        )
        .bind(query_text)
        .bind(search_type)
        .execute(&self.pool)
        .await?;

        // Upsert daily analytics bucket
        sqlx::query(
            r#"
            INSERT INTO search_analytics (date, search_type, query_count, unique_terms, updated_at)
            VALUES (CURRENT_DATE, $1, 1, 1, NOW())
            ON CONFLICT (date, search_type) DO UPDATE
                SET query_count  = search_analytics.query_count + 1,
                    unique_terms = (
                        SELECT COUNT(DISTINCT query_text)
                        FROM search_history
                        WHERE search_type = $1
                          AND created_at::DATE = CURRENT_DATE
                    ),
                    updated_at = NOW()
            "#,
        )
        .bind(search_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_search_analytics(
        &self,
        query: &crate::models::SearchAnalyticsQuery,
    ) -> Result<crate::models::SearchAnalyticsResponse, AppError> {
        use crate::models::{SearchAnalyticsResponse, SearchAnalyticsRow};

        let mut conditions = vec!["1=1".to_string()];
        let mut idx = 1usize;

        if query.from.is_some() {
            conditions.push(format!("date >= ${}", idx));
            idx += 1;
        }
        if query.to.is_some() {
            conditions.push(format!("date <= ${}", idx));
            idx += 1;
        }
        if query.search_type.is_some() {
            conditions.push(format!("search_type = ${}", idx));
            idx += 1;
        }
        let _ = idx;

        let sql = format!(
            "SELECT date, search_type, query_count, unique_terms \
             FROM search_analytics WHERE {} ORDER BY date DESC, search_type",
            conditions.join(" AND ")
        );

        let mut qb = sqlx::query_as::<_, SearchAnalyticsRow>(&sql);
        if let Some(v) = query.from {
            qb = qb.bind(v);
        }
        if let Some(v) = query.to {
            qb = qb.bind(v);
        }
        if let Some(ref v) = query.search_type {
            qb = qb.bind(v);
        }

        let rows = qb.fetch_all(&self.pool).await?;
        let total_queries: i64 = rows.iter().map(|r| r.query_count).sum();

        let top_terms = self.get_search_suggestions("", 10).await?;

        Ok(SearchAnalyticsResponse {
            rows,
            top_terms,
            total_queries,
        })
    }

    pub async fn search_trades(
        &self,
        query: &TradeSearchQuery,
    ) -> Result<Vec<TradeSearchResult>, AppError> {
        let limit = query.limit.unwrap_or(25).clamp(1, 100);
        let offset = query.offset.unwrap_or(0).max(0);
        let q = query.q.clone().unwrap_or_default();

        // Use full-text search when a query term is provided, fall back to no filter
        let rows = sqlx::query_as::<_, TradeSearchResult>(
            r#"
            WITH latest_trade_events AS (
                SELECT DISTINCT ON ((data->>'trade_id'))
                    (data->>'trade_id')::BIGINT AS trade_id,
                    event_type
                FROM events
                WHERE data->>'trade_id' IS NOT NULL
                ORDER BY (data->>'trade_id'), ledger DESC, timestamp DESC
            ),
            trade_base AS (
                SELECT
                    (e.data->>'trade_id')::BIGINT AS trade_id,
                    e.data->>'seller'             AS seller,
                    e.data->>'buyer'              AS buyer,
                    (e.data->>'amount')::BIGINT   AS amount,
                    e.timestamp                   AS created_at,
                    e.search_vec
                FROM events e
                WHERE e.event_type = 'trade_created'
            )
            SELECT
                tb.trade_id,
                tb.seller,
                tb.buyer,
                tb.amount,
                lte.event_type AS status,
                tb.created_at
            FROM trade_base tb
            JOIN latest_trade_events lte ON lte.trade_id = tb.trade_id
            WHERE
                ($1 = '' OR tb.search_vec @@ plainto_tsquery('english', $1))
                AND ($2::TEXT IS NULL OR lte.event_type = $2)
                AND ($3::TEXT IS NULL OR tb.seller = $3)
                AND ($4::TEXT IS NULL OR tb.buyer = $4)
                AND ($5::BIGINT IS NULL OR tb.amount >= $5)
                AND ($6::BIGINT IS NULL OR tb.amount <= $6)
            ORDER BY
                CASE WHEN $1 = '' THEN 0 ELSE ts_rank(tb.search_vec, plainto_tsquery('english', $1)) END DESC,
                tb.created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(q.as_str())
        .bind(query.status.as_deref())
        .bind(query.seller.as_deref())
        .bind(query.buyer.as_deref())
        .bind(query.min_amount.map(|v| v as i64))
        .bind(query.max_amount.map(|v| v as i64))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn discover_entities(
        &self,
        query: &DiscoveryQuery,
    ) -> Result<Vec<DiscoveryResult>, AppError> {
        let limit = query.limit.unwrap_or(25).clamp(1, 100);
        let q = query.q.clone().unwrap_or_default();

        let rows = sqlx::query_as::<_, DiscoveryResult>(
            r#"
            WITH entities AS (
                SELECT data->>'seller' AS address, 'user' AS role, timestamp, search_vec
                FROM events
                WHERE event_type = 'trade_created' AND data->>'seller' IS NOT NULL
                UNION ALL
                SELECT data->>'buyer' AS address, 'user' AS role, timestamp, search_vec
                FROM events
                WHERE event_type = 'trade_created' AND data->>'buyer' IS NOT NULL
                UNION ALL
                SELECT data->>'arbitrator' AS address, 'arbitrator' AS role, timestamp, search_vec
                FROM events
                WHERE event_type = 'arb_reg' AND data->>'arbitrator' IS NOT NULL
            )
            SELECT
                address,
                role,
                COUNT(*)::BIGINT AS seen_count,
                MAX(timestamp) AS last_seen
            FROM entities
            WHERE
                ($1 = '' OR search_vec @@ plainto_tsquery('english', $1))
                AND ($2::TEXT IS NULL OR role = $2)
            GROUP BY address, role
            ORDER BY seen_count DESC, last_seen DESC
            LIMIT $3
            "#,
        )
        .bind(q.as_str())
        .bind(query.role.as_deref())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Full-text search over registered user profiles.
    pub async fn search_users(
        &self,
        q: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::UserProfile>, AppError> {
        let rows = sqlx::query_as::<_, crate::models::UserProfile>(
            r#"
            SELECT *
            FROM user_profiles
            WHERE $1 = '' OR search_vec @@ plainto_tsquery('english', $1)
            ORDER BY
                CASE WHEN $1 = '' THEN 0
                     ELSE ts_rank(search_vec, plainto_tsquery('english', $1))
                END DESC,
                registered_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(q)
        .bind(limit.clamp(1, 100))
        .bind(offset.max(0))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_search_suggestions(
        &self,
        prefix: &str,
        limit: i64,
    ) -> Result<Vec<SearchSuggestion>, AppError> {
        let q_pattern = format!("{}%", prefix);
        let rows = sqlx::query_as::<_, SearchSuggestion>(
            r#"
            SELECT
                query_text AS term,
                COUNT(*)::BIGINT AS hits
            FROM search_history
            WHERE query_text ILIKE $1
            GROUP BY query_text
            ORDER BY hits DESC, term ASC
            LIMIT $2
            "#,
        )
        .bind(q_pattern)
        .bind(limit.clamp(1, 20))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_search_history(
        &self,
        limit: i64,
    ) -> Result<Vec<SearchHistoryEntry>, AppError> {
        let rows = sqlx::query_as::<_, SearchHistoryEntry>(
            r#"
            SELECT id, query_text, search_type, created_at
            FROM search_history
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // =========================================================================
    // Audit Log Operations
    // =========================================================================

    /// Insert a new audit log entry.
    pub async fn insert_audit_log(&self, entry: &NewAuditLog) -> Result<AuditLog, AppError> {
        let row = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs
                (actor, category, action, resource_type, resource_id,
                 outcome, ledger, tx_hash, metadata, severity)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(&entry.actor)
        .bind(entry.category.as_str())
        .bind(&entry.action)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(entry.outcome.as_str())
        .bind(entry.ledger)
        .bind(&entry.tx_hash)
        .bind(&entry.metadata)
        .bind(entry.severity.as_str())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    /// Query audit logs with optional filters and pagination.
    pub async fn query_audit_logs(&self, q: &AuditQuery) -> Result<Vec<AuditLog>, AppError> {
        let limit = q.limit.unwrap_or(50).clamp(1, 500);
        let offset = q.offset.unwrap_or(0).max(0);

        // Build dynamic WHERE clauses
        let mut conditions = vec!["1=1".to_string()];
        let mut idx = 1usize;

        macro_rules! push_cond {
            ($field:expr, $val:expr) => {
                if $val.is_some() {
                    conditions.push(format!("{} = ${}", $field, idx));
                    idx += 1;
                }
            };
        }

        push_cond!("actor", q.actor);
        push_cond!("category", q.category);
        push_cond!("action", q.action);
        push_cond!("resource_type", q.resource_type);
        push_cond!("resource_id", q.resource_id);
        push_cond!("outcome", q.outcome);
        push_cond!("severity", q.severity);

        if q.from.is_some() {
            conditions.push(format!("created_at >= ${}", idx));
            idx += 1;
        }
        if q.to.is_some() {
            conditions.push(format!("created_at <= ${}", idx));
            idx += 1;
        }

        let sql = format!(
            "SELECT * FROM audit_logs WHERE {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            conditions.join(" AND "),
            limit,
            offset
        );

        let mut qb = sqlx::query_as::<_, AuditLog>(&sql);
        if let Some(v) = &q.actor {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.category {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.action {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.resource_type {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.resource_id {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.outcome {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.severity {
            qb = qb.bind(v);
        }
        if let Some(v) = q.from {
            qb = qb.bind(v);
        }
        if let Some(v) = q.to {
            qb = qb.bind(v);
        }

        Ok(qb.fetch_all(&self.pool).await?)
    }

    /// Count audit logs matching the same filters (for pagination).
    pub async fn count_audit_logs(&self, q: &AuditQuery) -> Result<i64, AppError> {
        let mut conditions = vec!["1=1".to_string()];
        let mut idx = 1usize;

        macro_rules! push_cond {
            ($field:expr, $val:expr) => {
                if $val.is_some() {
                    conditions.push(format!("{} = ${}", $field, idx));
                    idx += 1;
                }
            };
        }

        push_cond!("actor", q.actor);
        push_cond!("category", q.category);
        push_cond!("action", q.action);
        push_cond!("resource_type", q.resource_type);
        push_cond!("resource_id", q.resource_id);
        push_cond!("outcome", q.outcome);
        push_cond!("severity", q.severity);

        if q.from.is_some() {
            conditions.push(format!("created_at >= ${}", idx));
            idx += 1;
        }
        if q.to.is_some() {
            conditions.push(format!("created_at <= ${}", idx));
            idx += 1;
        }

        let sql = format!(
            "SELECT COUNT(*) FROM audit_logs WHERE {}",
            conditions.join(" AND ")
        );

        let mut qb = sqlx::query(&sql);
        if let Some(v) = &q.actor {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.category {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.action {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.resource_type {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.resource_id {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.outcome {
            qb = qb.bind(v);
        }
        if let Some(v) = &q.severity {
            qb = qb.bind(v);
        }
        if let Some(v) = q.from {
            qb = qb.bind(v);
        }
        if let Some(v) = q.to {
            qb = qb.bind(v);
        }

        let row = qb.fetch_one(&self.pool).await?;
        Ok(row.get::<i64, _>(0))
    }

    /// Aggregate statistics for the analysis dashboard.
    pub async fn audit_stats(&self) -> Result<AuditStats, AppError> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&self.pool)
            .await?;

        let by_category = sqlx::query_as::<_, AuditBucket>(
            "SELECT category AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY category ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let by_outcome = sqlx::query_as::<_, AuditBucket>(
            "SELECT outcome AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY outcome ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let by_severity = sqlx::query_as::<_, AuditBucket>(
            "SELECT severity AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY severity ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let top_actors = sqlx::query_as::<_, AuditBucket>(
            "SELECT actor AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY actor ORDER BY count DESC LIMIT 20",
        )
        .fetch_all(&self.pool)
        .await?;

        let top_actions = sqlx::query_as::<_, AuditBucket>(
            "SELECT action AS label, COUNT(*)::BIGINT AS count FROM audit_logs GROUP BY action ORDER BY count DESC LIMIT 20",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(AuditStats {
            total,
            by_category,
            by_outcome,
            by_severity,
            top_actors,
            top_actions,
        })
    }

    /// Delete audit logs older than `days` days. Returns the number of rows deleted.
    pub async fn purge_old_audit_logs(&self, days: i64) -> Result<u64, AppError> {
        let result = sqlx::query(
            "DELETE FROM audit_logs WHERE created_at < NOW() - ($1 || ' days')::INTERVAL",
        )
        .bind(days)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn insert_fraud_alert(&self, report: &FraudReport) -> Result<(), AppError> {
        let rules_json =
            serde_json::to_value(&report.rules_triggered).unwrap_or(serde_json::Value::Null);
        let status = if report.risk_score >= 80 {
            "pending"
        } else {
            "approved"
        };

        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO fraud_alerts (trade_id, risk_score, rules_triggered, ml_score)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(report.trade_id as i64)
        .bind(report.risk_score)
        .bind(&rules_json)
        .bind(report.ml_result.score as f64)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO fraud_reviews (trade_id, status)
            VALUES ($1, $2)
            ON CONFLICT (trade_id) DO NOTHING
            "#,
        )
        .bind(report.trade_id as i64)
        .bind(status)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_fraud_alerts(&self) -> Result<Vec<serde_json::Value>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT a.id, a.trade_id, a.risk_score, a.rules_triggered, a.ml_score, a.created_at,
                   r.status, r.reviewer, r.review_notes, r.updated_at
            FROM fraud_alerts a
            LEFT JOIN fraud_reviews r ON a.trade_id = r.trade_id
            ORDER BY a.risk_score DESC, a.created_at DESC
            LIMIT 50
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut alerts = Vec::new();
        for row in rows {
            alerts.push(serde_json::json!({
                "id": row.try_get::<uuid::Uuid, _>("id").ok(),
                "trade_id": row.get::<i64, _>("trade_id"),
                "risk_score": row.get::<i32, _>("risk_score"),
                "rules_triggered": row.get::<serde_json::Value, _>("rules_triggered"),
                "ml_score": row.try_get::<f64, _>("ml_score").ok(),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                "status": row.try_get::<String, _>("status").unwrap_or_else(|_| "pending".to_string()),
                "reviewer": row.try_get::<String, _>("reviewer").ok(),
                "review_notes": row.try_get::<String, _>("review_notes").ok(),
                "updated_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").ok(),
            }));
        }

        Ok(alerts)
    }

    pub async fn update_fraud_review(
        &self,
        trade_id: u64,
        status: &str,
        reviewer: &str,
        notes: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO fraud_reviews (trade_id, status, reviewer, review_notes)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (trade_id) DO UPDATE SET
                status = EXCLUDED.status,
                reviewer = EXCLUDED.reviewer,
                review_notes = EXCLUDED.review_notes,
                updated_at = NOW()
            "#,
        )
        .bind(trade_id as i64)
        .bind(status)
        .bind(reviewer)
        .bind(notes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // =========================================================================
    // Notification Operations
    // =========================================================================

    pub async fn get_notification_preferences(
        &self,
        address: &str,
    ) -> Result<Option<crate::models::NotificationPreferences>, AppError> {
        let row = sqlx::query_as::<_, crate::models::NotificationPreferences>(
            "SELECT * FROM notification_preferences WHERE address = $1",
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn upsert_notification_preferences(
        &self,
        address: &str,
        upd: &crate::models::UpdateNotificationPreferences,
    ) -> Result<crate::models::NotificationPreferences, AppError> {
        // Fetch existing or use defaults, then apply partial update
        let existing = self.get_notification_preferences(address).await?;
        let base = existing.unwrap_or_else(|| {
            crate::models::NotificationPreferences::default_for_address(address.to_string())
        });

        let row = sqlx::query_as::<_, crate::models::NotificationPreferences>(
            r#"
            INSERT INTO notification_preferences
                (address, email_enabled, email_address, sms_enabled, phone_number,
                 push_enabled, push_token,
                 on_trade_created, on_trade_funded, on_trade_completed, on_trade_confirmed,
                 on_dispute_raised, on_dispute_resolved, on_trade_cancelled, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,NOW())
            ON CONFLICT (address) DO UPDATE SET
                email_enabled   = EXCLUDED.email_enabled,
                email_address   = EXCLUDED.email_address,
                sms_enabled     = EXCLUDED.sms_enabled,
                phone_number    = EXCLUDED.phone_number,
                push_enabled    = EXCLUDED.push_enabled,
                push_token      = EXCLUDED.push_token,
                on_trade_created    = EXCLUDED.on_trade_created,
                on_trade_funded     = EXCLUDED.on_trade_funded,
                on_trade_completed  = EXCLUDED.on_trade_completed,
                on_trade_confirmed  = EXCLUDED.on_trade_confirmed,
                on_dispute_raised   = EXCLUDED.on_dispute_raised,
                on_dispute_resolved = EXCLUDED.on_dispute_resolved,
                on_trade_cancelled  = EXCLUDED.on_trade_cancelled,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(address)
        .bind(upd.email_enabled.unwrap_or(base.email_enabled))
        .bind(upd.email_address.as_ref().or(base.email_address.as_ref()))
        .bind(upd.sms_enabled.unwrap_or(base.sms_enabled))
        .bind(upd.phone_number.as_ref().or(base.phone_number.as_ref()))
        .bind(upd.push_enabled.unwrap_or(base.push_enabled))
        .bind(upd.push_token.as_ref().or(base.push_token.as_ref()))
        .bind(upd.on_trade_created.unwrap_or(base.on_trade_created))
        .bind(upd.on_trade_funded.unwrap_or(base.on_trade_funded))
        .bind(upd.on_trade_completed.unwrap_or(base.on_trade_completed))
        .bind(upd.on_trade_confirmed.unwrap_or(base.on_trade_confirmed))
        .bind(upd.on_dispute_raised.unwrap_or(base.on_dispute_raised))
        .bind(upd.on_dispute_resolved.unwrap_or(base.on_dispute_resolved))
        .bind(upd.on_trade_cancelled.unwrap_or(base.on_trade_cancelled))
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn unregister_push_token(&self, token: &str) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE notification_preferences
            SET push_enabled = FALSE,
                push_token = NULL,
                updated_at = NOW()
            WHERE push_token = $1
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn log_notification(
        &self,
        address: &str,
        channel: &str,
        template_id: &str,
        subject: Option<&str>,
        body: &str,
        result: Result<(), String>,
    ) {
        let (status, error) = match result {
            Ok(()) => ("sent", None),
            Err(e) => ("failed", Some(e)),
        };
        let _ = sqlx::query(
            r#"
            INSERT INTO notification_log (address, channel, template_id, subject, body, status, error)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(address)
        .bind(channel)
        .bind(template_id)
        .bind(subject)
        .bind(body)
        .bind(status)
        .bind(error)
        .execute(&self.pool)
        .await;
    }

    pub async fn get_notification_log(
        &self,
        address: &str,
        limit: i64,
    ) -> Result<Vec<crate::models::NotificationLogEntry>, AppError> {
        let rows = sqlx::query_as::<_, crate::models::NotificationLogEntry>(
            r#"
            SELECT id, address, channel, template_id, subject, body, status, error, created_at
            FROM notification_log
            WHERE address = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(address)
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // -----------------------------------------------------------------------
    // Performance monitoring
    // -----------------------------------------------------------------------

    pub async fn insert_perf_sample(
        &self,
        route: &str,
        method: &str,
        status_code: u16,
        duration_ms: u64,
        is_error: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO performance_metrics (route, method, status_code, duration_ms, is_error)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(route)
        .bind(method)
        .bind(status_code as i16)
        .bind(duration_ms as i64)
        .bind(is_error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Integration service
    // -----------------------------------------------------------------------

    pub async fn insert_integration_delivery(
        &self,
        record: &crate::integration_service::DeliveryRecord,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO integration_deliveries
                (id, connector_id, event_id, status, status_code, error, duration_ms, attempted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(record.id)
        .bind(&record.connector_id)
        .bind(record.event_id)
        .bind(format!("{:?}", record.status).to_lowercase())
        .bind(record.status_code.map(|c| c as i32))
        .bind(&record.error)
        .bind(record.duration_ms as i64)
        .bind(record.attempted_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_perf_alert(
        &self,
        rule_name: &str,
        severity: &str,
        message: &str,
        threshold: f64,
        observed: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO performance_alerts (rule_name, severity, message, threshold, observed)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(rule_name)
        .bind(severity)
        .bind(message)
        .bind(threshold)
        .bind(observed)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_integration_deliveries(
    /// Query the hourly APM rollup materialized view for the last `hours` hours.
    pub async fn get_perf_hourly_rollup(
        &self,
        hours: i64,
    ) -> Result<Vec<serde_json::Value>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT hour, route, method, requests, errors, avg_ms, p95_ms, p99_ms
            FROM perf_metrics_hourly
            WHERE hour >= NOW() - ($1 || ' hours')::INTERVAL
            ORDER BY hour DESC
            LIMIT 500
            "#,
        )
        .bind(hours)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                use sqlx::Row;
                serde_json::json!({
                    "hour": r.get::<chrono::DateTime<chrono::Utc>, _>("hour"),
                    "route": r.get::<String, _>("route"),
                    "method": r.get::<String, _>("method"),
                    "requests": r.get::<i64, _>("requests"),
                    "errors": r.get::<i64, _>("errors"),
                    "avg_ms": r.get::<f64, _>("avg_ms"),
                    "p95_ms": r.get::<f64, _>("p95_ms"),
                    "p99_ms": r.get::<f64, _>("p99_ms"),
                })
            })
            .collect())
    }
        &self,
        connector_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<crate::integration_service::DeliveryRecord>, sqlx::Error> {
        let rows = if let Some(cid) = connector_id {
            sqlx::query_as::<_, IntegrationDeliveryRow>(
                r#"
                SELECT id, connector_id, event_id, status, status_code, error, duration_ms, attempted_at
                FROM integration_deliveries
                WHERE connector_id = $1
                ORDER BY attempted_at DESC
                LIMIT $2
                "#,
            )
            .bind(cid)
            .bind(limit.clamp(1, 200))
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, IntegrationDeliveryRow>(
                r#"
                SELECT id, connector_id, event_id, status, status_code, error, duration_ms, attempted_at
                FROM integration_deliveries
                ORDER BY attempted_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit.clamp(1, 200))
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

// =============================================================================
// Analytics Operations
// =============================================================================

impl Database {
    pub async fn insert_analytics_event(
        &self,
        event_type: &str,
        data: &serde_json::Value,
        ledger: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO analytics_events (event_type, data, ledger) VALUES ($1, $2, $3)"
        )
        .bind(event_type)
        .bind(data)
        .bind(ledger)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn prune_analytics_events_older_than(&self, retention_days: u32) -> Result<u64, AppError> {
        if retention_days == 0 {
            return Ok(0);
        }

        let result = sqlx::query(
            "DELETE FROM analytics_events WHERE recorded_at < NOW() - ($1::INT * INTERVAL '1 day')"
        )
        .bind(retention_days as i32)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_analytics_events_in_range(
        &self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
        event_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<crate::analytics_service::export::AnalyticsRow>, AppError> {
        let clamped_limit = limit.clamp(1, 50_000);
        let rows = if let Some(event_type) = event_type {
            sqlx::query(
                "SELECT event_type, ledger, data, recorded_at FROM analytics_events WHERE recorded_at >= $1 AND recorded_at <= $2 AND event_type = $3 ORDER BY recorded_at DESC LIMIT $4"
            )
            .bind(from)
            .bind(to)
            .bind(event_type)
            .bind(clamped_limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT event_type, ledger, data, recorded_at FROM analytics_events WHERE recorded_at >= $1 AND recorded_at <= $2 ORDER BY recorded_at DESC LIMIT $3"
            )
            .bind(from)
            .bind(to)
            .bind(clamped_limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(|r| crate::analytics_service::export::AnalyticsRow {
            event_type: r.get("event_type"),
            ledger: r.get("ledger"),
            data: r.get("data"),
            recorded_at: r.get("recorded_at"),
        }).collect())
    }

    pub async fn get_top_analytics_event_types(&self, limit: i64) -> Result<Vec<(String, u64)>, AppError> {
        let rows = sqlx::query(
            "SELECT event_type, COUNT(*)::BIGINT AS cnt FROM analytics_events GROUP BY event_type ORDER BY cnt DESC LIMIT $1"
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let key: String = r.get("event_type");
                let count: i64 = r.get("cnt");
                (key, count.max(0) as u64)
            })
            .collect())
    }

    pub async fn get_top_analytics_categories(&self, limit: i64) -> Result<Vec<(String, u64)>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT
                split_part(event_type, '_', 1) AS category,
                COUNT(*)::BIGINT AS cnt
            FROM analytics_events
            GROUP BY category
            ORDER BY cnt DESC
            LIMIT $1
            "#
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let key: String = r.get("category");
                let count: i64 = r.get("cnt");
                (key, count.max(0) as u64)
            })
            .collect())
    }

    pub async fn get_analytics_trend_24h(
        &self,
        limit: i64,
    ) -> Result<Vec<crate::analytics_service::AnalyticsTrendPoint>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT
                date_trunc('minute', recorded_at) AS minute,
                COUNT(*)::BIGINT AS events,
                COALESCE(SUM((data->>'amount')::BIGINT), 0)::BIGINT AS volume_stroops,
                COUNT(*) FILTER (WHERE event_type = 'trade_created')::BIGINT AS trades_created,
                COUNT(*) FILTER (WHERE event_type = 'trade_confirmed')::BIGINT AS trades_completed
            FROM analytics_events
            WHERE recorded_at >= NOW() - INTERVAL '24 hours'
            GROUP BY minute
            ORDER BY minute DESC
            LIMIT $1
            "#
        )
        .bind(limit.clamp(1, 1_440))
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| crate::analytics_service::AnalyticsTrendPoint {
                minute: r.get("minute"),
                events: (r.get::<i64, _>("events")).max(0) as u64,
                volume_stroops: (r.get::<i64, _>("volume_stroops")).max(0) as u64,
                trades_created: (r.get::<i64, _>("trades_created")).max(0) as u64,
                trades_completed: (r.get::<i64, _>("trades_completed")).max(0) as u64,
            })
            .collect())
    }

    pub async fn get_realtime_analytics_window(
        &self,
        window_seconds: u64,
    ) -> Result<crate::analytics_service::aggregator::MetricWindow, AppError> {
        let seconds = (window_seconds as i64).clamp(10, 86_400);

        let total_row = sqlx::query(
            r#"
            SELECT
                COUNT(*)::BIGINT AS total_events,
                COUNT(*) FILTER (WHERE event_type = 'trade_created')::BIGINT AS trades_created,
                COUNT(*) FILTER (WHERE event_type = 'trade_funded')::BIGINT AS trades_funded,
                COUNT(*) FILTER (WHERE event_type = 'trade_confirmed')::BIGINT AS trades_completed,
                COUNT(*) FILTER (WHERE event_type = 'dispute_raised')::BIGINT AS disputes_raised,
                COALESCE(SUM((data->>'amount')::BIGINT), 0)::BIGINT AS volume_stroops
            FROM analytics_events
            WHERE recorded_at >= NOW() - ($1::BIGINT * INTERVAL '1 second')
            "#
        )
        .bind(seconds)
        .fetch_one(&self.pool)
        .await?;

        let by_type_rows = sqlx::query(
            r#"
            SELECT event_type, COUNT(*)::BIGINT AS cnt
            FROM analytics_events
            WHERE recorded_at >= NOW() - ($1::BIGINT * INTERVAL '1 second')
            GROUP BY event_type
            ORDER BY cnt DESC
            "#
        )
        .bind(seconds)
        .fetch_all(&self.pool)
        .await?;

        let mut event_counts = std::collections::HashMap::new();
        for row in by_type_rows {
            let event_type: String = row.get("event_type");
            let count: i64 = row.get("cnt");
            event_counts.insert(event_type, count.max(0) as u64);
        }

        let trades_created = (total_row.get::<i64, _>("trades_created")).max(0) as u64;
        let trades_completed = (total_row.get::<i64, _>("trades_completed")).max(0) as u64;
        let disputes_raised = (total_row.get::<i64, _>("disputes_raised")).max(0) as u64;
        let completion_rate_bps = if trades_created > 0 {
            ((trades_completed * 10_000) / trades_created) as u32
        } else {
            0
        };
        let dispute_rate_bps = if trades_created > 0 {
            ((disputes_raised * 10_000) / trades_created) as u32
        } else {
            0
        };

        Ok(crate::analytics_service::aggregator::MetricWindow {
            window_seconds: seconds as u64,
            event_counts,
            total_events: (total_row.get::<i64, _>("total_events")).max(0) as u64,
            trades_created,
            trades_funded: (total_row.get::<i64, _>("trades_funded")).max(0) as u64,
            trades_completed,
            disputes_raised,
            volume_stroops: (total_row.get::<i64, _>("volume_stroops")).max(0) as u64,
            completion_rate_bps,
            dispute_rate_bps,
            captured_at: Some(Utc::now()),
        })
    }

    pub async fn get_trade_stats(&self) -> Result<crate::analytics_service::TradeStats, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE event_type = 'trade_created') AS total_trades,
                COALESCE(SUM((data->>'amount')::BIGINT) FILTER (WHERE event_type = 'trade_created'), 0) AS total_volume,
                COUNT(*) FILTER (WHERE event_type = 'trade_confirmed') AS completed_trades,
                COUNT(*) FILTER (WHERE event_type = 'dispute_raised') AS disputed_trades,
                COUNT(*) FILTER (WHERE event_type = 'trade_cancelled') AS cancelled_trades
            FROM events
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let total: i64 = row.get("total_trades");
        let completed: i64 = row.get("completed_trades");
        let disputed: i64 = row.get("disputed_trades");
        let cancelled: i64 = row.get("cancelled_trades");
        let volume: i64 = row.get("total_volume");

        let terminal = completed + cancelled + disputed;
        let success_rate_bps = if terminal > 0 { (completed * 10_000 / terminal) as u32 } else { 0 };
        let dispute_rate_bps = if total > 0 { (disputed * 10_000 / total) as u32 } else { 0 };
        let avg = if total > 0 { volume as f64 / total as f64 } else { 0.0 };

        Ok(crate::analytics_service::TradeStats {
            total_trades: total as u64,
            total_volume: volume as u64,
            completed_trades: completed as u64,
            disputed_trades: disputed as u64,
            cancelled_trades: cancelled as u64,
            avg_trade_amount: avg,
            success_rate_bps,
            dispute_rate_bps,
        })
    }

    pub async fn get_user_behavior(&self) -> Result<crate::analytics_service::UserBehavior, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(DISTINCT data->>'seller') AS unique_sellers,
                COUNT(DISTINCT data->>'buyer') AS unique_buyers
            FROM events WHERE event_type = 'trade_created'
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let sellers: i64 = row.get("unique_sellers");
        let buyers: i64 = row.get("unique_buyers");
        let total_users = (sellers + buyers).max(1);
        let total_trades: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE event_type = 'trade_created'"
        ).fetch_one(&self.pool).await?;
        let repeat_traders: i64 = sqlx::query_scalar(
            r#"
            WITH participants AS (
                SELECT data->>'seller' AS address
                FROM events
                WHERE event_type = 'trade_created' AND data->>'seller' IS NOT NULL
                UNION ALL
                SELECT data->>'buyer' AS address
                FROM events
                WHERE event_type = 'trade_created' AND data->>'buyer' IS NOT NULL
            )
            SELECT COUNT(*)::BIGINT
            FROM (
                SELECT address
                FROM participants
                GROUP BY address
                HAVING COUNT(*) > 1
            ) repeated
            "#
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(crate::analytics_service::UserBehavior {
            unique_sellers: sellers as u64,
            unique_buyers: buyers as u64,
            repeat_traders: repeat_traders.max(0) as u64,
            avg_trades_per_user: total_trades as f64 / total_users as f64,
        })
    }

    pub async fn get_platform_metrics(&self) -> Result<crate::analytics_service::PlatformMetrics, AppError> {
        let created: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::BIGINT FROM events WHERE event_type = 'trade_created'"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        let terminal: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::BIGINT FROM events WHERE event_type IN ('trade_confirmed', 'trade_cancelled', 'dispute_raised')"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let events_last_5m: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::BIGINT FROM analytics_events WHERE recorded_at >= NOW() - INTERVAL '5 minutes'"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let total_fees_collected: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM((data->>'fee')::BIGINT), 0)::BIGINT FROM events WHERE event_type = 'trade_confirmed'"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(crate::analytics_service::PlatformMetrics {
            events_per_minute: events_last_5m as f64 / 5.0,
            active_trades: created.saturating_sub(terminal) as u64,
            total_fees_collected: total_fees_collected.max(0) as u64,
            websocket_connections: 0,
            api_requests_per_minute: 0.0,
        })
    }
}

// =============================================================================
// Backup Operations
// =============================================================================

impl Database {
    pub async fn insert_backup_record(
        &self,
        record: &crate::backup_service::BackupRecord,
    ) -> Result<(), AppError> {
        let status = format!("{:?}", record.status).to_lowercase();
        sqlx::query(
            "INSERT INTO backup_records (id, started_at, status, verified) VALUES ($1, $2, $3, $4)"
        )
        .bind(record.id)
        .bind(record.started_at)
        .bind(&status)
        .bind(record.verified)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_backup_record(
        &self,
        record: &crate::backup_service::BackupRecord,
    ) -> Result<(), AppError> {
        let status = format!("{:?}", record.status).to_lowercase();
        sqlx::query(
            r#"UPDATE backup_records SET
                completed_at = $1, status = $2, size_bytes = $3,
                location = $4, checksum = $5, error = $6, verified = $7
               WHERE id = $8"#
        )
        .bind(record.completed_at)
        .bind(&status)
        .bind(record.size_bytes.map(|s| s as i64))
        .bind(&record.location)
        .bind(&record.checksum)
        .bind(&record.error)
        .bind(record.verified)
        .bind(record.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// =============================================================================
// Webhook Operations
// =============================================================================

impl Database {
    pub async fn get_webhook_endpoints(&self) -> Result<Vec<crate::webhook_service::WebhookEndpoint>, AppError> {
        let rows = sqlx::query(
            "SELECT id, url, secret, event_types, active, created_at, failure_count FROM webhook_endpoints ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| {
            let event_types_json: serde_json::Value = r.get("event_types");
            let event_types: Vec<String> = serde_json::from_value(event_types_json).unwrap_or_default();
            crate::webhook_service::WebhookEndpoint {
                id: r.get("id"),
                url: r.get("url"),
                secret: r.get("secret"),
                event_types,
                active: r.get("active"),
                created_at: r.get("created_at"),
                failure_count: r.get::<i32, _>("failure_count") as u32,
            }
        }).collect())
    }

    pub async fn insert_webhook_endpoint(
        &self,
        ep: &crate::webhook_service::WebhookEndpoint,
    ) -> Result<(), anyhow::Error> {
        let event_types = serde_json::to_value(&ep.event_types)?;
        sqlx::query(
            "INSERT INTO webhook_endpoints (id, url, secret, event_types, active, created_at, failure_count) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(ep.id)
        .bind(&ep.url)
        .bind(&ep.secret)
        .bind(&event_types)
        .bind(ep.active)
        .bind(ep.created_at)
        .bind(ep.failure_count as i32)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn deactivate_webhook_endpoint(&self, id: Uuid) -> Result<(), anyhow::Error> {
        sqlx::query("UPDATE webhook_endpoints SET active = false WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_webhook_delivery(
        &self,
        record: &crate::webhook_service::WebhookDeliveryRecord,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO webhook_deliveries (id, endpoint_id, event_type, payload, status_code, success, attempt, error, delivered_at, duration_ms) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
        )
        .bind(record.id)
        .bind(record.endpoint_id)
        .bind(&record.event_type)
        .bind(&record.payload)
        .bind(record.status_code.map(|c| c as i32))
        .bind(record.success)
        .bind(record.attempt as i32)
        .bind(&record.error)
        .bind(record.delivered_at)
        .bind(record.duration_ms as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

// Compliance Operations
// =============================================================================

impl Database {
    pub async fn insert_compliance_check(
        &self,
        check: &crate::compliance_service::ComplianceCheck,
    ) -> Result<(), AppError> {
        let kyc_json = serde_json::to_value(&check.kyc_result).unwrap_or_default();
        let aml_json = serde_json::to_value(&check.aml_result).unwrap_or_default();
        let status = format!("{:?}", check.status).to_lowercase();

        sqlx::query(
            r#"
            INSERT INTO compliance_checks
                (id, address, trade_id, kyc_result, aml_result, status, risk_score, notes, checked_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(check.id)
        .bind(&check.address)
        .bind(check.trade_id.map(|id| id as i64))
        .bind(&kyc_json)
        .bind(&aml_json)
        .bind(&status)
        .bind(check.risk_score as i32)
        .bind(&check.notes)
        .bind(check.checked_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_latest_compliance_check(
        &self,
        address: &str,
    ) -> Result<Option<crate::compliance_service::ComplianceCheck>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT id, address, trade_id, kyc_result, aml_result, status, risk_score,
                   notes, checked_at, reviewed_by, reviewed_at
            FROM compliance_checks
            WHERE address = $1
            ORDER BY checked_at DESC
            LIMIT 1
            "#,
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| self.row_to_compliance_check(&r)))
    }

    pub async fn get_compliance_checks_in_range(
        &self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<crate::compliance_service::ComplianceCheck>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, address, trade_id, kyc_result, aml_result, status, risk_score,
                   notes, checked_at, reviewed_by, reviewed_at
            FROM compliance_checks
            WHERE checked_at >= $1 AND checked_at <= $2
            ORDER BY checked_at DESC
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| self.row_to_compliance_check(r)).collect())
    }

    pub async fn update_compliance_review(
        &self,
        check_id: uuid::Uuid,
        status: &crate::compliance_service::ComplianceStatus,
        reviewer: &str,
        notes: &str,
    ) -> Result<(), anyhow::Error> {
        let status_str = format!("{:?}", status).to_lowercase();
        sqlx::query(
            r#"
            UPDATE compliance_checks
            SET status = $1, reviewed_by = $2, notes = $3, reviewed_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(&status_str)
        .bind(reviewer)
        .bind(notes)
        .bind(check_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // =========================================================================
    // Dispute Evidence (Issue #125)
    // =========================================================================

    /// Verify that `address` is a buyer, seller, or arbitrator for the given dispute (trade_id).
    /// Returns Forbidden if not a participant.
    pub async fn check_dispute_participant(&self, dispute_id: i64, address: &str) -> Result<(), crate::error::AppError> {
        use sqlx::Row;
        // Participants are derived from trade events stored in the events table.
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) AS cnt
            FROM events
            WHERE event_type IN ('trade_created', 'dispute_raised', 'arbitrator_registered')
              AND (
                data->>'trade_id' = $1::text
                AND (
                    data->>'seller'     = $2
                    OR data->>'buyer'   = $2
                    OR data->>'raised_by' = $2
                    OR data->>'arbitrator' = $2
                )
              )
            "#,
        )
        .bind(dispute_id)
        .bind(address)
        .fetch_one(&self.pool)
        .await?;

        let cnt: i64 = row.get("cnt");
        if cnt == 0 {
            return Err(crate::error::AppError::Forbidden(
                "Only dispute participants or arbitrators may access evidence".into(),
            ));
        }
        Ok(())
    }

    /// Insert a dispute evidence record linking a file to a dispute.
    pub async fn insert_dispute_evidence(
        &self,
        dispute_id: i64,
        file_id: uuid::Uuid,
        uploader: &str,
        description: Option<&str>,
    ) -> Result<crate::models::DisputeEvidenceRecord, crate::error::AppError> {
        let record = sqlx::query_as::<_, crate::models::DisputeEvidenceRecord>(
            r#"
            INSERT INTO dispute_evidence (dispute_id, file_id, uploader, description)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(dispute_id)
        .bind(file_id)
        .bind(uploader)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;
        Ok(record)
    }

    /// List all evidence records for a dispute.
    pub async fn list_dispute_evidence(
        &self,
        dispute_id: i64,
    ) -> Result<Vec<crate::models::DisputeEvidenceRecord>, crate::error::AppError> {
        let records = sqlx::query_as::<_, crate::models::DisputeEvidenceRecord>(
            "SELECT * FROM dispute_evidence WHERE dispute_id = $1 ORDER BY created_at ASC",
        )
        .bind(dispute_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }

    /// Fetch a single evidence record by its id.
    pub async fn get_dispute_evidence(
        &self,
        evidence_id: uuid::Uuid,
    ) -> Result<Option<crate::models::DisputeEvidenceRecord>, crate::error::AppError> {
        let record = sqlx::query_as::<_, crate::models::DisputeEvidenceRecord>(
            "SELECT * FROM dispute_evidence WHERE id = $1",
        )
        .bind(evidence_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(record)
    }

    /// Create a short-lived (15-minute) signed download token for an evidence file.
    pub async fn create_evidence_download_token(
        &self,
        file_id: uuid::Uuid,
        requester: &str,
    ) -> Result<crate::models::EvidenceDownloadToken, crate::error::AppError> {
        use chrono::Duration;
        let expires_at = chrono::Utc::now() + Duration::minutes(15);
        let record = sqlx::query_as::<_, crate::models::EvidenceDownloadToken>(
            r#"
            INSERT INTO evidence_download_tokens (file_id, requester, expires_at)
            VALUES ($1, $2, $3)
            RETURNING token, file_id, requester, expires_at, used, created_at
            "#,
        )
        .bind(file_id)
        .bind(requester)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(record)
    }

    /// Consume (mark used) a download token if it is valid and unexpired.
    /// Returns None if the token is invalid, expired, or already used.
    pub async fn consume_evidence_download_token(
        &self,
        token: uuid::Uuid,
    ) -> Result<Option<crate::models::EvidenceDownloadToken>, crate::error::AppError> {
        let record = sqlx::query_as::<_, crate::models::EvidenceDownloadToken>(
            r#"
            UPDATE evidence_download_tokens
            SET used = TRUE
            WHERE token = $1
              AND used = FALSE
              AND expires_at > NOW()
            RETURNING token, file_id, requester, expires_at, used, created_at
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;
        Ok(record)
    }

    fn row_to_compliance_check(
        &self,
        row: &sqlx::postgres::PgRow,
    ) -> crate::compliance_service::ComplianceCheck {
        use sqlx::Row;
        let kyc_json: serde_json::Value = row.get("kyc_result");
        let aml_json: serde_json::Value = row.get("aml_result");
        let status_str: String = row.get("status");

        let status = match status_str.as_str() {
            "approved" => crate::compliance_service::ComplianceStatus::Approved,
            "rejected" => crate::compliance_service::ComplianceStatus::Rejected,
            "blocked" => crate::compliance_service::ComplianceStatus::Blocked,
            "requires_review" => crate::compliance_service::ComplianceStatus::RequiresReview,
            _ => crate::compliance_service::ComplianceStatus::Pending,
        };

        crate::compliance_service::ComplianceCheck {
            id: row.get("id"),
            address: row.get("address"),
            trade_id: row.get::<Option<i64>, _>("trade_id").map(|v| v as u64),
            kyc_result: serde_json::from_value(kyc_json).unwrap_or_else(|_| {
                crate::compliance_service::kyc::KycResult {
                    status: crate::compliance_service::kyc::KycStatus::Unverified,
                    level: 0,
                    provider: "unknown".to_string(),
                    reference_id: None,
                    jurisdiction: None,
                    failure_reason: None,
                }
            }),
            aml_result: serde_json::from_value(aml_json).unwrap_or_else(|_| {
                crate::compliance_service::aml::AmlResult {
                    risk_score: 0,
                    is_blocked: false,
                    sanctions_matches: vec![],
                    exposure_categories: vec![],
                    provider: "unknown".to_string(),
                    reference_id: None,
                }
            }),
            status,
            risk_score: row.get::<i32, _>("risk_score") as u8,
            notes: row.get("notes"),
            checked_at: row.get("checked_at"),
            reviewed_by: row.get("reviewed_by"),
            reviewed_at: row.get("reviewed_at"),
        }
    }
}
