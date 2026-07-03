//! Service-layer helpers: trade pair metadata, fee schedule.

use parking_lot::RwLock;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TradePair {
    pub pair: String,         // e.g. "BTC_EGP"
    pub base: String,         // "BTC"
    pub quote: String,        // "EGP"
    pub binance_symbol: String, // "BTCUSDT"
}

#[derive(Debug, Clone)]
pub struct Fees {
    pub maker_bps: i32,
    pub taker_bps: i32,
    pub min_egp_deposit: Decimal,
    pub min_egp_withdrawal: Decimal,
}

impl Default for Fees {
    fn default() -> Self {
        Self {
            maker_bps: 10,
            taker_bps: 20,
            min_egp_deposit: Decimal::from(100),
            min_egp_withdrawal: Decimal::from(200),
        }
    }
}

/// Load trade pairs from the `settings` table. Falls back to a default set.
pub async fn load_trade_pairs(pool: &sqlx::PgPool) -> anyhow::Result<Vec<TradePair>> {
    let row: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT value FROM settings WHERE key = 'trade_pairs'")
            .fetch_optional(pool)
            .await?;
    if let Some((v,)) = row {
        let pairs: Vec<TradePair> = serde_json::from_value(v)?;
        return Ok(pairs);
    }
    Ok(vec![
        TradePair {
            pair: "BTC_EGP".into(),
            base: "BTC".into(),
            quote: "EGP".into(),
            binance_symbol: "BTCUSDT".into(),
        },
        TradePair {
            pair: "ETH_EGP".into(),
            base: "ETH".into(),
            quote: "EGP".into(),
            binance_symbol: "ETHUSDT".into(),
        },
        TradePair {
            pair: "USDT_EGP".into(),
            base: "USDT".into(),
            quote: "EGP".into(),
            binance_symbol: "USDTUSDT".into(),
        },
    ])
}

pub type SharedTradePairs = Arc<RwLock<Vec<TradePair>>>;
