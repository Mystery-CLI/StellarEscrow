use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    response::Json,
    routing::{delete, get, post},
    extract::FromRef,
    middleware,
    routing::{delete, get, post},
    routing::{get, post},
    Router,
};
use clap::Parser;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod auth;
mod config;
mod database;
mod error;
mod event_monitor;
mod file_handlers;
mod handlers;
mod health;
mod help;
mod models;
mod rate_limit;
mod rate_limit_handlers;
mod storage;
mod websocket;
mod fraud_service;
mod notification_service;

#[cfg(test)]
mod test;

use config::Config;
use database::Database;
use event_monitor::EventMonitor;
use auth::auth_middleware;
use handlers::{AppState, *};
use health::{alerts, liveness, metrics, readiness, status_page, HealthMonitor, HealthState};
use file_handlers::{delete_file, download_file, list_files, upload_file};
use handlers::*;
use rate_limit::RateLimiter;
use rate_limit_handlers::*;
use storage::StorageService;
use websocket::WebSocketManager;
use help::{
    get_contact, get_docs, get_faqs, get_tutorial_by_id, get_tutorials, help_index, search_help,
};
use fraud_service::FraudDetectionService;
use notification_service::NotificationService;

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

    // Load configuration
    let config = Config::load(&args.config)?;
    info!("Loaded configuration from {}", args.config);

    // Initialize database
    let db_pool = PgPool::connect(&config.database.url).await?;
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
    // Initialize file storage service
    let storage_service = Arc::new(
        StorageService::new(db_pool, &config.storage.base_dir).await?,
    );
    // Initialize Fraud Detection Service
    let fraud_service = Arc::new(FraudDetectionService::new(database.clone()).await);

    // Initialize Notification Service
    let notification_service = Arc::new(NotificationService::new(
        database.clone(),
        config.notification.clone(),
    ));

    // Initialize event monitor
    let event_monitor = EventMonitor::new(
        config.stellar.clone(),
        database.clone(),
        ws_manager.clone(),
        fraud_service.clone(),
        notification_service.clone(),
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
        .route("/admin/rate-limits/whitelist", post(add_to_whitelist).delete(remove_from_whitelist))
        .route("/admin/rate-limits/blacklist", post(add_to_blacklist).delete(remove_from_blacklist))
        .route("/admin/rate-limits/tier", post(set_ip_tier))
        .with_state(rate_limiter.clone());
    let file_router = Router::new()
        .route("/files", get(list_files))
        .route("/files/:category", post(upload_file))
        .route("/files/:id", get(download_file).delete(delete_file))
        .with_state(storage_service);

    // Versioned API router (v1)
    let v1_api = Router::new()
        .route("/", get(api_index))
        .route("/docs", get(api_docs))
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
        .route("/health", get(liveness))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .route("/health/metrics", get(metrics))
        .route("/health/alerts", get(alerts))
        .route("/status", get(status_page))
        .route("/health", get(health_check))
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
        .route("/notifications/preferences/:address", get(get_notification_preferences).put(upsert_notification_preferences))
        .route("/notifications/log/:address", get(get_notification_log))
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
        })
        .layer(middleware::from_fn_with_state(auth_config, auth_middleware))
        .layer(middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    // Wait for monitor to finish (shouldn't happen in normal operation)
    monitor_handle.await?;

    Ok(())
}

impl FromRef<AppState> for HealthState {
    fn from_ref(state: &AppState) -> Self {
        state.health.clone()
    }
}