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
pub struct MetricBreakdown {
    pub key: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTrendPoint {
    pub minute: DateTime<Utc>,
    pub events: u64,
    pub volume_stroops: u64,
    pub trades_created: u64,
    pub trades_completed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDashboard {
    pub generated_at: DateTime<Utc>,
    pub trade_stats: TradeStats,
    pub user_behavior: UserBehavior,
    pub platform_metrics: PlatformMetrics,
    pub realtime: MetricWindow,
    pub top_event_types: Vec<MetricBreakdown>,
    pub top_categories: Vec<MetricBreakdown>,
    pub trend_24h: Vec<AnalyticsTrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportOptions {
    pub event_type: Option<String>,
    pub limit: Option<u64>,
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
    }

    /// Get the full analytics dashboard snapshot.
    pub async fn get_dashboard(&self) -> anyhow::Result<AnalyticsDashboard> {
        let limit = self.config.export_limit.min(24 * 60) as i64;
        let (trade_stats, user_behavior, platform_metrics, top_event_types, top_categories, trend_24h, realtime) = tokio::try_join!(
            self.db.get_trade_stats(),
            self.db.get_user_behavior(),
            self.db.get_platform_metrics(),
            self.db.get_top_analytics_event_types(10),
            self.db.get_top_analytics_categories(10),
            self.db.get_analytics_trend_24h(limit),
            self.db.get_realtime_analytics_window(300),
        )?;

        Ok(AnalyticsDashboard {
            generated_at: Utc::now(),
            trade_stats,
            user_behavior,
            platform_metrics,
            realtime,
            top_event_types: top_event_types
                .into_iter()
                .map(|(key, count)| MetricBreakdown {
                    key,
                    count,
                })
                .collect(),
            top_categories: top_categories
                .into_iter()
                .map(|(key, count)| MetricBreakdown {
                    key,
                    count,
                })
                .collect(),
            trend_24h,
        })
    }

    /// Get real-time stats from the analytics event stream.
    pub async fn get_realtime(&self) -> MetricWindow {
        match self.db.get_realtime_analytics_window(300).await {
            Ok(window) => window,
            Err(e) => {
                tracing::warn!("realtime analytics query failed, using in-memory fallback: {}", e);
                self.aggregator.read().await.window()
            }
        }
    }

    /// Export analytics data as CSV or JSON for a date range.
    pub async fn export(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        format: export::ExportFormat,
        options: ExportOptions,
    ) -> anyhow::Result<String> {
        let requested_limit = options.limit.unwrap_or(self.config.export_limit);
        let limit = requested_limit.min(self.config.export_limit).min(50_000) as i64;
        let events = self
            .db
            .get_analytics_events_in_range(from, to, options.event_type.as_deref(), limit)
            .await?;
        Ok(export::render(events, format))
    }
}
