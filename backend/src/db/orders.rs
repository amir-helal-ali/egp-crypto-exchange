//! `orders` table queries.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Order, OrderStatus};

pub async fn insert(pool: &PgPool, order: &Order) -> AppResult<Order> {
    let row = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders
          (id, user_id, pair, side, order_type, price, quantity, filled_quantity, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(order.id)
    .bind(order.user_id)
    .bind(&order.pair)
    .bind(order.side)
    .bind(order.order_type)
    .bind(order.price)
    .bind(order.quantity)
    .bind(order.filled_quantity)
    .bind(order.status)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<Order> {
    let row = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("order {id}")))?;
    Ok(row)
}

pub async fn list_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<Order>> {
    let rows = sqlx::query_as::<_, Order>(
        r#"
        SELECT * FROM orders
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_open_by_pair(pool: &PgPool, pair: &str) -> AppResult<Vec<Order>> {
    let rows = sqlx::query_as::<_, Order>(
        r#"
        SELECT * FROM orders
        WHERE pair = $1
          AND status IN ('open','partially_filled')
        ORDER BY created_at ASC
        "#,
    )
    .bind(pair)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Update filled quantity + status atomically. Used by matching engine settlement.
pub async fn update_fill(
    pool: &PgPool,
    order_id: Uuid,
    additional_filled: Decimal,
    new_status: OrderStatus,
) -> AppResult<Order> {
    let row = sqlx::query_as::<_, Order>(
        r#"
        UPDATE orders
        SET filled_quantity = filled_quantity + $2,
            status = $3,
            updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(order_id)
    .bind(additional_filled)
    .bind(new_status)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn set_status(
    pool: &PgPool,
    order_id: Uuid,
    status: OrderStatus,
) -> AppResult<Order> {
    let row = sqlx::query_as::<_, Order>(
        "UPDATE orders SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
    )
    .bind(order_id)
    .bind(status)
    .fetch_one(pool)
    .await?;
    Ok(row)
}
