use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{FromRef, State},
    middleware,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod analytics_service;
mod backup_service;
mod cache_service;
mod compliance_service;
mod monitoring_service;
mod webhook_service;
mod job_queue;
mod auth;
mod cache;
mod config;
mod database;
mod error;
mod event_monitor;
mod file_handlers;
mod fraud_service;
mod gateway;
mod handlers;
mod health;
mod help;
mod integration_service;
mod models;
mod notification_service;
mod rate_limit;
mod rate_limit_handlers;
mod storage;
mod websocket;
mod performance_service;

#[cfg(test)]
mod test;

use analytics_service::AnalyticsService;
use auth::auth_middleware;
use backup_service::BackupService;
use cache_service::CacheService;
use compliance_service::ComplianceService;
use config::Config;
use database::Database;
use event_monitor::EventMonitor;
use file_handlers::{delete_file, download_file, list_files, upload_file};
use fraud_service::FraudDetectionService;
use gateway::{GatewayConfig, GatewayState};
use handlers::{AppState, *};
use health::{liveness, HealthMonitor, HealthState};
use help::{
    get_contact, get_docs, get_faqs, get_tutorial_by_id, get_tutorials, help_index, search_help,
};
use integration_service::IntegrationService;
use job_queue::JobQueue;
use monitoring_service::MonitoringService;
use notification_service::NotificationService;
use performance_service::PerformanceService;
use rate_limit::RateLimiter;
use rate_limit_handlers::*;
use storage::StorageService;
use tokio::sync::Mutex;
use websocket::WebSocketManager;
use webhook_service::WebhookService;

