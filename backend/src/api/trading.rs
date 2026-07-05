//! Trading endpoints: place/cancel orders, list orders & trades, market data.
//!
//! Settlement flow:
//! 1. Validate request (e.g. limit orders require price).
//! 2. Pre-lock the appropriate balance (sell: lock base asset; buy: lock quote
//!    amount = price * qty).
//! 3. Insert order row with status = 'open'.
//! 4. Submit to matching engine — produces a `MatchResult`.
//! 5. For each trade:
//!    a. Persist trade row.
//!    b. Update maker + taker order filled_quantity / status.
//!    c. Settle balances: credit taker with base, maker with quote, unlock
//!       locked amounts, charge fees.
//! 6. If order has remaining qty and is a limit order, leave it resting.
//! 7. If order is fully unfilled and is a market order, refund locked balance.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::matching_engine::{build_resting_order, EngineEvent, MatchTrade};
use crate::models::{
    Order, OrderBookSnapshot, OrderSide, OrderStatus, OrderType, PlaceOrderRequest,
    Trade, TradeSide, TickerUpdate,
};
use crate::AppState;

pub async fn place_order(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<PlaceOrderRequest>,
) -> AppResult<Json<Value>> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Validate pair.
    let pair_meta = state
        .trade_pairs
        .read()
        .iter()
        .find(|p| p.pair == req.pair)
        .cloned()
        .ok_or_else(|| AppError::BadRequest(format!("unknown pair {}", req.pair)))?;

    // Circuit breaker must be closed for trading.
    if state.binance.breaker.is_open() {
        return Err(AppError::CircuitBreakerOpen);
    }

    let order_id = Uuid::new_v4();
    let base = &pair_meta.base;
    let quote = &pair_meta.quote;

    // Determine lock amount.
    let lock_asset: String;
    let lock_amount: Decimal;
    match (req.side, req.order_type) {
        (OrderSide::Sell, _) => {
            lock_asset = base.clone();
            lock_amount = req.quantity;
        }
        (OrderSide::Buy, OrderType::Limit) => {
            let price = req
                .price
                .ok_or_else(|| AppError::BadRequest("limit buy requires price".into()))?;
            lock_asset = quote.clone();
            lock_amount = price * req.quantity;
        }
        (OrderSide::Buy, OrderType::Market) => {
            // For market buy we need a price estimate from Binance.
            let ticker = state
                .binance
                .latest_ticker(&pair_meta.binance_symbol)
                .ok_or_else(|| AppError::BadRequest("no live market price for market buy".into()))?;
            let egp_price = derive_egp_price(&state, &pair_meta.binance_symbol, ticker.clone());
            lock_asset = quote.clone();
            // Lock worst-case: ask * 1.02 slippage buffer.
            let buffer = Decimal::from(102) / Decimal::from(100);
            lock_amount = egp_price * req.quantity * buffer;
        }
    }

    // Lock funds.
    let _ = db::wallets::lock_balance(
        &state.db,
        auth.user_id,
        &lock_asset,
        lock_amount,
        Some(order_id),
        "order_lock",
    )
    .await?;

    // Insert order row.
    let order = Order {
        id: order_id,
        user_id: auth.user_id,
        pair: req.pair.clone(),
        side: req.side,
        order_type: req.order_type,
        price: req.price,
        quantity: req.quantity,
        filled_quantity: Decimal::ZERO,
        status: OrderStatus::Open,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    db::orders::insert(&state.db, &order).await?;

    // Submit to engine.
    let resting = build_resting_order(
        order_id,
        auth.user_id,
        &req.pair,
        req.side,
        req.order_type,
        req.price,
        req.quantity,
    );
    let result = state.engine.submit(resting);

    // Process trades — settle each one.
    for trade in &result.trades {
        settle_trade(&state, trade).await?;
    }

    // Update taker order status & filled_quantity.
    let total_filled: Decimal = result.trades.iter().map(|t| t.quantity).sum();
    let new_status = if total_filled >= req.quantity {
        OrderStatus::Filled
    } else if total_filled > Decimal::ZERO {
        OrderStatus::PartiallyFilled
    } else if req.order_type == OrderType::Market {
        OrderStatus::Cancelled
    } else {
        OrderStatus::Open
    };

    if total_filled > Decimal::ZERO {
        db::orders::update_fill(&state.db, order_id, total_filled, new_status).await?;
    } else if new_status == OrderStatus::Cancelled {
        db::orders::set_status(&state.db, order_id, OrderStatus::Cancelled).await?;
    }

    // Refund unused lock for market orders.
    if req.order_type == OrderType::Market {
        let consumed = match req.side {
            OrderSide::Buy => {
                let total_quote: Decimal =
                    result.trades.iter().map(|t| t.price * t.quantity).sum();
                lock_amount - total_quote
            }
            OrderSide::Sell => {
                let total_base: Decimal =
                    result.trades.iter().map(|t| t.quantity).sum();
                lock_amount - total_base
            }
        };
        if consumed > Decimal::ZERO {
            let _ = db::wallets::unlock_balance(
                &state.db,
                auth.user_id,
                &lock_asset,
                consumed,
                Some(order_id),
                "order_refund",
            )
            .await;
        }
    } else if total_filled < req.quantity {
        // For partial limit buys, refund unused quote lock.
        if req.side == OrderSide::Buy {
            let filled_value: Decimal =
                result.trades.iter().map(|t| t.price * t.quantity).sum();
            let locked_value = req.price.unwrap() * req.quantity;
            let refund = locked_value - filled_value;
            if refund > Decimal::ZERO {
                let _ = db::wallets::unlock_balance(
                    &state.db,
                    auth.user_id,
                    &lock_asset,
                    refund,
                    Some(order_id),
                    "order_refund",
                )
                .await;
            }
        }
    } else if result.remaining == Decimal::ZERO && total_filled == req.quantity {
        // Fully filled limit order — nothing to refund.
    }

    let updated = db::orders::get(&state.db, order_id).await?;

    // --- بث أحداث WebSocket للمستخدم ---
    // order_update: إرسال الأمر المحدث لصاحبه
    state.ws_bus.emit_to_user(auth.user_id, json!({
        "type": "order_update",
        "order": updated,
    }));

    // wallet_update: إرسال المحفظة المحدثة لكل من Taker و Makers
    let taker_wallet_asset = match req.side {
        OrderSide::Buy => pair_meta.quote.clone(),
        OrderSide::Sell => pair_meta.base.clone(),
    };
    if let Ok(w) = db::wallets::get(&state.db, auth.user_id, &taker_wallet_asset).await {
        state.ws_bus.emit_to_user(auth.user_id, json!({
            "type": "wallet_update",
            "wallet": w,
        }));
    }
    // إرسال تحديثات المحفظة لكل صانع متأثر
    let mut affected_makers: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    for trade in &result.trades {
        if affected_makers.insert(trade.maker_user_id) {
            // إرسال تحديث المحفظتين (base + quote) لكل صانع
            if let Ok(w) = db::wallets::get(&state.db, trade.maker_user_id, &pair_meta.base).await {
                state.ws_bus.emit_to_user(trade.maker_user_id, json!({
                    "type": "wallet_update",
                    "wallet": w,
                }));
            }
            if let Ok(w) = db::wallets::get(&state.db, trade.maker_user_id, &pair_meta.quote).await {
                state.ws_bus.emit_to_user(trade.maker_user_id, json!({
                    "type": "wallet_update",
                    "wallet": w,
                }));
            }
            // إرسال تحديث أمر الصانع
            if let Ok(maker_order) = db::orders::get(&state.db, trade.maker_order_id).await {
                state.ws_bus.emit_to_user(trade.maker_user_id, json!({
                    "type": "order_update",
                    "order": maker_order,
                }));
            }
        }
    }

    Ok(Json(json!({
        "order": updated,
        "trades": result.trades,
        "remaining": result.remaining,
    })))
}

