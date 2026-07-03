//! API التداول بين الأفراد - P2P endpoints

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{P2PMessage, P2POffer, P2PSide, P2PTrade, P2PTradeStatus};
use crate::AppState;

// --- العروض - Offers -------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateOfferRequest {
    pub side: P2PSide,
    pub asset_symbol: String,
    pub price_margin_pct: Decimal,
    pub min_amount_egp: Decimal,
    pub max_amount_egp: Decimal,
    pub payment_methods: Vec<String>,
    pub time_limit_min: i32,
}

pub async fn list_offers(
    State(state): State<Arc<AppState>>,
    Query(q): Query<P2POfferQuery>,
) -> AppResult<Json<Vec<P2POffer>>> {
    let offers = db::p2p::list_offers(
        &state.db,
        q.side,
        q.asset.as_deref(),
        q.payment_method.as_deref(),
    )
    .await?;
    Ok(Json(offers))
}

#[derive(Debug, Deserialize)]
pub struct P2POfferQuery {
    pub side: Option<P2PSide>,
    pub asset: Option<String>,
    pub payment_method: Option<String>,
}

pub async fn get_offer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<P2POffer>> {
    let offer = db::p2p::get_offer(&state.db, id).await?;
    Ok(Json(offer))
}

pub async fn create_offer(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateOfferRequest>,
) -> AppResult<Json<P2POffer>> {
    if req.payment_methods.is_empty() {
        return Err(AppError::BadRequest("يجب اختيار طريقة دفع واحدة على الأقل".into()));
    }
    if req.min_amount_egp <= Decimal::ZERO || req.max_amount_egp < req.min_amount_egp {
        return Err(AppError::BadRequest("الحدود غير صحيحة".into()));
    }
    if req.time_limit_min < 5 || req.time_limit_min > 240 {
        return Err(AppError::BadRequest("المهلة الزمنية يجب أن تكون بين 5 و 240 دقيقة".into()));
    }
    let offer = db::p2p::create_offer(
        &state.db,
        auth.user_id,
        req.side,
        &req.asset_symbol,
        req.price_margin_pct,
        req.min_amount_egp,
        req.max_amount_egp,
        &req.payment_methods,
        req.time_limit_min,
    )
    .await?;
    Ok(Json(offer))
}

// --- الصفقات - Trades -----------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct StartTradeRequest {
    pub offer_id: Uuid,
    pub amount: Decimal,
    pub payment_method: String,
}

pub async fn start_trade(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartTradeRequest>,
) -> AppResult<Json<P2PTrade>> {
    let offer = db::p2p::get_offer(&state.db, req.offer_id).await?;
    if offer.user_id == auth.user_id {
        return Err(AppError::BadRequest("لا يمكنك البدء بصفقة على عرضك".into()));
    }
    if !offer.payment_methods.contains(&req.payment_method) {
        return Err(AppError::BadRequest("طريقة الدفع غير مدعومة في هذا العرض".into()));
    }

    // حساب السعر الإجمالي
    let ticker = state
        .binance
        .latest_ticker(&format!("{}USDT", offer.asset_symbol))
        .or_else(|| state.binance.latest_ticker("BTCUSDT"));
    let market_price = match ticker {
        Some(t) => crate::api::trading::derive_egp_price(
            &state,
            &format!("{}USDT", offer.asset_symbol),
            t,
        ),
        None => state.config.egp_usd_rate, // fallback
    };
    let price_egp = market_price * (Decimal::from(1) + offer.price_margin_pct / Decimal::from(100));
    let total_egp = price_egp * req.amount;

    // التحقق من الحدود
    if total_egp < offer.min_amount_egp || total_egp > offer.max_amount_egp {
        return Err(AppError::BadRequest(format!(
            "المبلغ يجب أن يكون بين {} و {} جنيه",
            offer.min_amount_egp, offer.max_amount_egp
        )));
    }

    // تحديد المشتري والبائع
    let (buyer_id, seller_id) = match offer.side {
        P2PSide::Buy => (offer.user_id, auth.user_id),   // صاحب العرض يشتري، المستخدم يبيع
        P2PSide::Sell => (auth.user_id, offer.user_id),  // صاحب العرض يبيع، المستخدم يشتري
    };

    // حجز العملة الرقمية من البائع (escrow)
    let _ = db::wallets::lock_balance(
        &state.db,
        seller_id,
        &offer.asset_symbol,
        req.amount,
        None,
        "p2p_escrow",
    )
    .await?;

    let trade = db::p2p::create_trade(
        &state.db,
        req.offer_id,
        buyer_id,
        seller_id,
        &offer.asset_symbol,
        req.amount,
        price_egp,
        total_egp,
        &req.payment_method,
    )
    .await?;

    Ok(Json(trade))
}

