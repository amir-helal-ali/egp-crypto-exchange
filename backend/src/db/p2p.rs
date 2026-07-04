//! استعلامات التداول بين الأفراد - P2P queries

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{P2PMessage, P2POffer, P2PSide, P2PTrade, P2PTradeStatus};

// --- العروض - Offers -------------------------------------------------------

pub async fn create_offer(
    pool: &PgPool,
    user_id: Uuid,
    side: P2PSide,
    asset_symbol: &str,
    price_margin_pct: Decimal,
    min_amount_egp: Decimal,
    max_amount_egp: Decimal,
    payment_methods: &[String],
    time_limit_min: i32,
) -> AppResult<P2POffer> {
    let offer = sqlx::query_as::<_, P2POffer>(
        r#"
        INSERT INTO p2p_offers
          (user_id, side, asset_symbol, price_margin_pct, min_amount_egp,
           max_amount_egp, payment_methods, time_limit_min, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'active')
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(side)
    .bind(asset_symbol)
    .bind(price_margin_pct)
    .bind(min_amount_egp)
    .bind(max_amount_egp)
    .bind(payment_methods)
    .bind(time_limit_min)
    .fetch_one(pool)
    .await?;
    Ok(offer)
}

pub async fn get_offer(pool: &PgPool, id: Uuid) -> AppResult<P2POffer> {
    let offer = sqlx::query_as::<_, P2POffer>("SELECT * FROM p2p_offers WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("offer {id}")))?;
    Ok(offer)
}

pub async fn list_offers(
    pool: &PgPool,
    side: Option<P2PSide>,
    asset: Option<&str>,
    payment_method: Option<&str>,
) -> AppResult<Vec<P2POffer>> {
    let rows = if let Some(s) = side {
        sqlx::query_as::<_, P2POffer>(
            r#"SELECT o.*, u.email as user_email FROM p2p_offers o
               JOIN users u ON u.id = o.user_id
               WHERE o.status = 'active' AND o.side = $1
               ORDER BY o.created_at DESC"#,
        )
        .bind(s)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, P2POffer>(
            r#"SELECT o.*, u.email as user_email FROM p2p_offers o
               JOIN users u ON u.id = o.user_id
               WHERE o.status = 'active'
               ORDER BY o.created_at DESC"#,
        )
        .fetch_all(pool)
        .await?
    };

    let rows = if let Some(a) = asset {
        rows.into_iter().filter(|o| o.asset_symbol == a).collect()
    } else { rows };

    let rows = if let Some(pm) = payment_method {
        rows.into_iter().filter(|o| o.payment_methods.iter().any(|m| m == pm)).collect()
    } else { rows };

    Ok(rows)
}

// --- الصفقات - Trades -----------------------------------------------------

pub async fn create_trade(
    pool: &PgPool,
    offer_id: Uuid,
    buyer_id: Uuid,
    seller_id: Uuid,
    asset_symbol: &str,
    amount: Decimal,
    price_egp: Decimal,
    total_egp: Decimal,
    payment_method: &str,
) -> AppResult<P2PTrade> {
    let trade = sqlx::query_as::<_, P2PTrade>(
        r#"
        INSERT INTO p2p_trades
          (offer_id, buyer_id, seller_id, asset_symbol, amount, price_egp,
           total_egp, payment_method, status, escrow_locked)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', true)
        RETURNING *
        "#,
    )
    .bind(offer_id)
    .bind(buyer_id)
    .bind(seller_id)
    .bind(asset_symbol)
    .bind(amount)
    .bind(price_egp)
    .bind(total_egp)
    .bind(payment_method)
    .fetch_one(pool)
    .await?;

    // زيادة عداد الصفقات للعرض
    sqlx::query("UPDATE p2p_offers SET total_trades = total_trades + 1 WHERE id = $1")
        .bind(offer_id)
        .execute(pool)
        .await?;

    Ok(trade)
}

pub async fn get_trade(pool: &PgPool, id: Uuid) -> AppResult<P2PTrade> {
    sqlx::query_as::<_, P2PTrade>("SELECT * FROM p2p_trades WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("trade {id}")))
}

pub async fn list_my_trades(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<P2PTrade>> {
    let rows = sqlx::query_as::<_, P2PTrade>(
        r#"SELECT * FROM p2p_trades
           WHERE buyer_id = $1 OR seller_id = $1
           ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update_trade_status(
    pool: &PgPool,
    id: Uuid,
    new_status: P2PTradeStatus,
) -> AppResult<P2PTrade> {
    let now = Utc::now();
    let trade = sqlx::query_as::<_, P2PTrade>(
        r#"
        UPDATE p2p_trades
        SET status = $2,
            paid_at = CASE WHEN $2 = 'paid' THEN $3 ELSE paid_at END,
            released_at = CASE WHEN $2 = 'released' OR $2 = 'completed' THEN $3 ELSE released_at END,
            cancelled_at = CASE WHEN $2 = 'cancelled' THEN $3 ELSE cancelled_at END,
            completed_at = CASE WHEN $2 = 'completed' THEN $3 ELSE completed_at END
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(new_status)
    .bind(now)
    .fetch_one(pool)
    .await?;
    Ok(trade)
}

// --- الرسائل - Messages ---------------------------------------------------

pub async fn list_messages(pool: &PgPool, trade_id: Uuid) -> AppResult<Vec<P2PMessage>> {
    let rows = sqlx::query_as::<_, P2PMessage>(
        r#"SELECT * FROM p2p_messages
           WHERE trade_id = $1
           ORDER BY created_at ASC"#,
    )
    .bind(trade_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn send_message(
    pool: &PgPool,
    trade_id: Uuid,
    sender_id: Uuid,
    message: &str,
) -> AppResult<P2PMessage> {
    let msg = sqlx::query_as::<_, P2PMessage>(
        r#"INSERT INTO p2p_messages (trade_id, sender_id, message)
           VALUES ($1, $2, $3) RETURNING *"#,
    )
    .bind(trade_id)
    .bind(sender_id)
    .bind(message)
    .fetch_one(pool)
    .await?;
    Ok(msg)
}