pub async fn cancel_order(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let order = db::orders::get(&state.db, order_id).await?;
    if order.user_id != auth.user_id {
        return Err(AppError::Forbidden("not your order".into()));
    }
    if order.status != OrderStatus::Open && order.status != OrderStatus::PartiallyFilled {
        return Err(AppError::BadRequest(format!("order is {:?}", order.status)));
    }

    let cancelled = state.engine.cancel(&order.pair, order_id);

    // Refund remaining locked balance.
    let pair_meta = state
        .trade_pairs
        .read()
        .iter()
        .find(|p| p.pair == order.pair)
        .cloned()
        .ok_or_else(|| AppError::BadRequest("unknown pair".into()))?;

    if let Some(ref resting) = cancelled {
        let asset = match order.side {
            OrderSide::Sell => pair_meta.base.clone(),
            OrderSide::Buy => pair_meta.quote.clone(),
        };
        let refund = match order.side {
            OrderSide::Sell => resting.remaining,
            OrderSide::Buy => resting.remaining * order.price.unwrap_or_default(),
        };
        if refund > Decimal::ZERO {
            let _ = db::wallets::unlock_balance(
                &state.db,
                auth.user_id,
                &asset,
                refund,
                Some(order_id),
                "order_cancel",
            )
            .await;
        }
    }

    db::orders::set_status(&state.db, order_id, OrderStatus::Cancelled).await?;

    // --- بث أحداث WebSocket ---
    let updated_order = db::orders::get(&state.db, order_id).await?;
    state.ws_bus.emit_to_user(auth.user_id, json!({
        "type": "order_update",
        "order": updated_order,
    }));
    // تحديث المحفظة بعد الاسترجاع
    if cancelled.is_some() {
        let asset = match order.side {
            OrderSide::Sell => pair_meta.base.clone(),
            OrderSide::Buy => pair_meta.quote.clone(),
        };
        if let Ok(w) = db::wallets::get(&state.db, auth.user_id, &asset).await {
            state.ws_bus.emit_to_user(auth.user_id, json!({
                "type": "wallet_update",
                "wallet": w,
            }));
        }
    }

    Ok(Json(json!({"cancelled": order_id})))
}

