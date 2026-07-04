//! `wallets` + `wallet_ledger` queries.
//!
//! All mutating operations are wrapped in transactions and emit ledger rows
//! for full audit traceability.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Wallet, WalletType};

pub async fn find_or_create(
    pool: &PgPool,
    user_id: Uuid,
    asset_symbol: &str,
    wallet_type: WalletType,
) -> AppResult<Wallet> {
    if let Some(w) = sqlx::query_as::<_, Wallet>(
        "SELECT * FROM wallets WHERE user_id = $1 AND asset_symbol = $2",
    )
    .bind(user_id)
    .bind(asset_symbol)
    .fetch_optional(pool)
    .await?
    {
        return Ok(w);
    }
    let w = sqlx::query_as::<_, Wallet>(
        r#"
        INSERT INTO wallets (user_id, asset_symbol, wallet_type)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(asset_symbol)
    .bind(wallet_type)
    .fetch_one(pool)
    .await?;
    Ok(w)
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Wallet>> {
    let wallets = sqlx::query_as::<_, Wallet>(
        "SELECT * FROM wallets WHERE user_id = $1 ORDER BY asset_symbol",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(wallets)
}

pub async fn get(pool: &PgPool, user_id: Uuid, asset: &str) -> AppResult<Wallet> {
    sqlx::query_as::<_, Wallet>(
        "SELECT * FROM wallets WHERE user_id = $1 AND asset_symbol = $2",
    )
    .bind(user_id)
    .bind(asset)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("wallet {asset}")))
}

/// Atomically:
/// 1. Lock `amount` of the user's `asset` wallet (for placing a sell order).
/// 2. Emit a ledger entry.
///
/// Returns the updated wallet.
pub async fn lock_balance(
    pool: &PgPool,
    user_id: Uuid,
    asset: &str,
    amount: Decimal,
    ref_id: Option<Uuid>,
    reason: &str,
) -> AppResult<Wallet> {
    let mut tx = pool.begin().await?;

    let wallet: Wallet = sqlx::query_as::<_, Wallet>(
        r#"
        UPDATE wallets
        SET locked_balance = locked_balance + $3,
            balance = balance - $3,
            updated_at = now()
        WHERE user_id = $1 AND asset_symbol = $2
          AND balance >= $3
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(asset)
    .bind(amount)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::InsufficientBalance {
        required: amount.to_string(),
        available: "0".into(),
    })?;

    sqlx::query(
        r#"
        INSERT INTO wallet_ledger (wallet_id, user_id, delta, balance_after, reason, ref_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(wallet.id)
    .bind(user_id)
    .bind(-amount)
    .bind(wallet.balance)
    .bind(reason)
    .bind(ref_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(wallet)
}

/// Reverse a previous lock (used on order cancel).
pub async fn unlock_balance(
    pool: &PgPool,
    user_id: Uuid,
    asset: &str,
    amount: Decimal,
    ref_id: Option<Uuid>,
    reason: &str,
) -> AppResult<Wallet> {
    let mut tx = pool.begin().await?;

    let wallet: Wallet = sqlx::query_as::<_, Wallet>(
        r#"
        UPDATE wallets
        SET locked_balance = locked_balance - $3,
            balance = balance + $3,
            updated_at = now()
        WHERE user_id = $1 AND asset_symbol = $2
          AND locked_balance >= $3
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(asset)
    .bind(amount)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Internal(format!(
        "cannot unlock {amount} {asset} for user {user_id}"
    )))?;

    sqlx::query(
        r#"
        INSERT INTO wallet_ledger (wallet_id, user_id, delta, balance_after, reason, ref_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(wallet.id)
    .bind(user_id)
    .bind(amount)
    .bind(wallet.balance)
    .bind(reason)
    .bind(ref_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(wallet)
}

/// Credit a wallet — used on trade settlement, deposit completion, order refund.
pub async fn credit(
    pool: &PgPool,
    user_id: Uuid,
    asset: &str,
    amount: Decimal,
    ref_id: Option<Uuid>,
    reason: &str,
) -> AppResult<Wallet> {
    let mut tx = pool.begin().await?;

    let wallet: Wallet = sqlx::query_as::<_, Wallet>(
        r#"
        INSERT INTO wallets (user_id, asset_symbol, wallet_type, balance, locked_balance)
        VALUES ($1, $2,
            CASE WHEN $2 = 'EGP' THEN 'fiat'::wallet_type ELSE 'crypto'::wallet_type END,
            $3, 0)
        ON CONFLICT (user_id, asset_symbol)
        DO UPDATE SET balance = wallets.balance + $3,
                      updated_at = now()
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(asset)
    .bind(amount)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO wallet_ledger (wallet_id, user_id, delta, balance_after, reason, ref_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(wallet.id)
    .bind(user_id)
    .bind(amount)
    .bind(wallet.balance)
    .bind(reason)
    .bind(ref_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(wallet)
}

/// Debit a wallet directly (used for crypto withdrawal completion,
/// or admin manual adjustment). Returns updated wallet.
pub async fn debit(
    pool: &PgPool,
    user_id: Uuid,
    asset: &str,
    amount: Decimal,
    ref_id: Option<Uuid>,
    reason: &str,
) -> AppResult<Wallet> {
    let mut tx = pool.begin().await?;

    let wallet: Wallet = sqlx::query_as::<_, Wallet>(
        r#"
        UPDATE wallets
        SET balance = balance - $3,
            updated_at = now()
        WHERE user_id = $1 AND asset_symbol = $2
          AND balance >= $3
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(asset)
    .bind(amount)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::InsufficientBalance {
        required: amount.to_string(),
        available: "0".into(),
    })?;

    sqlx::query(
        r#"
        INSERT INTO wallet_ledger (wallet_id, user_id, delta, balance_after, reason, ref_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(wallet.id)
    .bind(user_id)
    .bind(-amount)
    .bind(wallet.balance)
    .bind(reason)
    .bind(ref_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(wallet)
}

/// Aggregate system liquidity across all wallets per asset.
pub async fn system_liquidity(pool: &PgPool) -> AppResult<Vec<(String, Decimal, Decimal)>> {
    let rows: Vec<(String, Decimal, Decimal)> = sqlx::query_as(
        r#"
        SELECT asset_symbol,
               COALESCE(SUM(balance), 0),
               COALESCE(SUM(locked_balance), 0)
        FROM wallets
        GROUP BY asset_symbol
        ORDER BY asset_symbol
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
