//! API routes module.

pub mod auth;
pub mod user;
pub mod trading;
pub mod wallet;
pub mod admin;
pub mod ws;
pub mod futures;
pub mod p2p;
pub mod settings;

use std::sync::Arc;

use axum::extract::State;
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
        .route("/api/market/orderbook/{pair}", get(trading::public_orderbook))
        .route("/api/market/trades/{pair}", get(trading::public_recent_trades))
        .route("/api/market/circuit", get(trading::circuit_status))
        .route("/api/market/ws", get(ws::market_ws_handler))

        // Authenticated user routes
        .route("/api/user/me", get(user::me))
        .route("/api/user/wallets", get(wallet::list_wallets))
        .route("/api/user/orders", get(trading::list_orders).post(trading::place_order))
        .route("/api/user/orders/{id}", delete(trading::cancel_order))
        .route("/api/user/trades", get(trading::list_my_trades))
        .route("/api/user/deposits", post(wallet::request_deposit).get(wallet::list_my_deposits))
        .route("/api/user/withdrawals", post(wallet::request_withdrawal).get(wallet::list_my_withdrawals))
        .route("/api/user/withdrawals/status", get(wallet::withdrawal_status_ws))
        .route("/api/user/ledger", get(wallet::ledger))

        // Futures - العقود الآجلة
        .route("/api/futures/positions", get(futures::list_positions).post(futures::open_position))
        .route("/api/futures/positions/{id}/close", post(futures::close_position))

        // P2P - التداول بين الأفراد
        .route("/api/p2p/offers", get(p2p::list_offers).post(p2p::create_offer))
        .route("/api/p2p/offers/mine", get(p2p::list_my_offers))
        .route("/api/p2p/offers/{id}", get(p2p::get_offer))
        .route("/api/p2p/offers/{id}/status", post(p2p::update_offer_status))
        .route("/api/p2p/trades", post(p2p::start_trade).get(p2p::list_my_trades))
        .route("/api/p2p/trades/{id}", get(p2p::get_trade))
        .route("/api/p2p/trades/{id}/paid", post(p2p::confirm_paid))
        .route("/api/p2p/trades/{id}/release", post(p2p::release_crypto))
        .route("/api/p2p/trades/{id}/cancel", post(p2p::cancel_trade))
        .route("/api/p2p/trades/{id}/messages", get(p2p::list_messages).post(p2p::send_message))
        .route("/api/p2p/overview", get(p2p::p2p_overview))

        // Admin routes (separate auth scope)
        .route("/api/admin/overview", get(admin::overview))
        .route("/api/admin/users", get(admin::list_users))
        .route("/api/admin/users/{id}", get(admin::get_user))
        .route("/api/admin/users/{id}/status", put(admin::update_user_status))
        .route("/api/admin/manual_tx", get(admin::list_manual_tx))
        .route("/api/admin/manual_tx/{id}", get(admin::get_manual_tx))
        .route("/api/admin/manual_tx/{id}/review", post(admin::review_manual_tx))
        .route("/api/admin/liquidity", get(admin::liquidity))
        .route("/api/admin/orders", get(admin::list_all_orders))
        .route("/api/admin/trades", get(admin::list_all_trades))
        .route("/api/admin/audit", get(admin::audit_log))
        .route("/api/admin/futures/positions", get(admin_futures_positions))

        // Admin P2P management - إدارة صفقات وعروض P2P
        .route("/api/admin/p2p/trades", get(p2p::admin_list_all_trades))
        .route("/api/admin/p2p/offers", get(p2p::admin_list_all_offers))

        // Admin wallet management - إدارة المحافظ
        .route("/api/admin/wallets/adjust", post(admin::adjust_wallet))
        .route("/api/admin/users/{id}/wallets", get(admin::list_user_wallets))

        // Admin currency & pair management - إدارة العملات والأزواج
        .route("/api/admin/currencies", get(settings::list_currencies).post(settings::create_currency))
        .route("/api/admin/currencies/{id}", put(settings::update_currency).delete(settings::delete_currency))
        .route("/api/admin/pairs", get(settings::list_pairs).post(settings::create_pair))
        .route("/api/admin/pairs/{id}", put(settings::update_pair).delete(settings::delete_pair))
        .route("/api/admin/settings", get(settings::get_settings).put(settings::update_settings))

        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

/// Helper: قائمة كل المراكز المفتوحة (admin)
async fn admin_futures_positions(
    _admin: crate::auth::AdminUser,
    State(state): State<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    let positions = crate::db::futures::list_all_open(&state.db).await.unwrap_or_default();
    axum::Json(serde_json::json!({ "positions": positions, "count": positions.len() }))
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

async fn health_ready() -> &'static str {
    "READY"
}