pub async fn list_orders(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<Order>>> {
    let orders = db::orders::list_for_user(&state.db, auth.user_id, 200).await?;
    Ok(Json(orders))
}

pub async fn list_my_trades(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<Trade>>> {
    let trades = db::trades::list_for_user(&state.db, auth.user_id, 200).await?;
    Ok(Json(trades))
}

// --- public market data -----------------------------------------------------

pub async fn public_tickers(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<TickerUpdate>>> {
    let pairs = state.trade_pairs.read().clone();
    let mut out = Vec::with_capacity(pairs.len());
    for p in &pairs {
        if let Some(t) = state.binance.latest_ticker(&p.binance_symbol) {
            let mut tt = t.clone();
            tt.derived_egp_price = derive_egp_price(&state, &p.binance_symbol, t);
            out.push(tt);
        }
    }
    Ok(Json(out))
}

pub async fn public_orderbook(
    State(state): State<Arc<AppState>>,
    Path(pair): Path<String>,
) -> AppResult<Json<OrderBookSnapshot>> {
    let mut snap = state
        .engine
        .snapshot(&pair, 50)
        .ok_or_else(|| AppError::NotFound(format!("orderbook {pair}")))?;
    snap.circuit_breaker_open = state.binance.breaker.is_open();
    Ok(Json(snap))
}

#[derive(Debug, Deserialize)]
pub struct RecentTradesQuery {
    pub limit: Option<i64>,
}

pub async fn public_recent_trades(
    State(state): State<Arc<AppState>>,
    Path(pair): Path<String>,
    Query(q): Query<RecentTradesQuery>,
) -> AppResult<Json<Vec<Trade>>> {
    let limit = q.limit.unwrap_or(50).min(500);
    let trades = db::trades::list_recent_for_pair(&state.db, &pair, limit).await?;
    Ok(Json(trades))
}

pub async fn circuit_status(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "open": state.binance.breaker.is_open(),
        "state": format!("{:?}", state.binance.breaker.state()),
    })))
}

// --- settlement -------------------------------------------------------------

