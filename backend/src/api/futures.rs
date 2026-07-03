//! API العقود الآجلة - Futures trading endpoints
//!
//! منطق فتح المركز:
//! 1. تحقق من قاطع الدائرة (circuit breaker).
//! 2. احصل على سعر السوق من Binance.
//! 3. احسب حجم المركز = الهامش × الرافعة.
//! 4. احسب سعر التصفية بناءً على الجهة والرافعة.
//! 5. خصم الهامش من محفظة المستخدم (lock).
//! 6. أدخل صف المركز في DB.
//! 7. بث تحديث المركز عبر WebSocket.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{FuturesPosition, MarginMode, PositionSide};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct OpenPositionRequest {
    pub pair: String,
    pub side: PositionSide,
    pub margin_mode: MarginMode,
    pub leverage: i32,
    pub margin: Decimal,
}

pub async fn open_position(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<OpenPositionRequest>,
) -> AppResult<Json<FuturesPosition>> {
    if state.binance.breaker.is_open() {
        return Err(AppError::CircuitBreakerOpen);
    }
    if req.leverage < 1 || req.leverage > 125 {
        return Err(AppError::BadRequest("الرافعة يجب أن تكون بين 1 و 125".into()));
    }
    if req.margin <= Decimal::ZERO {
        return Err(AppError::BadRequest("الهامش يجب أن يكون موجباً".into()));
    }

    // تحقق من الزوج
    let pair_meta = state
        .trade_pairs
        .read()
        .iter()
        .find(|p| p.pair == req.pair && p.is_futures_active)
        .cloned()
        .ok_or_else(|| AppError::BadRequest(format!("العقود الآجلة غير مفعلة لـ {}", req.pair)))?;

    // احصل على سعر السوق
    let ticker = state
        .binance
        .latest_ticker(&pair_meta.binance_symbol)
        .ok_or_else(|| AppError::BadRequest("لا يوجد سعر سوق حالي".into()))?;
    let mark_price = crate::api::trading::derive_egp_price(&state, &pair_meta.binance_symbol, ticker);

    // حساب حجم المركز
    let quantity = req.margin * Decimal::from(req.leverage);

    // حساب سعر التصفية
    // للمركز الطويل: liquidation = entry * (1 - 1/leverage + maintenance_margin)
    // للمركز القصير: liquidation = entry * (1 + 1/leverage - maintenance_margin)
    let maintenance_margin_rate = dec!(0.005); // 0.5%
    let liquidation_price = match req.side {
        PositionSide::Long => {
            mark_price * (Decimal::from(1) - Decimal::from(1) / Decimal::from(req.leverage) + maintenance_margin_rate)
        }
        PositionSide::Short => {
            mark_price * (Decimal::from(1) + Decimal::from(1) / Decimal::from(req.leverage) - maintenance_margin_rate)
        }
    };

    // خصم الهامش من محفظة المستخدم
    let quote = &pair_meta.quote;
    let _ = db::wallets::lock_balance(
        &state.db,
        auth.user_id,
        quote,
        req.margin,
        None,
        "futures_margin",
    )
    .await?;

    // فتح المركز
    let position = db::futures::open(
        &state.db,
        auth.user_id,
        &req.pair,
        req.side,
        req.margin_mode,
        req.leverage,
        req.margin,
        quantity,
        mark_price,
        mark_price,
        liquidation_price,
    )
    .await?;

    // بث التحديث عبر WebSocket
    let _ = state.engine_bcast.send(crate::matching_engine::EngineEvent::BookUpdate {
        pair: format!("futures:{}", req.pair),
    });

    Ok(Json(position))
}

