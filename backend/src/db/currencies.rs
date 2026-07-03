//! استعلامات العملات والأزواج - Currencies and Trading Pairs queries

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Currency, CurrencyType, TradingPair};

// --- العملات ---------------------------------------------------------------

pub async fn list_currencies(pool: &PgPool, active_only: bool) -> AppResult<Vec<Currency>> {
    let rows = if active_only {
        sqlx::query_as::<_, Currency>(
            "SELECT * FROM currencies WHERE is_active = true ORDER BY type, symbol"
        )
        .fetch_all(pool).await?
    } else {
        sqlx::query_as::<_, Currency>(
            "SELECT * FROM currencies ORDER BY type, symbol"
        )
        .fetch_all(pool).await?
    };
    Ok(rows)
}

pub async fn get_currency(pool: &PgPool, symbol: &str) -> AppResult<Currency> {
    sqlx::query_as::<_, Currency>("SELECT * FROM currencies WHERE symbol = $1")
        .bind(symbol)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("currency {symbol}")))
}

pub async fn create_currency(
    pool: &PgPool,
    symbol: &str,
    name: &str,
    type_: CurrencyType,
    precision: i16,
    withdraw_fee: Decimal,
    min_withdrawal: Decimal,
    network: Option<&str>,
    is_active: bool,
) -> AppResult<Currency> {
    let row = sqlx::query_as::<_, Currency>(
        r#"
        INSERT INTO currencies (symbol, name, type, precision, withdraw_fee, min_withdrawal, network, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(symbol).bind(name).bind(type_).bind(precision)
    .bind(withdraw_fee).bind(min_withdrawal).bind(network).bind(is_active)
    .fetch_one(pool).await?;
    Ok(row)
}

pub async fn update_currency(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    precision: Option<i16>,
    withdraw_fee: Option<Decimal>,
    min_withdrawal: Option<Decimal>,
    network: Option<Option<&str>>,
    is_active: Option<bool>,
) -> AppResult<Currency> {
    let row = sqlx::query_as::<_, Currency>(
        r#"
        UPDATE currencies SET
            name = COALESCE($2, name),
            precision = COALESCE($3, precision),
            withdraw_fee = COALESCE($4, withdraw_fee),
            min_withdrawal = COALESCE($5, min_withdrawal),
            network = COALESCE($6, network),
            is_active = COALESCE($7, is_active)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(precision)
    .bind(withdraw_fee)
    .bind(min_withdrawal)
    .bind(network)
    .bind(is_active)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_currency(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM currencies WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- أزواج التداول --------------------------------------------------------

pub async fn list_pairs(pool: &PgPool, active_only: bool) -> AppResult<Vec<TradingPair>> {
    let rows = if active_only {
        sqlx::query_as::<_, TradingPair>(
            "SELECT * FROM trading_pairs WHERE is_active = true ORDER BY sort_order, pair"
        )
        .fetch_all(pool).await?
    } else {
        sqlx::query_as::<_, TradingPair>(
            "SELECT * FROM trading_pairs ORDER BY sort_order, pair"
        )
        .fetch_all(pool).await?
    };
    Ok(rows)
}

pub async fn get_pair(pool: &PgPool, pair: &str) -> AppResult<TradingPair> {
    sqlx::query_as::<_, TradingPair>("SELECT * FROM trading_pairs WHERE pair = $1")
        .bind(pair)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("pair {pair}")))
}

#[allow(clippy::too_many_arguments)]
pub async fn create_pair(
    pool: &PgPool,
    pair: &str,
    base_asset: &str,
    quote_asset: &str,
    binance_symbol: &str,
    is_spot_active: bool,
    is_futures_active: bool,
    maker_fee_bps: i32,
    taker_fee_bps: i32,
    min_order_qty: Decimal,
    price_precision: i16,
    qty_precision: i16,
    sort_order: i32,
) -> AppResult<TradingPair> {
    let row = sqlx::query_as::<_, TradingPair>(
        r#"
        INSERT INTO trading_pairs
          (pair, base_asset, quote_asset, binance_symbol, is_spot_active,
           is_futures_active, maker_fee_bps, taker_fee_bps, min_order_qty,
           price_precision, qty_precision, sort_order, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, true)
        RETURNING *
        "#,
    )
    .bind(pair).bind(base_asset).bind(quote_asset).bind(binance_symbol)
    .bind(is_spot_active).bind(is_futures_active)
    .bind(maker_fee_bps).bind(taker_fee_bps).bind(min_order_qty)
    .bind(price_precision).bind(qty_precision).bind(sort_order)
    .fetch_one(pool).await?;
    Ok(row)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_pair(
    pool: &PgPool,
    id: Uuid,
    binance_symbol: Option<&str>,
    is_spot_active: Option<bool>,
    is_futures_active: Option<bool>,
    maker_fee_bps: Option<i32>,
    taker_fee_bps: Option<i32>,
    min_order_qty: Option<Decimal>,
    price_precision: Option<i16>,
    qty_precision: Option<i16>,
    sort_order: Option<i32>,
    is_active: Option<bool>,
) -> AppResult<TradingPair> {
    let row = sqlx::query_as::<_, TradingPair>(
        r#"
        UPDATE trading_pairs SET
            binance_symbol = COALESCE($2, binance_symbol),
            is_spot_active = COALESCE($3, is_spot_active),
            is_futures_active = COALESCE($4, is_futures_active),
            maker_fee_bps = COALESCE($5, maker_fee_bps),
            taker_fee_bps = COALESCE($6, taker_fee_bps),
            min_order_qty = COALESCE($7, min_order_qty),
            price_precision = COALESCE($8, price_precision),
            qty_precision = COALESCE($9, qty_precision),
            sort_order = COALESCE($10, sort_order),
            is_active = COALESCE($11, is_active)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(binance_symbol)
    .bind(is_spot_active)
    .bind(is_futures_active)
    .bind(maker_fee_bps)
    .bind(taker_fee_bps)
    .bind(min_order_qty)
    .bind(price_precision)
    .bind(qty_precision)
    .bind(sort_order)
    .bind(is_active)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_pair(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM trading_pairs WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