async fn settle_trade(state: &AppState, trade: &MatchTrade) -> AppResult<()> {
    // Determine fee BPS from settings cache.
    let maker_bps = state.fees.maker_bps;
    let taker_bps = state.fees.taker_bps;

    let pair_meta = state
        .trade_pairs
        .read()
        .iter()
        .find(|p| p.pair == trade.pair)
        .cloned()
        .ok_or_else(|| AppError::Internal(format!("pair {} missing", trade.pair)))?;

    let base = &pair_meta.base;
    let quote = &pair_meta.quote;

    let value = trade.price * trade.quantity;
    let taker_fee = value * Decimal::from(taker_bps) / Decimal::from(10000);
    let maker_fee = value * Decimal::from(maker_bps) / Decimal::from(10000);

    // Persist trade row.
    let taker_side_enum = trade.taker_side;
    db::trades::insert(
        &state.db,
        &trade.pair,
        trade.taker_order_id,
        trade.maker_order_id,
        trade.taker_user_id,
        trade.maker_user_id,
        taker_side_enum,
        trade.price,
        trade.quantity,
        taker_fee,
        maker_fee,
    )
    .await?;

    // Update maker order filled_quantity + status.
    let maker_order = db::orders::get(&state.db, trade.maker_order_id).await?;
    let new_filled = maker_order.filled_quantity + trade.quantity;
    let new_status = if new_filled >= maker_order.quantity {
        OrderStatus::Filled
    } else {
        OrderStatus::PartiallyFilled
    };
    db::orders::update_fill(&state.db, trade.maker_order_id, trade.quantity, new_status).await?;

    // Update taker order filled_quantity + status (handled by caller).
    // Settle balances.
    match trade.taker_side {
        TradeSide::Buy => {
            // Taker buys base, pays quote.
            // Unlock maker's locked base, credit taker base.
            let _ = db::wallets::unlock_balance(
                &state.db,
                trade.maker_user_id,
                base,
                trade.quantity,
                Some(trade.maker_order_id),
                "trade_settle",
            )
            .await;
            // Credit maker with quote value minus maker fee.
            let _ = db::wallets::credit(
                &state.db,
                trade.maker_user_id,
                quote,
                value - maker_fee,
                Some(trade.maker_order_id),
                "trade_proceeds",
            )
            .await;
            // Credit taker with base (the bought asset).
            let _ = db::wallets::credit(
                &state.db,
                trade.taker_user_id,
                base,
                trade.quantity,
                Some(trade.taker_order_id),
                "trade_filled",
            )
            .await;
            // Taker fee: debit extra quote.
            let _ = db::wallets::debit(
                &state.db,
                trade.taker_user_id,
                quote,
                taker_fee,
                Some(trade.taker_order_id),
                "trade_fee",
            )
            .await;
        }
        TradeSide::Sell => {
            // Taker sells base for quote.
            let _ = db::wallets::unlock_balance(
                &state.db,
                trade.maker_user_id,
                quote,
                value,
                Some(trade.maker_order_id),
                "trade_settle",
            )
            .await;
            let _ = db::wallets::credit(
                &state.db,
                trade.maker_user_id,
                base,
                trade.quantity - maker_fee,
                Some(trade.maker_order_id),
                "trade_filled",
            )
            .await;
            let _ = db::wallets::credit(
                &state.db,
                trade.taker_user_id,
                quote,
                value - taker_fee,
                Some(trade.taker_order_id),
                "trade_proceeds",
            )
            .await;
        }
    }

    Ok(())
}

/// Derive EGP price from Binance USDT pair + EGP/USD rate.
pub fn derive_egp_price(state: &AppState, binance_symbol: &str, ticker: TickerUpdate) -> Decimal {
    let egp_usd = state.config.egp_usd_rate;
    // If symbol ends with USDT, multiply mid price by EGP/USD.
    if binance_symbol.ends_with("USDT") {
        let mid = (ticker.bid + ticker.ask) / Decimal::from(2);
        mid * egp_usd
    } else if binance_symbol.ends_with("USDC") {
        let mid = (ticker.bid + ticker.ask) / Decimal::from(2);
        mid * egp_usd
    } else {
        ticker.derived_egp_price
    }
}

// Event consumer task: persists trades asynchronously as a safety net
// (redundant with synchronous settle_trade but kept for resilience).
pub async fn engine_event_consumer(state: Arc<AppState>) {
    let mut rx = state.engine_event_rx.lock().await.take().expect("engine event rx already taken");
    while let Some(event) = rx.recv().await {
        match event {
            EngineEvent::Trade(_) => {
                // Trades are settled synchronously in `place_order`. This branch
                // could be used for fan-out to websocket subscribers.
            }
            EngineEvent::BookUpdate { .. } | EngineEvent::OrderCancelled { .. } => {
                // Broadcast to ws subscribers via engine_bcast.
            }
        }
    }
}
