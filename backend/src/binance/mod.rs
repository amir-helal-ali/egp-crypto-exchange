//! Binance WebSocket client + circuit breaker.
//!
//! Connects to Binance combined streams (e.g. `btcusdt@bookTicker`).
//! The circuit breaker trips when:
//!   - the connection drops and cannot reconnect within `CIRCUIT_BREAKER_TIMEOUT_SECS`,
//!   - OR when N consecutive message-parsing failures occur
//!     (CIRCUIT_BREAKER_MAX_FAILURES).
//!
//! While the breaker is open:
//!   - The matching engine refuses new `submit()` calls (returns
//!     `AppError::CircuitBreakerOpen` via the trading API).
//!   - All connected users receive a `circuit_open` flag in market data
//!     snapshots.

use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::broadcast;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::models::TickerUpdate;

/// State of the circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    Closed,  // feed healthy, trading allowed
    Open,    // feed unhealthy, trading blocked
}

pub struct CircuitBreaker {
    state: RwLock<BreakerState>,
    last_message_at: RwLock<Option<Instant>>,
    consecutive_failures: RwLock<u32>,
    timeout: Duration,
    max_failures: u32,
    /// Broadcast breaker state changes.
    state_tx: broadcast::Sender<BreakerState>,
}

impl CircuitBreaker {
    pub fn new(timeout: Duration, max_failures: u32) -> Self {
        let (state_tx, _) = broadcast::channel(16);
        Self {
            state: RwLock::new(BreakerState::Open), // start open until first message
            last_message_at: RwLock::new(None),
            consecutive_failures: RwLock::new(0),
            timeout,
            max_failures,
            state_tx,
        }
    }

    pub fn state(&self) -> BreakerState {
        *self.state.read()
    }

    pub fn is_open(&self) -> bool {
        self.state() == BreakerState::Open
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BreakerState> {
        self.state_tx.subscribe()
    }

    /// Called when a valid message is received from Binance.
    pub fn record_success(&self) {
        *self.last_message_at.write() = Some(Instant::now());
        *self.consecutive_failures.write() = 0;
        let prev = *self.state.read();
        if prev != BreakerState::Closed {
            *self.state.write() = BreakerState::Closed;
            let _ = self.state_tx.send(BreakerState::Closed);
            tracing::info!("circuit breaker CLOSED — trading enabled");
        }
    }

    /// Called when a parse failure or transport error occurs.
    pub fn record_failure(&self) {
        let mut fails = self.consecutive_failures.write();
        *fails += 1;
        if *fails >= self.max_failures {
            self.trip();
        }
    }

    /// Force the breaker open (e.g. on disconnect).
    pub fn trip(&self) {
        let prev = *self.state.read();
        if prev != BreakerState::Open {
            *self.state.write() = BreakerState::Open;
            let _ = self.state_tx.send(BreakerState::Open);
            tracing::warn!("circuit breaker OPEN — trading halted");
        }
    }

    /// Background watchdog: trip if no message for `timeout`.
    pub async fn watchdog(self: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            let last = *self.last_message_at.read();
            if let Some(t) = last {
                if t.elapsed() > self.timeout {
                    self.trip();
                }
            } else {
                // No message ever received.
                self.trip();
            }
        }
    }
}

/// Binance combined-stream message envelope.
#[derive(Debug, Deserialize)]
struct CombinedEnvelope {
    stream: String,
    data: Value,
}

/// `bookTicker` payload.
#[derive(Debug, Deserialize)]
struct BookTicker {
    #[serde(rename = "u")]
    update_id: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "b")]
    bid: String,
    #[serde(rename = "B")]
    bid_qty: String,
    #[serde(rename = "a")]
    ask: String,
    #[serde(rename = "A")]
    ask_qty: String,
}

/// Top-level client.
pub struct BinanceClient {
    pub url: String,
    pub breaker: Arc<CircuitBreaker>,
    pub ticker_tx: broadcast::Sender<TickerUpdate>,
    /// Latest ticker per Binance symbol.
    pub latest: Arc<RwLock<std::collections::HashMap<String, TickerUpdate>>>,
}

