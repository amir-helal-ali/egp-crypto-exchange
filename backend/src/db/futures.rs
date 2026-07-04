//! استعلامات العقود الآجلة - Futures positions queries

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{FuturesPosition, MarginMode, PositionSide, PositionStatus};

pub async fn open(
    pool: &PgPool,
    user_id: Uuid,
    pair: &str,
    side: PositionSide,
    margin_mode: MarginMode,
    leverage: i32,
    margin: Decimal,
    quantity: Decimal,
    entry_price: Decimal,
    mark_price: Decimal,
    liquidation_price: Decimal,
) -> AppResult<FuturesPosition> {
    let pos = sqlx::query_as::<_, FuturesPosition>(
        r#"
        INSERT INTO futures_positions
          (user_id, pair, side, margin_mode, leverage, margin, quantity,
           entry_price, mark_price, liquidation_price, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'open')
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(pair)
    .bind(side)
    .bind(margin_mode)
    .bind(leverage)
    .bind(margin)
    .bind(quantity)
    .bind(entry_price)
    .bind(mark_price)
    .bind(liquidation_price)
    .fetch_one(pool)
    .await?;
    Ok(pos)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<FuturesPosition> {
    sqlx::query_as::<_, FuturesPosition>("SELECT * FROM futures_positions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("position {id}")))
}

pub async fn list_open_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<FuturesPosition>> {
    let rows = sqlx::query_as::<_, FuturesPosition>(
        r#"SELECT * FROM futures_positions
           WHERE user_id = $1 AND status = 'open'
           ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_open_by_pair(pool: &PgPool, pair: &str) -> AppResult<Vec<FuturesPosition>> {
    let rows = sqlx::query_as::<_, FuturesPosition>(
        r#"SELECT * FROM futures_positions
           WHERE pair = $1 AND status = 'open'
           ORDER BY created_at DESC"#,
    )
    .bind(pair)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_all_open(pool: &PgPool) -> AppResult<Vec<FuturesPosition>> {
    let rows = sqlx::query_as::<_, FuturesPosition>(
        r#"SELECT * FROM futures_positions
           WHERE status = 'open'
           ORDER BY created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn close(
    pool: &PgPool,
    id: Uuid,
    close_price: Decimal,
    realized_pnl: Decimal,
) -> AppResult<FuturesPosition> {
    let pos = sqlx::query_as::<_, FuturesPosition>(
        r#"
        UPDATE futures_positions
        SET status = 'closed',
            close_price = $2,
            realized_pnl = $3,
            closed_at = $4
        WHERE id = $1 AND status = 'open'
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(close_price)
    .bind(realized_pnl)
    .bind(Utc::now())
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| crate::error::AppError::BadRequest("position not open".into()))?;
    Ok(pos)
}

pub async fn liquidate(
    pool: &PgPool,
    id: Uuid,
    mark_price: Decimal,
    realized_pnl: Decimal,
    fee: Decimal,
) -> AppResult<FuturesPosition> {
    let pos = sqlx::query_as::<_, FuturesPosition>(
        r#"
        UPDATE futures_positions
        SET status = 'liquidated',
            close_price = $2,
            mark_price = $2,
            realized_pnl = $3,
            closed_at = $4
        WHERE id = $1 AND status = 'open'
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(mark_price)
    .bind(realized_pnl)
    .bind(Utc::now())
    .fetch_optional(pool)
    .await?;

    if let Some(ref p) = pos {
        sqlx::query(
            r#"INSERT INTO liquidations (position_id, user_id, pair, liquidation_price, mark_price, realized_pnl, fee)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(id)
        .bind(p.user_id)
        .bind(&p.pair)
        .bind(p.liquidation_price)
        .bind(mark_price)
        .bind(realized_pnl)
        .bind(fee)
        .execute(pool)
        .await?;
    }

    pos.ok_or_else(|| crate::error::AppError::BadRequest("position not open".into()))
}

pub async fn update_mark_prices(
    pool: &PgPool,
    updates: &[(Uuid, Decimal, Decimal, Decimal)],
) -> AppResult<()>
// (id, mark_price, liquidation_price, unrealized_pnl)
{
    let mut tx = pool.begin().await?;
    for (id, mark, liq, pnl) in updates {
        sqlx::query(
            r#"UPDATE futures_positions
               SET mark_price = $2, liquidation_price = $3, unrealized_pnl = $4
               WHERE id = $1"#,
        )
        .bind(id).bind(mark).bind(liq).bind(pnl)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
