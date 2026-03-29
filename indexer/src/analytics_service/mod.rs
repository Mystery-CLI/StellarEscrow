pub mod aggregator;
pub mod export;

use crate::config::AnalyticsConfig;
use crate::database::Database;
use crate::models::Event;
use aggregator::{Aggregator, MetricWindow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStats {
    pub total_trades: u64,
    pub total_volume: u64,
    pub completed_trades: u64,
    pub disputed_trades: u64,
    pub cancelled_trades: u64,
    pub avg_trade_amount: f64,
    pub success_rate_bps: u32,
    pub dispute_rate_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBehavior {
    pub unique_sellers: u64,
    pub unique_buyers: u64,
    pub repeat_traders: u64,
    pub avg_trades_per_user: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMetrics {
    pub events_per_minute: f64,
    pub active_trades: u64,
    pub total_fees_collected: u64,
    pub websocket_connections: u64,
    pub api_requests_per_minute: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDashboard {
    pub generated_at: DateTime<Utc>,
    pub trade_stats: TradeStats,
    pub user_behavior: UserBehavior,
    pub platform_metrics: PlatformMetrics,
    pub realtime: MetricWindow,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct AnalyticsService {
    db: Arc<Database>,
    config: AnalyticsConfig,
    aggregator: Arc<RwLock<Aggregator>>,
}

impl AnalyticsService {
    pub fn new(db: Arc<Database>, config: AnalyticsConfig) -> Self {
        Self {
            db,
            config,
            aggregator: Arc::new(RwLock::new(Aggregator::new())),
        }
    }

    /// Track an event — called by EventMonitor for every contract event.
    pub async fn track_event(&self, event: &Event) {
        self.aggregator.write().await.ingest(event);

        // Persist to analytics_events table (fire-and-forget)
        let db = self.db.clone();
        let event_type = event.event_type.clone();
        let data = event.data.clone();
        let ledger = event.ledger;
        tokio::spawn(async move {
            if let Err(e) = db.insert_analytics_event(&event_type, &data, ledger).await {
                tracing::warn!("analytics persist error: {}", e);
            }
        });
    }

    /// Get the full analytics dashboard snapshot.
    pub async fn get_dashboard(&self) -> anyhow::Result<AnalyticsDashboard> {
        let (trade_stats, user_behavior, platform_metrics) = tokio::try_join!(
            self.db.get_trade_stats(),
            self.db.get_user_behavior(),
            self.db.get_platform_metrics(),
        )?;

        let realtime = self.aggregator.read().await.window();

        Ok(AnalyticsDashboard {
            generated_at: Utc::now(),
            trade_stats,
            user_behavior,
            platform_metrics,
            realtime,
        })
    }

    /// Get real-time stats from the in-memory aggregator (no DB hit).
    pub async fn get_realtime(&self) -> MetricWindow {
        self.aggregator.read().await.window()
    }

    /// Export analytics data as CSV or JSON for a date range.
    pub async fn export(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        format: export::ExportFormat,
    ) -> anyhow::Result<String> {
        let events = self.db.get_analytics_events_in_range(from, to).await?;
        Ok(export::render(events, format))
    }
}
