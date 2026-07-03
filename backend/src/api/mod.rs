//! API routes module.

pub mod auth;
pub mod user;
pub mod trading;
pub mod wallet;
pub mod admin;
pub mod ws;

use std::sync::Arc;

use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::AppState;

/// Build the entire HTTP router.
pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = build_cors(&state.config);

    Router::new()
        // Health
        .route("/health", get(health))
        .route("/health/ready", get(health_ready))

        // Auth (public)
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/refresh", post(auth::refresh))

        // Public market data
        .route("/api/market/tickers", get(trading::public_tickers))
        .route("/api/market/orderbook/:pair", get(trading::public_orderbook))
        .route("/api/market/trades/:pair", get(trading::public_recent_trades))
        .route("/api/market/circuit", get(trading::circuit_status))
        .route("/api/market/ws", get(ws::market_ws_handler))

        // Authenticated user routes
        .route("/api/user/me", get(user::me))
        .route("/api/user/wallets", get(wallet::list_wallets))
        .route("/api/user/orders", get(trading::list_orders).post(trading::place_order))
        .route("/api/user/orders/:id", delete(trading::cancel_order))
        .route("/api/user/trades", get(trading::list_my_trades))
        .route("/api/user/deposits", post(wallet::request_deposit).get(wallet::list_my_deposits))
        .route("/api/user/withdrawals", post(wallet::request_withdrawal).get(wallet::list_my_withdrawals))
        .route("/api/user/withdrawals/status", get(wallet::withdrawal_status_ws))
        .route("/api/user/ledger", get(wallet::ledger))

        // Admin routes (separate auth scope)
        .route("/api/admin/overview", get(admin::overview))
        .route("/api/admin/users", get(admin::list_users))
        .route("/api/admin/users/:id", get(admin::get_user))
        .route("/api/admin/users/:id/status", put(admin::update_user_status))
        .route("/api/admin/manual_tx", get(admin::list_manual_tx))
        .route("/api/admin/manual_tx/:id", get(admin::get_manual_tx))
        .route("/api/admin/manual_tx/:id/review", post(admin::review_manual_tx))
        .route("/api/admin/liquidity", get(admin::liquidity))
        .route("/api/admin/orders", get(admin::list_all_orders))
        .route("/api/admin/trades", get(admin::list_all_trades))
        .route("/api/admin/audit", get(admin::audit_log))

        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

fn build_cors(config: &crate::config::Config) -> tower_http::cors::CorsLayer {
    use axum::http::HeaderValue;
    use tower_http::cors::AllowOrigin;

    let origins: Vec<HeaderValue> = [
        &config.user_frontend_origin,
        &config.admin_frontend_origin,
    ]
    .iter()
    .filter_map(|s| s.parse::<HeaderValue>().ok())
    .collect();

    let allow_origin = if origins.is_empty() {
        AllowOrigin::any()
    } else {
        AllowOrigin::list(origins)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(false)
        .max_age(Duration::from_secs(3600))
}

use std::time::Duration;

async fn health() -> &'static str {
    "OK"
}

async fn health_ready(state: axum::extract::State<Arc<AppState>>) -> &'static str {
    // Verify DB and Redis are reachable.
    let _ = sqlx::query("SELECT 1").execute(&state.db).await;
    let _ = redis::cmd("PING").query_async::<String>(&mut state.queue.conn.clone()).await;
    "READY"
}