pub async fn get_trade(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<P2PTrade>> {
    let trade = db::p2p::get_trade(&state.db, id).await?;
    if trade.buyer_id != auth.user_id && trade.seller_id != auth.user_id {
        return Err(AppError::Forbidden("ليست صفقتك".into()));
    }
    Ok(Json(trade))
}

pub async fn list_my_trades(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<P2PTrade>>> {
    let trades = db::p2p::list_my_trades(&state.db, auth.user_id).await?;
    Ok(Json(trades))
}

pub async fn confirm_paid(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<P2PTrade>> {
    let trade = db::p2p::get_trade(&state.db, id).await?;
    if trade.buyer_id != auth.user_id {
        return Err(AppError::Forbidden("المشتري فقط يستطيع تأكيد الدفع".into()));
    }
    if trade.status != P2PTradeStatus::Pending {
        return Err(AppError::BadRequest("الحالة لا تسمح بالتأكيد".into()));
    }
    let updated = db::p2p::update_trade_status(&state.db, id, P2PTradeStatus::Paid).await?;
    Ok(Json(updated))
}

pub async fn release_crypto(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<P2PTrade>> {
    let trade = db::p2p::get_trade(&state.db, id).await?;
    if trade.seller_id != auth.user_id {
        return Err(AppError::Forbidden("البائع فقط يستطيع إطلاق العملات".into()));
    }
    if trade.status != P2PTradeStatus::Paid {
        return Err(AppError::BadRequest("يجب تأكيد الدفع أولاً".into()));
    }

    // تحويل العملة الرقمية من البائع إلى المشتري
    let _ = db::wallets::unlock_balance(
        &state.db,
        trade.seller_id,
        &trade.asset_symbol,
        trade.amount,
        None,
        "p2p_release",
    )
    .await?;
    let _ = db::wallets::credit(
        &state.db,
        trade.buyer_id,
        &trade.asset_symbol,
        trade.amount,
        Some(id),
        "p2p_received",
    )
    .await?;

    // إضافة الجنيه إلى البائع (من محفظة المشتري)
    let _ = db::wallets::debit(
        &state.db,
        trade.buyer_id,
        "EGP",
        trade.total_egp,
        Some(id),
        "p2p_paid",
    )
    .await?;
    let _ = db::wallets::credit(
        &state.db,
        trade.seller_id,
        "EGP",
        trade.total_egp,
        Some(id),
        "p2p_received_egp",
    )
    .await?;

    let updated = db::p2p::update_trade_status(&state.db, id, P2PTradeStatus::Completed).await?;
    Ok(Json(updated))
}

pub async fn cancel_trade(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<P2PTrade>> {
    let trade = db::p2p::get_trade(&state.db, id).await?;
    if trade.buyer_id != auth.user_id && trade.seller_id != auth.user_id {
        return Err(AppError::Forbidden("ليست صفقتك".into()));
    }
    if trade.status != P2PTradeStatus::Pending && trade.status != P2PTradeStatus::Paid {
        return Err(AppError::BadRequest("لا يمكن إلغاء الصفقة في هذه الحالة".into()));
    }

    // استرجاع العملة الرقمية المحجوزة إلى البائع
    let _ = db::wallets::unlock_balance(
        &state.db,
        trade.seller_id,
        &trade.asset_symbol,
        trade.amount,
        None,
        "p2p_cancel_refund",
    )
    .await?;

    let updated = db::p2p::update_trade_status(&state.db, id, P2PTradeStatus::Cancelled).await?;
    Ok(Json(updated))
}

// --- الرسائل - Messages ---------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub message: String,
}

pub async fn list_messages(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(trade_id): Path<Uuid>,
) -> AppResult<Json<Vec<P2PMessage>>> {
    let trade = db::p2p::get_trade(&state.db, trade_id).await?;
    if trade.buyer_id != auth.user_id && trade.seller_id != auth.user_id {
        return Err(AppError::Forbidden("ليست صفقتك".into()));
    }
    let msgs = db::p2p::list_messages(&state.db, trade_id).await?;
    Ok(Json(msgs))
}

pub async fn send_message(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(trade_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> AppResult<Json<P2PMessage>> {
    let trade = db::p2p::get_trade(&state.db, trade_id).await?;
    if trade.buyer_id != auth.user_id && trade.seller_id != auth.user_id {
        return Err(AppError::Forbidden("ليست صفقتك".into()));
    }
    if req.message.trim().is_empty() {
        return Err(AppError::BadRequest("الرسالة فارغة".into()));
    }
    let msg = db::p2p::send_message(&state.db, trade_id, auth.user_id, &req.message).await?;
    Ok(Json(msg))
}

// JSON helper for overview
pub async fn p2p_overview(State(state): State<Arc<AppState>>, auth: AuthUser) -> AppResult<Json<Value>> {
    let trades = db::p2p::list_my_trades(&state.db, auth.user_id).await?;
    let total = trades.len();
    let completed = trades.iter().filter(|t| t.status == P2PTradeStatus::Completed).count();
    let completion_rate = if total > 0 { (completed * 100 / total) as u32 } else { 100 };

    Ok(Json(json!({
        "total_trades": total,
        "completed": completed,
        "completion_rate": completion_rate,
    })))
}
