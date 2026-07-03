//! API إدارة العملات والأزواج - Admin currency & pair management

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::auth::AdminUser;
use crate::db;
use crate::error::AppResult;
use crate::models::{Currency, CurrencyType, TradingPair};
use crate::AppState;

// --- العملات - Currencies -------------------------------------------------

pub async fn list_currencies(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<Currency>>> {
    let rows = db::currencies::list_currencies(&state.db, false).await?;
    Ok(Json(rows))
}

pub async fn create_currency(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCurrencyRequest>,
) -> AppResult<Json<Currency>> {
    let row = db::currencies::create_currency(
        &state.db,
        &req.symbol,
        &req.name,
        req.r#type,
        req.precision.unwrap_or(8),
        req.withdraw_fee.unwrap_or_default(),
        req.min_withdrawal.unwrap_or_default(),
        req.network.as_deref(),
        req.is_active.unwrap_or(true),
    )
    .await?;
    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct CreateCurrencyRequest {
    pub symbol: String,
    pub name: String,
    pub r#type: CurrencyType,
    pub precision: Option<i16>,
    pub withdraw_fee: Option<Decimal>,
    pub min_withdrawal: Option<Decimal>,
    pub network: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn update_currency(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCurrencyRequest>,
) -> AppResult<Json<Currency>> {
    let row = db::currencies::update_currency(
        &state.db,
        id,
        req.name.as_deref(),
        req.precision,
        req.withdraw_fee,
        req.min_withdrawal,
        req.network.as_deref().map(Some).flatten().map(|s| Some(s)).unwrap_or(None),
        req.is_active,
    )
    .await?;
    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct UpdateCurrencyRequest {
    pub name: Option<String>,
    pub precision: Option<i16>,
    pub withdraw_fee: Option<Decimal>,
    pub min_withdrawal: Option<Decimal>,
    pub network: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn delete_currency(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    db::currencies::delete_currency(&state.db, id).await?;
    Ok(Json(serde_json::json!({"deleted": id})))
}

// --- أزواج التداول - Trading pairs ---------------------------------------

pub async fn list_pairs(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<TradingPair>>> {
    let rows = db::currencies::list_pairs(&state.db, false).await?;
    Ok(Json(rows))
}

pub async fn create_pair(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePairRequest>,
) -> AppResult<Json<TradingPair>> {
    let row = db::currencies::create_pair(
        &state.db,
        &req.pair,
        &req.base_asset,
        &req.quote_asset,
        &req.binance_symbol,
        req.is_spot_active.unwrap_or(true),
        req.is_futures_active.unwrap_or(false),
        req.maker_fee_bps.unwrap_or(10),
        req.taker_fee_bps.unwrap_or(20),
        req.min_order_qty.unwrap_or_else(|| Decimal::new(1, 4)),
        req.price_precision.unwrap_or(2),
        req.qty_precision.unwrap_or(8),
        req.sort_order.unwrap_or(0),
    )
    .await?;

    // إعادة تحميل أزواج التداول في الذاكرة
    let pairs = db::currencies::list_pairs(&state.db, true).await?;
    *state.trade_pairs.write() = pairs;

    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct CreatePairRequest {
    pub pair: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub binance_symbol: String,
    pub is_spot_active: Option<bool>,
    pub is_futures_active: Option<bool>,
    pub maker_fee_bps: Option<i32>,
    pub taker_fee_bps: Option<i32>,
    pub min_order_qty: Option<Decimal>,
    pub price_precision: Option<i16>,
    pub qty_precision: Option<i16>,
    pub sort_order: Option<i32>,
}

pub async fn update_pair(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePairRequest>,
) -> AppResult<Json<TradingPair>> {
    let row = db::currencies::update_pair(
        &state.db,
        id,
        req.binance_symbol.as_deref(),
        req.is_spot_active,
        req.is_futures_active,
        req.maker_fee_bps,
        req.taker_fee_bps,
        req.min_order_qty,
        req.price_precision,
        req.qty_precision,
        req.sort_order,
        req.is_active,
    )
    .await?;

    // إعادة تحميل أزواج التداول في الذاكرة
    let pairs = db::currencies::list_pairs(&state.db, true).await?;
    *state.trade_pairs.write() = pairs;

    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePairRequest {
    pub binance_symbol: Option<String>,
    pub is_spot_active: Option<bool>,
    pub is_futures_active: Option<bool>,
    pub maker_fee_bps: Option<i32>,
    pub taker_fee_bps: Option<i32>,
    pub min_order_qty: Option<Decimal>,
    pub price_precision: Option<i16>,
    pub qty_precision: Option<i16>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

pub async fn delete_pair(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    db::currencies::delete_pair(&state.db, id).await?;

    let pairs = db::currencies::list_pairs(&state.db, true).await?;
    *state.trade_pairs.write() = pairs;

    Ok(Json(serde_json::json!({"deleted": id})))
}

// --- إعدادات النظام - System settings ------------------------------------

pub async fn get_settings(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    let egp_usd_rate = db::settings::get(&state.db, "egp_usd_rate").await?;
    let min_deposit = db::settings::get(&state.db, "min_egp_deposit").await?;
    let min_withdrawal = db::settings::get(&state.db, "min_egp_withdrawal").await?;
    Ok(Json(serde_json::json!({
        "egp_usd_rate": egp_usd_rate,
        "min_egp_deposit": min_deposit,
        "min_egp_withdrawal": min_withdrawal,
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub egp_usd_rate: Option<Decimal>,
    pub min_egp_deposit: Option<Decimal>,
    pub min_egp_withdrawal: Option<Decimal>,
}

pub async fn update_settings(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateSettingsRequest>,
) -> AppResult<Json<Value>> {
    if let Some(rate) = req.egp_usd_rate {
        db::settings::set(&state.db, "egp_usd_rate", &serde_json::json!(rate.to_string())).await?;
    }
    if let Some(min) = req.min_egp_deposit {
        db::settings::set(&state.db, "min_egp_deposit", &serde_json::json!(min.to_string())).await?;
    }
    if let Some(min) = req.min_egp_withdrawal {
        db::settings::set(&state.db, "min_egp_withdrawal", &serde_json::json!(min.to_string())).await?;
    }
    Ok(Json(serde_json::json!({"updated": true})))
}
