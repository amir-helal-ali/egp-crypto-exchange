//! In-memory Order Matching Engine.
//!
//! Architecture:
//! - One `OrderBook` per trading pair, stored inside an `Engine` (DashMap).
//! - Each side uses a `BTreeMap<PriceKey, VecDeque<Order>>` for O(log n) insertion,
//!   O(1) best-price lookup, and FIFO execution at each price level.
//! - `PriceKey` is a wrapper around `Decimal` that inverts `Ord` for the **bids** side
//!   so that highest prices sort first (best bid = first entry).
//! - Trades are produced synchronously; persistence to PostgreSQL is delegated to
//!   the caller (the engine itself is pure in-memory).
//!
//! Concurrency model:
//! - Each `OrderBook` is guarded by a `parking_lot::RwLock`.
//! - Inserts/cancels take a write lock; reads (snapshot) take a read lock.
//! - Trade events are emitted through an `mpsc` channel that the persistence
//!   layer consumes asynchronously.

use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use chrono::Utc;
use dashmap::DashMap;
use parking_lot::RwLock;
use rust_decimal::Decimal;
use serde::Serialize;
use tokio::sync::{mpsc, broadcast};
use uuid::Uuid;

use crate::models::{
    OrderBookLevel, OrderSide, OrderType, TradeSide,
};

/// A price level key. For the asks side we use natural ascending order
/// (lowest ask first); for the bids side we invert via `Reverse` wrapper
/// in the `OrderBook` to get highest bid first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PriceKey(pub Decimal);

/// Internal representation of a resting order.
#[derive(Debug, Clone, Serialize)]
pub struct RestingOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Decimal,
    /// Remaining quantity (after partial fills).
    pub remaining: Decimal,
    pub created_at: chrono::DateTime<Utc>,
}

/// One side of the book (bids or asks).
/// `ASCENDING = true` means natural order (asks: lowest first).
/// `ASCENDING = false` means reversed (bids: highest first).
#[derive(Default)]
pub struct BookSide<const ASCENDING: bool> {
    pub levels: BTreeMap<PriceKey, VecDeque<RestingOrder>>,
}

impl<const ASCENDING: bool> BookSide<ASCENDING> {
    pub fn insert(&mut self, order: RestingOrder) {
        let key = PriceKey(order.price);
        self.levels.entry(key).or_default().push_back(order);
    }

    pub fn best_key(&self) -> Option<PriceKey> {
        if ASCENDING {
            self.levels.keys().next().copied()
        } else {
            self.levels.keys().next_back().copied()
        }
    }

    pub fn peek_best(&self) -> Option<&RestingOrder> {
        let key = self.best_key()?;
        self.levels.get(&key).and_then(|q| q.front())
    }

    pub fn pop_best(&mut self) -> Option<RestingOrder> {
        let key = self.best_key()?;
        let queue = self.levels.get_mut(&key)?;
        let order = queue.pop_front()?;
        if queue.is_empty() {
            self.levels.remove(&key);
        }
        Some(order)
    }

    pub fn reduce_best(&mut self, taken: Decimal) {
        if let Some(key) = self.best_key() {
            if let Some(queue) = self.levels.get_mut(&key) {
                if let Some(front) = queue.front_mut() {
                    front.remaining -= taken;
                }
            }
        }
    }

    pub fn cancel(&mut self, order_id: Uuid) -> Option<RestingOrder> {
        for (_key, queue) in self.levels.iter_mut() {
            if let Some(pos) = queue.iter().position(|o| o.id == order_id) {
                return Some(queue.remove(pos).unwrap());
            }
        }
        None
    }

    pub fn snapshot(&self, depth: usize) -> Vec<OrderBookLevel> {
        let iter: Box<dyn Iterator<Item = (&PriceKey, &VecDeque<RestingOrder>)>> = if ASCENDING {
            Box::new(self.levels.iter())
        } else {
            Box::new(self.levels.iter().rev())
        };
        iter.take(depth)
            .map(|(k, q)| {
                let total: Decimal = q.iter().map(|o| o.remaining).sum();
                OrderBookLevel {
                    price: k.0,
                    quantity: total,
                }
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.levels.values().map(|q| q.len()).sum()
    }
}

/// Per-pair order book.
pub struct OrderBook {
    pub pair: String,
    pub bids: RwLock<BookSide<false>>, // descending: best bid first
    pub asks: RwLock<BookSide<true>>,  // ascending: best ask first
    pub last_trade_price: RwLock<Option<Decimal>>,
}

impl OrderBook {
    pub fn new(pair: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            pair: pair.into(),
            bids: RwLock::new(BookSide::<false>::default()),
            asks: RwLock::new(BookSide::<true>::default()),
            last_trade_price: RwLock::new(None),
        })
    }
}

/// A trade produced by the matching engine.
#[derive(Debug, Clone, Serialize)]
pub struct MatchTrade {
    pub pair: String,
    pub taker_order_id: Uuid,
    pub maker_order_id: Uuid,
    pub taker_user_id: Uuid,
    pub maker_user_id: Uuid,
    pub taker_side: TradeSide,
    pub price: Decimal,
    pub quantity: Decimal,
    pub executed_at: chrono::DateTime<Utc>,
}

/// Result of submitting an order to the engine.
#[derive(Debug, Clone, Serialize)]
pub struct MatchResult {
    pub taker_order_id: Uuid,
    pub trades: Vec<MatchTrade>,
    /// Remaining (unfilled) quantity — to be placed as a resting order.
    pub remaining: Decimal,
}

/// Engine event broadcasted to subscribers (websocket push, etc).
#[derive(Debug, Clone, Serialize)]
pub enum EngineEvent {
    Trade(MatchTrade),
    BookUpdate {
        pair: String,
    },
    OrderCancelled {
        pair: String,
        order_id: Uuid,
    },
}

