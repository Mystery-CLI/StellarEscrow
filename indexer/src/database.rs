use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{Event, EventQuery};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_event(&self, event: &Event) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO events (id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(event.id)
        .bind(&event.event_type)
        .bind(&event.contract_id)
        .bind(event.ledger)
        .bind(&event.transaction_hash)
        .bind(event.timestamp)
        .bind(&event.data)
        .bind(event.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_events(&self, query: &EventQuery) -> Result<Vec<Event>, AppError> {
        let mut sql = "SELECT id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at FROM events WHERE 1=1".to_string();
        let mut bindings = vec![];

        if let Some(event_type) = &query.event_type {
            sql.push_str(&format!(" AND event_type = ${}", bindings.len() + 1));
            bindings.push(event_type.as_str());
        }

        if let Some(trade_id) = query.trade_id {
            sql.push_str(&format!(" AND data->>'trade_id' = ${}", bindings.len() + 1));
            bindings.push(&trade_id.to_string());
        }

        if let Some(from_ledger) = query.from_ledger {
            sql.push_str(&format!(" AND ledger >= ${}", bindings.len() + 1));
            bindings.push(&from_ledger.to_string());
        }

        if let Some(to_ledger) = query.to_ledger {
            sql.push_str(&format!(" AND ledger <= ${}", bindings.len() + 1));
            bindings.push(&to_ledger.to_string());
        }

        sql.push_str(" ORDER BY ledger DESC, timestamp DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut query_builder = sqlx::query(&sql);

        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        let events = rows
            .into_iter()
            .map(|row| Event {
                id: row.get("id"),
                event_type: row.get("event_type"),
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
            SELECT id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at
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

    pub async fn get_events_in_range(&self, from_ledger: i64, to_ledger: i64, contract_id: &str) -> Result<Vec<Event>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_type, contract_id, ledger, transaction_hash, timestamp, data, created_at
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
        let rows = sqlx::query("SELECT event_type, COUNT(*) as cnt FROM events GROUP BY event_type ORDER BY cnt DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.get::<String, _>("event_type"), r.get::<i64, _>("cnt")))
            .collect())
    }

    /// Latest indexed ledger and its timestamp across all contracts.
    pub async fn get_latest_ledger_global(&self) -> Result<Option<(i64, DateTime<Utc>)>, AppError> {
        let row = sqlx::query(
            "SELECT ledger, timestamp FROM events ORDER BY ledger DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| (r.get::<i64, _>("ledger"), r.get::<DateTime<Utc>, _>("timestamp"))))
    }
}