pub async fn close_position(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let position = db::futures::get(&state.db, id).await?;
    if position.user_id != auth.user_id {
        return Err(AppError::Forbidden("ليس مركزك".into()));
    }
    if position.status != crate::models::PositionStatus::Open {
        return Err(AppError::BadRequest("المركز ليس مفتوحاً".into()));
    }

    // احصل على السعر الحالي
    let pair_meta = state
        .trade_pairs
        .read()
        .iter()
        .find(|p| p.pair == position.pair)
        .cloned()
        .ok_or_else(|| AppError::Internal("pair missing".into()))?;

    let ticker = state
        .binance
        .latest_ticker(&pair_meta.binance_symbol)
        .ok_or_else(|| AppError::BadRequest("لا يوجد سعر سوق".into()))?;
    let close_price = crate::api::trading::derive_egp_price(&state, &pair_meta.binance_symbol, ticker);

    // حساب الربح/الخسارة المحقق
    let realized_pnl = match position.side {
        PositionSide::Long => (close_price - position.entry_price) * position.quantity,
        PositionSide::Short => (position.entry_price - close_price) * position.quantity,
    };

    // إغلاق المركز
    let closed = db::futures::close(&state.db, id, close_price, realized_pnl).await?;

    // استرجاع الهامس + الربح/الخسارة إلى المحفظة
    let credit = position.margin + realized_pnl;
    let _ = db::wallets::credit(
        &state.db,
        auth.user_id,
        &pair_meta.quote,
        credit,
        Some(id),
        "futures_close",
    )
    .await?;

    Ok(Json(json!({
        "closed": id,
        "realized_pnl": realized_pnl,
        "close_price": close_price,
        "position": closed,
    })))
}

pub async fn list_positions(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<FuturesPosition>>> {
    let positions = db::futures::list_open_for_user(&state.db, auth.user_id).await?;
    Ok(Json(positions))
}

/// مهمة خلفية: تحديث أسعار السوق للمراكز المفتوحة + فحص التصفية
pub async fn mark_price_updater(state: Arc<AppState>) {
    use std::collections::HashMap;
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
    loop {
        interval.tick().await;
        let positions = match db::futures::list_all_open(&state.db).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = ?e, "failed to load open positions");
                continue;
            }
        };

        let mut updates: Vec<(Uuid, Decimal, Decimal, Decimal)> = Vec::new();
        let mut to_liquidate: Vec<Uuid> = Vec::new();
        let pair_metas = state.trade_pairs.read().clone();

        for pos in &positions {
            let pair_meta = match pair_metas.iter().find(|p| p.pair == pos.pair) {
                Some(m) => m.clone(),
                None => continue,
            };
            let ticker = match state.binance.latest_ticker(&pair_meta.binance_symbol) {
                Some(t) => t,
                None => continue,
            };
            let mark = crate::api::trading::derive_egp_price(&state, &pair_meta.binance_symbol, ticker);

            // حساب PnL غير المحقق
            let unrealized = match pos.side {
                PositionSide::Long => (mark - pos.entry_price) * pos.quantity,
                PositionSide::Short => (pos.entry_price - mark) * pos.quantity,
            };

            // إعادة حساب سعر التصفية
            let maintenance_margin_rate = dec!(0.005);
            let liq = match pos.side {
                PositionSide::Long => {
                    mark * (Decimal::from(1) - Decimal::from(1) / Decimal::from(pos.leverage) + maintenance_margin_rate)
                }
                PositionSide::Short => {
                    mark * (Decimal::from(1) + Decimal::from(1) / Decimal::from(pos.leverage) - maintenance_margin_rate)
                }
            };

            updates.push((pos.id, mark, liq, unrealized));

            // فحص التصفية
            let should_liquidate = match pos.side {
                PositionSide::Long => mark <= pos.liquidation_price,
                PositionSide::Short => mark >= pos.liquidation_price,
            };
            if should_liquidate {
                to_liquidate.push(pos.id);
            }
        }

        if !updates.is_empty() {
            if let Err(e) = db::futures::update_mark_prices(&state.db, &updates).await {
                tracing::warn!(error = ?e, "failed to update mark prices");
            }
        }

        // تصفية المراكز
        for pos_id in &to_liquidate {
            let pos = match db::futures::get(&state.db, *pos_id).await {
                Ok(p) => p,
                Err(_) => continue,
            };
            let realized = -pos.margin; // خسارة كاملة للهامش
            if let Err(e) = db::futures::liquidate(&state.db, *pos_id, pos.mark_price, realized, Decimal::ZERO).await {
                tracing::warn!(error = ?e, "failed to liquidate position {}", pos_id);
            } else {
                tracing::info!(position_id = %pos_id, "position liquidated");
            }
        }

        // بث تحديثات المراكز عبر WebSocket
        let _ = state.engine_bcast.send(crate::matching_engine::EngineEvent::BookUpdate {
            pair: "futures_update".into(),
        });
    }
}