pub struct Engine {
    books: DashMap<String, Arc<OrderBook>>,
    event_tx: mpsc::UnboundedSender<EngineEvent>,
    /// Broadcast for ws subscribers — cheap cloneable fan-out.
    event_bcast: broadcast::Sender<EngineEvent>,
}

impl Engine {
    pub fn new(
        event_tx: mpsc::UnboundedSender<EngineEvent>,
        event_bcast: broadcast::Sender<EngineEvent>,
    ) -> Self {
        Self {
            books: DashMap::new(),
            event_tx,
            event_bcast,
        }
    }

    pub fn ensure_book(&self, pair: &str) -> Arc<OrderBook> {
        if let Some(g) = self.books.get(pair) {
            return g.clone();
        }
        let book = OrderBook::new(pair);
        self.books.entry(pair.to_string()).or_insert_with(|| book.clone()).clone()
    }

    pub fn get_book(&self, pair: &str) -> Option<Arc<OrderBook>> {
        self.books.get(pair).map(|g| g.clone())
    }

    /// Submit a new order. The engine matches it against the opposite side
    /// and (for limit orders) places any remaining quantity as a resting order.
    pub fn submit(&self, order: RestingOrder) -> MatchResult {
        let pair = order.pair.clone();
        let book = self.ensure_book(&pair);
        let mut trades: Vec<MatchTrade> = Vec::new();
        let mut remaining = order.remaining;
        let taker_side = order.side;
        let taker_order_id = order.id;
        let taker_user_id = order.user_id;

        let (taker_side_enum, opp_book) = match order.side {
            OrderSide::Buy => (TradeSide::Buy, &book.asks),
            OrderSide::Sell => (TradeSide::Sell, &book.bids),
        };

        let mut opp = opp_book.write();

        while remaining > Decimal::ZERO {
            let best = match opp.peek_best() {
                Some(o) => o.clone(),
                None => break,
            };

            // Price check: a buy taker only matches asks <= its price (limit).
            // A sell taker only matches bids >= its price (limit).
            let price_ok = match order.order_type {
                OrderType::Market => true,
                OrderType::Limit => match order.side {
                    OrderSide::Buy => best.price <= order.price,
                    OrderSide::Sell => best.price >= order.price,
                },
            };
            if !price_ok {
                break;
            }

            let trade_qty = remaining.min(best.remaining);
            let trade_price = best.price;
            let trade = MatchTrade {
                pair: book.pair.clone(),
                taker_order_id,
                maker_order_id: best.id,
                taker_user_id,
                maker_user_id: best.user_id,
                taker_side: taker_side_enum,
                price: trade_price,
                quantity: trade_qty,
                executed_at: Utc::now(),
            };
            trades.push(trade);

            // Update remaining quantities.
            remaining -= trade_qty;
            if trade_qty >= best.remaining {
                // Maker fully consumed.
                let _ = opp.pop_best();
            } else {
                opp.reduce_best(trade_qty);
            }
        }

        // Update last trade price.
        if let Some(last) = trades.last() {
            *book.last_trade_price.write() = Some(last.price);
        }

        // For limit orders with remaining qty, place as resting order.
        if order.order_type == OrderType::Limit && remaining > Decimal::ZERO {
            let mut resting = order.clone();
            resting.remaining = remaining;
            match resting.side {
                OrderSide::Buy => book.bids.write().insert(resting),
                OrderSide::Sell => book.asks.write().insert(resting),
            }
        }

        // Emit events.
        for t in &trades {
            let _ = self.event_tx.send(EngineEvent::Trade(t.clone()));
        }
        let _ = self.event_bcast.send(EngineEvent::BookUpdate {
            pair: book.pair.clone(),
        });

        MatchResult {
            taker_order_id,
            trades,
            remaining: if order.order_type == OrderType::Limit {
                remaining
            } else {
                Decimal::ZERO
            },
        }
    }

    pub fn cancel(&self, pair: &str, order_id: Uuid) -> Option<RestingOrder> {
        let book = self.get_book(pair)?;
        let cancelled = book.bids.write().cancel(order_id).or_else(|| {
            book.asks.write().cancel(order_id)
        });
        if cancelled.is_some() {
            let _ = self.event_tx.send(EngineEvent::OrderCancelled {
                pair: pair.to_string(),
                order_id,
            });
            let _ = self.event_bcast.send(EngineEvent::BookUpdate {
                pair: pair.to_string(),
            });
        }
        cancelled
    }

    pub fn snapshot(&self, pair: &str, depth: usize) -> Option<crate::models::OrderBookSnapshot> {
        let book = self.get_book(pair)?;
        let bids = book.bids.read().snapshot(depth);
        let asks = book.asks.read().snapshot(depth);
        let last = *book.last_trade_price.read();
        Some(crate::models::OrderBookSnapshot {
            pair: pair.to_string(),
            bids,
            asks,
            last_price: last,
            circuit_breaker_open: false, // populated by caller
        })
    }
}

impl RestingOrder {
    // No additional methods needed — all fields are public.
}

/// Convenience constructor used by the API layer.
#[allow(clippy::too_many_arguments)]
pub fn build_resting_order(
    id: Uuid,
    user_id: Uuid,
    pair: &str,
    side: OrderSide,
    order_type: OrderType,
    price: Option<Decimal>,
    quantity: Decimal,
) -> RestingOrder {
    RestingOrder {
        id,
        user_id,
        pair: pair.to_string(),
        side,
        order_type,
        price: price.unwrap_or_default(),
        remaining: quantity,
        created_at: Utc::now(),
    }
}
