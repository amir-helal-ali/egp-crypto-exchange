//! WebSocket endpoints.
//!
//! Single multiplexed endpoint `/api/market/ws` that accepts a JWT via query
//! string (`?token=...`) and streams:
//!   - order book snapshots + deltas (public)
//!   - public trades (public)
//!   - ticker updates (public)
//!   - circuit breaker state changes (public)
//!   - order_update, wallet_update, manual_tx_update, position_update,
//!     p2p_offer_update, p2p_trade_update (auth only — per-user via ws_bus)

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth::TokenType;
use crate::binance::BinanceClient;
use crate::error::AppResult;
use crate::matching_engine::EngineEvent;
use crate::models::TickerUpdate;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct WsAuth {
    pub token: Option<String>,
}

pub async fn market_ws_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<WsAuth>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let auth_user = match q.token.as_deref() {
        Some(token) => match state.jwt.verify(token) {
            Ok(claims) if claims.typ == TokenType::Access => Some(claims),
            _ => None,
        },
        None => None,
    };

    let user_id = auth_user.as_ref().and_then(|c| Uuid::parse_str(&c.sub).ok());

    ws.on_upgrade(move |socket| market_ws_loop(socket, state, user_id))
}

async fn market_ws_loop(socket: WebSocket, state: Arc<AppState>, user_id: Option<Uuid>) {
    let (mut tx, mut rx) = socket.split();

    // Send initial hello.
    let _ = tx
        .send(Message::Text(
            json!({
                "type": "hello",
                "circuit_open": state.binance.breaker.is_open(),
                "pairs": state.trade_pairs.read().iter().map(|p| p.pair.clone()).collect::<Vec<_>>(),
                "authenticated": user_id.is_some(),
            })
            .to_string(),
        ))
        .await;

    // Subscribe to public broadcast channels.
    let mut ticker_rx = state.binance.subscribe_tickers();
    let mut engine_rx = state.engine_bcast.subscribe();
    let mut breaker_rx = state.binance.breaker.subscribe();

    // Subscribe to per-user events (if authenticated).
    let mut user_rx = if let Some(uid) = user_id {
        Some(state.ws_bus.register(uid))
    } else {
        None
    };

    loop {
        tokio::select! {
            // --- أحداث عامة (tickers) ---
            recv = ticker_rx.recv() => {
                if let Ok(t) = recv {
                    let pairs = state.trade_pairs.read().clone();
                    for p in &pairs {
                        if p.binance_symbol == t.binance_symbol {
                            let egp_price = crate::api::trading::derive_egp_price(&state, &p.binance_symbol, t.clone());
                            let msg = json!({
                                "type": "ticker",
                                "pair": p.pair,
                                "binance_symbol": t.binance_symbol,
                                "bid": t.bid,
                                "ask": t.ask,
                                "derived_egp_price": egp_price,
                                "ts": t.ts,
                            });
                            if tx.send(Message::Text(msg.to_string())).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            // --- أحداث المحرك (صفقات + تحديثات الدفتر) ---
            recv = engine_rx.recv() => {
                if let Ok(ev) = recv {
                    let msg = match ev {
                        EngineEvent::Trade(t) => json!({
                            "type": "trade",
                            "pair": t.pair,
                            "price": t.price,
                            "quantity": t.quantity,
                            "taker_side": t.taker_side,
                            "ts": t.executed_at,
                        }),
                        EngineEvent::BookUpdate { pair } => {
                            let snap = state.engine.snapshot(&pair, 20);
                            json!({
                                "type": "orderbook",
                                "pair": pair,
                                "snapshot": snap,
                            })
                        }
                        EngineEvent::OrderCancelled { pair, order_id } => json!({
                            "type": "order_cancelled",
                            "pair": pair,
                            "order_id": order_id,
                        }),
                    };
                    if tx.send(Message::Text(msg.to_string())).await.is_err() {
                        break;
                    }
                }
            }
            // --- تغييرات قاطع الدائرة ---
            recv = breaker_rx.recv() => {
                if let Ok(state_change) = recv {
                    let msg = json!({
                        "type": "circuit_breaker",
                        "open": state_change == crate::binance::BreakerState::Open,
                    });
                    if tx.send(Message::Text(msg.to_string())).await.is_err() {
                        break;
                    }
                }
            }
            // --- أحداث المستخدم الخاصة (order_update, wallet_update, ...) ---
            recv = async {
                if let Some(rx) = user_rx.as_mut() {
                    rx.recv().await
                } else {
                    // لا توجد قناة مستخدم — ننتظر للأبد
                    std::future::pending::<Option<serde_json::Value>>().await
                }
            } => {
                if let Some(msg) = recv {
                    if tx.send(Message::Text(msg.to_string())).await.is_err() {
                        break;
                    }
                }
            }
            // --- رسائل من العميل ---
            msg = rx.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        if t == "ping" {
                            let _ = tx.send(Message::Text("pong".into())).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    // Cleanup: unregister from ws_bus
    if let Some(uid) = user_id {
        // إنشاء receiver وهمي للتوافق مع API — في الواقع نحتاج فقط لإزالة senders المغلقة
        let (_, dummy_rx) = tokio::sync::mpsc::unbounded_channel::<serde_json::Value>();
        state.ws_bus.unregister(uid, &dummy_rx);
    }
}

#[allow(dead_code)]
async fn _unused(_: broadcast::Receiver<TickerUpdate>, _: &BinanceClient) -> AppResult<()> {
    Ok(())
}