impl BinanceClient {
    pub fn new(url: String, breaker: Arc<CircuitBreaker>) -> Self {
        let (ticker_tx, _) = broadcast::channel(256);
        Self {
            url,
            breaker,
            ticker_tx,
            latest: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Subscribe to live ticker updates.
    pub fn subscribe_tickers(&self) -> broadcast::Receiver<TickerUpdate> {
        self.ticker_tx.subscribe()
    }

    /// Spawn the WebSocket listener loop. Reconnects automatically with
    /// exponential backoff.
    pub async fn run(self: Arc<Self>) {
        // Spawn watchdog.
        let breaker = self.breaker.clone();
        tokio::spawn(breaker.watchdog());

        let mut backoff = Duration::from_secs(1);
        let max_backoff = Duration::from_secs(30);

        loop {
            tracing::info!(url = %self.url, "connecting to Binance WS");
            match connect_async(&self.url).await {
                Ok((ws, _)) => {
                    backoff = Duration::from_secs(1); // reset
                    tracing::info!("Binance WS connected");
                    let (mut write, mut read) = ws.split();

                    // Send periodic ping to keep alive.
                    let ping_tx = self.clone();
                    let ping_task = tokio::spawn(async move {
                        let mut ticker = tokio::time::interval(Duration::from_secs(20));
                        loop {
                            ticker.tick().await;
                            if write.send(Message::Ping(vec![1, 2, 3, 4])).await.is_err() {
                                break;
                            }
                            let _ = &ping_tx; // keep ref
                        }
                    });

                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(txt)) => {
                                self.handle_text(&txt);
                            }
                            Ok(Message::Binary(b)) => {
                                if let Ok(txt) = std::str::from_utf8(&b) {
                                    self.handle_text(txt);
                                }
                            }
                            Ok(Message::Ping(p)) => {
                                let _ = p; // tungstenite auto-pongs
                            }
                            Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {}
                            Ok(Message::Close(_)) => {
                                tracing::warn!("Binance WS closed by peer");
                                break;
                            }
                            Err(e) => {
                                tracing::error!(error = ?e, "Binance WS error");
                                self.breaker.record_failure();
                                break;
                            }
                        }
                    }
                    ping_task.abort();
                }
                Err(e) => {
                    tracing::error!(error = ?e, "failed to connect to Binance WS");
                    self.breaker.record_failure();
                }
            }

            // Disconnected — backoff before reconnect.
            tracing::warn!(backoff_ms = backoff.as_millis(), "reconnecting after backoff");
            self.breaker.trip();
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(max_backoff);
        }
    }

    fn handle_text(&self, txt: &str) {
        let env: CombinedEnvelope = match serde_json::from_str(txt) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "failed to parse Binance message");
                self.breaker.record_failure();
                return;
            }
        };

        // Try bookTicker payload.
        if let Ok(bt) = serde_json::from_value::<BookTicker>(env.data) {
            let bid = match bt.bid.parse::<Decimal>() {
                Ok(v) => v,
                Err(_) => {
                    self.breaker.record_failure();
                    return;
                }
            };
            let ask = match bt.ask.parse::<Decimal>() {
                Ok(v) => v,
                Err(_) => {
                    self.breaker.record_failure();
                    return;
                }
            };
            let mid = (bid + ask) / Decimal::from(2);
            let update = TickerUpdate {
                binance_symbol: bt.symbol,
                bid,
                ask,
                derived_egp_price: mid, // direct EGP rate computed downstream
                ts: chrono::Utc::now(),
            };
            self.latest.write().insert(update.binance_symbol.clone(), update.clone());
            let _ = self.ticker_tx.send(update);
            self.breaker.record_success();
        } else {
            // Other stream types (e.g. trade) — record success but ignore.
            self.breaker.record_success();
        }
    }

    /// Get latest cached ticker by Binance symbol.
    pub fn latest_ticker(&self, symbol: &str) -> Option<TickerUpdate> {
        self.latest.read().get(symbol).cloned()
    }
}
