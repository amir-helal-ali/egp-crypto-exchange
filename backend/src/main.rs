//! EGP Closed-Loop Cryptocurrency Exchange — Backend entry point.
//!
//! Wires together: HTTP server (Axum), PostgreSQL (SQLx), Redis (queue + pub/sub),
//! in-memory matching engine, Binance WebSocket client with circuit breaker.

use std::sync::Arc;

use axum::serve;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing_subscriber::EnvFilter;

mod api;
mod auth;
mod binance;
mod config;
mod db;
mod error;
mod matching_engine;
mod models;
mod redis;
mod services;
mod ws_bus;

use api::build_router;
use auth::JwtConfig;
use binance::{BinanceClient, CircuitBreaker};
use matching_engine::Engine;
use services::{load_trade_pairs, Fees};

/// Shared application state — cheaply cloneable via Arc.
pub struct AppState {
    pub config: config::Config,
    pub db: sqlx::PgPool,
    pub queue: redis::SharedQueue,
    pub jwt: auth::JwtConfig,
    pub engine: Arc<Engine>,
    pub engine_bcast: broadcast::Sender<matching_engine::EngineEvent>,
    pub engine_event_rx: Mutex<Option<mpsc::UnboundedReceiver<matching_engine::EngineEvent>>>,
    pub binance: Arc<BinanceClient>,
    pub trade_pairs: services::SharedTradePairs,
    pub fees: Fees,
    pub ws_bus: ws_bus::SharedWsBus,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment.
    dotenvy::from_filename(".env").ok();
    let config = config::Config::from_env()?;

    // Init logging.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new(&config.log_level).unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(true)
        .with_file(false)
        .with_line_number(false)
        .init();

    tracing::info!(host = %config.server_host, port = config.server_port, "starting EGP exchange backend");

    // DB pool + migrations.
    let pool = db::connect(&config.database_url, config.database_max_connections).await?;
    db::migrate(&pool).await?;
    db::users::ensure_admin_bootstrap(&pool, &config.admin_bootstrap_email).await?;

    // Redis queue.
    let queue = Arc::new(redis::RedisQueue::connect(&config.redis_url, &config.queue_prefix).await?);

    // Trade pairs + fees.
    let pairs = load_trade_pairs(&pool).await?;
    let trade_pairs: services::SharedTradePairs = Arc::new(parking_lot::RwLock::new(pairs));
    let fees = Fees::default();

    // JWT config.
    let jwt = JwtConfig::new(
        config.jwt_secret.clone(),
        config.jwt_access_ttl_hours,
        config.jwt_refresh_ttl_days,
    );

    // Circuit breaker + Binance client.
    let breaker = Arc::new(CircuitBreaker::new(
        config.circuit_breaker_timeout(),
        config.circuit_breaker_max_failures,
    ));
    let binance = Arc::new(BinanceClient::new(config.binance_combined_stream_url(), breaker.clone()));

    // Matching engine.
    let (engine_event_tx, engine_event_rx) = mpsc::unbounded_channel::<matching_engine::EngineEvent>();
    let (engine_bcast, _) = broadcast::channel::<matching_engine::EngineEvent>(1024);
    let engine = Arc::new(Engine::new(engine_event_tx, engine_bcast.clone()));

    // WebSocket event bus for per-user events.
    let ws_bus = Arc::new(ws_bus::WsBus::new());

    // App state.
    let state = Arc::new(AppState {
        config: config.clone(),
        db: pool.clone(),
        queue: queue.clone(),
        jwt,
        engine: engine.clone(),
        engine_bcast: engine_bcast.clone(),
        engine_event_rx: Mutex::new(Some(engine_event_rx)),
        binance: binance.clone(),
        trade_pairs: trade_pairs.clone(),
        fees: fees.clone(),
        ws_bus: ws_bus.clone(),
    });

    // Spawn background tasks.
    let binance_clone = binance.clone();
    tokio::spawn(async move {
        binance_clone.run().await;
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        api::trading::engine_event_consumer(state_clone).await;
    });

    // مهمة تحديث أسعار السوق للعقود الآجلة + فحص التصفية
    let state_clone2 = state.clone();
    tokio::spawn(async move {
        api::futures::mark_price_updater(state_clone2).await;
    });

    // HTTP server.
    let router = build_router(state.clone());
    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "HTTP server listening");

    // Graceful shutdown on Ctrl-C.
    let shutdown = async move {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("received Ctrl-C, shutting down");
    };

    serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await?;

    tracing::info!("server stopped");
    Ok(())
}
