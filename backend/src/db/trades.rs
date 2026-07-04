//! `trades` table queries.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Trade, TradeSide};

pub async fn insert(
    pool: &PgPool,
    pair: &str,
    taker_order_id: Uuid,
    maker_order_id: Uuid,
    taker_user_id: Uuid,
    maker_user_id: Uuid,
    taker_side: TradeSide,
    price: Decimal,
    quantity: Decimal,
    taker_fee: Decimal,
    maker_fee: Decimal,
) -> AppResult<Trade> {
    let trade = sqlx::query_as::<_, Trade>(
        r#"
        INSERT INTO trades
          (pair, taker_order_id, maker_order_id, taker_user_id, maker_user_id,
           taker_side, price, quantity, taker_fee, maker_fee)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(pair)
    .bind(taker_order_id)
    .bind(maker_order_id)
    .bind(taker_user_id)
    .bind(maker_user_id)
    .bind(taker_side)
    .bind(price)
    .bind(quantity)
    .bind(taker_fee)
    .bind(maker_fee)
    .fetch_one(pool)
    .await?;
    Ok(trade)
}

pub async fn list_recent_for_pair(
    pool: &PgPool,
    pair: &str,
    limit: i64,
) -> AppResult<Vec<Trade>> {
    let trades = sqlx::query_as::<_, Trade>(
        r#"
        SELECT * FROM trades
        WHERE pair = $1
        ORDER BY executed_at DESC
        LIMIT $2
        "#,
    )
    .bind(pair)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(trades)
}

pub async fn list_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<Trade>> {
    let trades = sqlx::query_as::<_, Trade>(
        r#"
        SELECT * FROM trades
        WHERE taker_user_id = $1 OR maker_user_id = $1
        ORDER BY executed_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(trades)
}