#[derive(Parser)]
#[command(name = "stellar-escrow-indexer")]
#[command(about = "Event indexer service for Stellar Escrow contract")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse command line arguments
    let args = Args::parse();

    // Load configuration (TOML file + STELLAR_ESCROW__* env var overrides)
    let config = Config::load(&args.config)?;
    info!(
        "Loaded configuration from {} | env={} version={} schema_v={}",
        args.config, config.meta.environment, config.meta.version, config.meta.schema_version,
    );

    // Initialize database with tuned connection pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.database.url)
        .await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    let database = Arc::new(Database::new(db_pool.clone()));

    // Initialize health monitor
    let health_monitor = Arc::new(HealthMonitor::new(
        db_pool.clone(),
        config.stellar.horizon_url.clone(),
    ));
    let health_state = HealthState {
        monitor: health_monitor.clone(),
    };

    // Start metrics persistence loop in background
    let metrics_monitor = health_monitor.clone();
    tokio::spawn(async move {
        metrics_monitor.run_metrics_loop().await;
    });

    // Initialize WebSocket manager
    let (tx, _rx) = broadcast::channel(100);
    let ws_manager = Arc::new(WebSocketManager::new(tx.clone()));

    // Initialize rate limiter
    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit.clone()));

    // Initialize API auth config
    let auth_config = Arc::new(config.auth.clone());

    // Initialize API gateway configuration
    // Gateway provides centralized routing, load balancing, and authentication
    let gateway_config = GatewayConfig {
        load_balancing_enabled: !config.gateway.service_instances.is_empty(),
        service_instances: config.gateway.service_instances.clone(),
        request_logging: true,
        transform_responses: true,
    };
    let gateway_state = Arc::new(GatewayState::new(gateway_config, auth_config.clone()));

    // Initialize file storage service
    let storage_service = Arc::new(StorageService::new(db_pool, &config.storage.base_dir).await?);
    // Initialize Fraud Detection Service
    let fraud_service = Arc::new(FraudDetectionService::new(database.clone()).await);

    // Initialize Notification Service
    let notification_service = Arc::new(NotificationService::new(
        database.clone(),
        config.notification.clone(),
    ));

    // Initialize Performance Monitoring Service
    let performance_service = Arc::new(PerformanceService::new(database.clone()));
    // Initialize Integration Service
    let integration_service = Arc::new(IntegrationService::new(
        database.clone(),
        config.integration.clone(),
    ));

    // Initialize Compliance Service
    let compliance_service = Arc::new(ComplianceService::new(
        database.clone(),
        config.compliance.clone(),
    ));

    // Initialize Monitoring Service
    let monitoring_service = Arc::new(MonitoringService::new(
        database.clone(),
        config.monitoring.clone(),
    ));
    let monitoring_loop = monitoring_service.clone();
    tokio::spawn(async move { monitoring_loop.run_alert_loop().await; });

    // Initialize Analytics Service
    let analytics_service = Arc::new(AnalyticsService::new(
        database.clone(),
        config.analytics.clone(),
    ));

    // Initialize Cache Service
    let cache_service = Arc::new(CacheService::new(config.cache.clone()).await);

    // Initialize Backup Service
    let mut backup_config = config.backup.clone();
    if backup_config.database_url.is_empty() {
        backup_config.database_url = config.database.url.clone();
    }
    let backup_service = Arc::new(BackupService::new(database.clone(), backup_config));
    if config.backup.interval_hours > 0 {
        let sched = backup_service.clone();
        tokio::spawn(async move { sched.run_scheduler().await; });
    }

    // Initialize Webhook Service
    let webhook_service = Arc::new(WebhookService::new(database.clone()));
    webhook_service.load_endpoints().await;

    let job_queue: Option<Arc<Mutex<JobQueue>>> = if !config.cache.redis_url.is_empty() {
        match JobQueue::new(&config.cache.redis_url).await {
            Ok(q) => Some(Arc::new(Mutex::new(q))),
            Err(e) => {
                warn!("Redis job queue unavailable: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Initialize event monitor
    let mut event_monitor = EventMonitor::new(
        config.stellar.clone(),
        database.clone(),
        ws_manager.clone(),
        fraud_service.clone(),
        notification_service.clone(),
        integration_service.clone(),
        webhook_service.clone(),
        job_queue.clone(),
    );

    // Start event monitoring in background
    let monitor_handle = tokio::spawn(async move {
        if let Err(e) = event_monitor.start().await {
            warn!("Event monitor error: {}", e);
        }
    });

    // Build application with routes
    let admin_router = Router::new()
        .route("/admin/rate-limits", get(get_rate_limit_stats))
        .route(
            "/admin/rate-limits/whitelist",
            post(add_to_whitelist).delete(remove_from_whitelist),
        )
        .route(
            "/admin/rate-limits/blacklist",
            post(add_to_blacklist).delete(remove_from_blacklist),
        )
        .route("/admin/rate-limits/tier", post(set_ip_tier))
        .with_state(rate_limiter.clone());
    let file_router = Router::new()
        .route("/files", get(list_files))
        .route("/files/:category", post(upload_file))
        .route("/files/:id", get(download_file).delete(delete_file))
        .with_state(storage_service);

    // Versioned API router (v1) - includes gateway-enhanced endpoints
    let v1_api = Router::new()
        .route("/", get(api_index))
        .route("/docs", get(api_docs))
        .route("/gateway/stats", get(gateway_stats))
        .route("/events", get(get_events))
        .route("/events/:id", get(get_event_by_id))
        .route("/events/trade/:trade_id", get(get_events_by_trade_id))
        .route("/events/type/:event_type", get(get_events_by_type))
        .route("/events/replay", post(replay_events))
        .route("/search", get(global_search))
        .route("/search/trades", get(search_trades))
        .route("/search/discovery", get(discover_entities))
        .route("/search/suggestions", get(search_suggestions))
        .route("/search/history", get(search_history))
        .route("/fraud/alerts", get(get_fraud_alerts))
        .route("/fraud/review", post(update_fraud_review))
        .route("/audit", post(create_audit_log))
        .route("/audit", get(query_audit_logs))
        .route("/audit/stats", get(audit_stats))
        .route("/audit/purge", delete(purge_audit_logs));

    let app = Router::new()
        .route("/", get(api_index))
        .route("/health", get(health::env_health))
        .route("/health/live", get(liveness))
        .route("/status", get(get_status))
        .route("/stats", get(get_stats))
        .route("/events", get(get_events))
        .route("/events/:id", get(get_event_by_id))
        .route("/events/trade/:trade_id", get(get_events_by_trade_id))
        .route("/events/type/:event_type", get(get_events_by_type))
        .route("/events/replay", post(replay_events))
        .route("/search", get(global_search))
        .route("/search/trades", get(search_trades))
        .route("/search/discovery", get(discover_entities))
        .route("/search/suggestions", get(search_suggestions))
        .route("/search/history", get(search_history))
        .route("/search/analytics", get(search_analytics))
        .route("/fraud/alerts", get(get_fraud_alerts))
        .route("/fraud/review", post(update_fraud_review))
        // Notifications
        .route(
            "/notifications/preferences/:address",
            get(get_notification_preferences).put(upsert_notification_preferences),
        )
        .route("/notifications/log/:address", get(get_notification_log))
        // Performance monitoring
        .route("/performance/dashboard", get(get_performance_dashboard))
        .route("/performance/alerts", get(get_performance_alerts))
        // Compliance
        .route("/compliance/check", post(run_compliance_check))
        .route("/compliance/status/:address", get(get_compliance_status))
        .route("/compliance/review", post(review_compliance_check))
        .route("/compliance/report", get(get_compliance_report))
        // Monitoring
        .route("/monitoring/dashboard", get(get_monitoring_dashboard))
        .route("/monitoring/alerts", get(get_monitoring_alerts))
        .route("/metrics", get(get_prometheus_metrics))
        // Integrations
        .route("/integrations/stats", get(get_integration_stats))
        .route("/integrations/log", get(get_integration_log))
        // Analytics
        .route("/analytics/dashboard", get(get_analytics_dashboard))
        .route("/analytics/realtime", get(get_analytics_realtime))
        .route("/analytics/export", get(export_analytics))
        // Cache
        .route("/cache/stats", get(get_cache_stats))
        .route("/cache/invalidate", post(invalidate_cache))
        // Backup
        .route("/backup/trigger", post(trigger_backup))
        .route("/backup/status", get(get_backup_status))
        .route("/backup/history", get(get_backup_history))
        .route("/backup/recovery-plan", get(get_recovery_plan))
        .route("/config/public", get(get_public_config))
        // Webhooks
        .route("/webhooks", post(register_webhook).get(list_webhooks))
        .route("/webhooks/:id", delete(deactivate_webhook))
        .route("/webhooks/deliveries", get(get_webhook_deliveries))
        .route("/webhooks/stats", get(get_webhook_stats))
        .route("/ws", get(ws_handler))
        .route("/help", get(help_index))
        .route("/help/faqs", get(get_faqs))
        .route("/help/tutorials", get(get_tutorials))
        .route("/help/tutorials/:id", get(get_tutorial_by_id))
        .route("/help/docs", get(get_docs))
        .route("/help/search", get(search_help))
        .route("/help/contact", get(get_contact))
        .route("/audit", post(create_audit_log))
        .route("/audit", get(query_audit_logs))
        .route("/audit/stats", get(audit_stats))
        .route("/audit/purge", delete(purge_audit_logs))
        .merge(admin_router)
        .merge(file_router)
        .merge(Router::new().nest("/api/v1", v1_api))
        .with_state(AppState {
            database,
            ws_manager,
            health: health_state,
            fraud_service,
            notification_service,
            gateway: gateway_state.clone(),
            performance_service,
            integration_service,
            compliance_service,
            monitoring_service,
            analytics_service,
            cache_service,
            backup_service,
            webhook_service,
            public_config: config.public_snapshot(),
        })
        // Apply gateway middleware for centralized routing and auth
        .layer(middleware::from_fn_with_state(
            gateway_state.clone(),
            gateway::gateway_middleware,
        ))
        // Apply rate limiting middleware
        .layer(middleware::from_fn_with_state(
            rate_limiter,
            rate_limit_middleware,
        ))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    // Wait for monitor to finish (shouldn't happen in normal operation)
    monitor_handle.await?;

    Ok(())
}

impl FromRef<AppState> for HealthState {
    fn from_ref(state: &AppState) -> Self {
        state.health.clone()
    }
}
