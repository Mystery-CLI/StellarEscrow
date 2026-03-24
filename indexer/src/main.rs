use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod config;
mod database;
mod error;
mod event_monitor;
mod file_handlers;
mod handlers;
mod help;
mod models;
mod storage;
mod websocket;

#[cfg(test)]
mod test;

use config::Config;
use database::Database;
use event_monitor::EventMonitor;
use file_handlers::{delete_file, download_file, list_files, upload_file};
use handlers::*;
use storage::StorageService;
use websocket::WebSocketManager;
use help::{
    get_contact, get_docs, get_faqs, get_tutorial_by_id, get_tutorials, help_index, search_help,
};

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

    // Initialize WebSocket manager
    let (tx, _rx) = broadcast::channel(100);
    let ws_manager = Arc::new(WebSocketManager::new(tx.clone()));

    // Initialize file storage service
    let storage_service = Arc::new(
        StorageService::new(db_pool, &config.storage.base_dir).await?,
    );

    // Initialize event monitor
    let event_monitor = EventMonitor::new(
        config.stellar.clone(),
        database.clone(),
        ws_manager.clone(),
    );

    // Start event monitoring in background
    let monitor_handle = tokio::spawn(async move {
        if let Err(e) = event_monitor.start().await {
            warn!("Event monitor error: {}", e);
        }
    });

    // Build application with routes
    let file_router = Router::new()
        .route("/files", get(list_files))
        .route("/files/:category", post(upload_file))
        .route("/files/:id", get(download_file).delete(delete_file))
        .with_state(storage_service);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/events", get(get_events))
        .route("/events/:id", get(get_event_by_id))
        .route("/events/trade/:trade_id", get(get_events_by_trade_id))
        .route("/events/type/:event_type", get(get_events_by_type))
        .route("/events/replay", post(replay_events))
        .route("/ws", get(ws_handler))
        // Help center
        .route("/help", get(help_index))
        .route("/help/faqs", get(get_faqs))
        .route("/help/tutorials", get(get_tutorials))
        .route("/help/tutorials/:id", get(get_tutorial_by_id))
        .route("/help/docs", get(get_docs))
        .route("/help/search", get(search_help))
        .route("/help/contact", get(get_contact))
        .with_state(AppState {
            database,
            ws_manager,
        })
        .merge(file_router)
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    // Wait for monitor to finish (shouldn't happen in normal operation)
    monitor_handle.await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    database: Arc<Database>,
    ws_manager: Arc<WebSocketManager>,
}