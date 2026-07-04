//! `manual_transactions` table queries + queue helpers.

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{
    ManualTransaction, ManualTxAssetClass, ManualTxStatus, ManualTxType,
};

pub async fn create_deposit(
    pool: &PgPool,
    user_id: Uuid,
    asset_symbol: &str,
    amount: Decimal,
    reference: &str,
    receipt_url: Option<&str>,
) -> AppResult<ManualTransaction> {
    let row = sqlx::query_as::<_, ManualTransaction>(
        r#"
        INSERT INTO manual_transactions
          (user_id, tx_type, asset_class, asset_symbol, amount, status,
           reference, receipt_url)
        VALUES ($1, 'deposit', 'fiat', $2, $3, 'pending', $4, $5)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(asset_symbol)
    .bind(amount)
    .bind(reference)
    .bind(receipt_url)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn create_crypto_withdrawal(
    pool: &PgPool,
    user_id: Uuid,
    asset_symbol: &str,
    amount: Decimal,
    destination: &str,
) -> AppResult<ManualTransaction> {
    let row = sqlx::query_as::<_, ManualTransaction>(
        r#"
        INSERT INTO manual_transactions
          (user_id, tx_type, asset_class, asset_symbol, amount, status, destination)
        VALUES ($1, 'withdrawal', 'crypto', $2, $3, 'pending', $4)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(asset_symbol)
    .bind(amount)
    .bind(destination)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get(pool: &PgPool, id: Uuid) -> AppResult<ManualTransaction> {
    sqlx::query_as::<_, ManualTransaction>(
        "SELECT * FROM manual_transactions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("manual_tx {id}")))
}

pub async fn list_for_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<ManualTransaction>> {
    let rows = sqlx::query_as::<_, ManualTransaction>(
        r#"
        SELECT * FROM manual_transactions
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

pub async fn list_for_admin(
    pool: &PgPool,
    status_filter: Option<ManualTxStatus>,
    tx_type_filter: Option<ManualTxType>,
    offset: i64,
    limit: i64,
) -> AppResult<(Vec<ManualTransaction>, i64)> {
    let total: (i64,) = match (status_filter, tx_type_filter) {
        (Some(s), Some(t)) => {
            sqlx::query_as("SELECT count(*) FROM manual_transactions WHERE status=$1 AND tx_type=$2")
                .bind(s).bind(t).fetch_one(pool).await?
        }
        (Some(s), None) => {
            sqlx::query_as("SELECT count(*) FROM manual_transactions WHERE status=$1")
                .bind(s).fetch_one(pool).await?
        }
        (None, Some(t)) => {
            sqlx::query_as("SELECT count(*) FROM manual_transactions WHERE tx_type=$1")
                .bind(t).fetch_one(pool).await?
        }
        (None, None) => {
            sqlx::query_as("SELECT count(*) FROM manual_transactions")
                .fetch_one(pool).await?
        }
    };

    let rows = match (status_filter, tx_type_filter) {
        (Some(s), Some(t)) => sqlx::query_as::<_, ManualTransaction>(
            r#"SELECT * FROM manual_transactions WHERE status=$1 AND tx_type=$2
               ORDER BY created_at ASC OFFSET $3 LIMIT $4"#)
            .bind(s).bind(t).bind(offset).bind(limit).fetch_all(pool).await?,
        (Some(s), None) => sqlx::query_as::<_, ManualTransaction>(
            r#"SELECT * FROM manual_transactions WHERE status=$1
               ORDER BY created_at ASC OFFSET $2 LIMIT $3"#)
            .bind(s).bind(offset).bind(limit).fetch_all(pool).await?,
        (None, Some(t)) => sqlx::query_as::<_, ManualTransaction>(
            r#"SELECT * FROM manual_transactions WHERE tx_type=$1
               ORDER BY created_at ASC OFFSET $2 LIMIT $3"#)
            .bind(t).bind(offset).bind(limit).fetch_all(pool).await?,
        (None, None) => sqlx::query_as::<_, ManualTransaction>(
            r#"SELECT * FROM manual_transactions
               ORDER BY created_at ASC OFFSET $1 LIMIT $2"#)
            .bind(offset).bind(limit).fetch_all(pool).await?,
    };
    Ok((rows, total.0))
}

pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    new_status: ManualTxStatus,
    reviewed_by: Uuid,
    admin_note: Option<&str>,
    tx_hash: Option<&str>,
) -> AppResult<ManualTransaction> {
    let now = Utc::now();
    let completed_at = if matches!(new_status, ManualTxStatus::Completed) {
        Some(now)
    } else {
        None
    };
    let row = sqlx::query_as::<_, ManualTransaction>(
        r#"
        UPDATE manual_transactions
        SET status = $2,
            reviewed_by = $3,
            admin_note = COALESCE($4, admin_note),
            tx_hash = COALESCE($5, tx_hash),
            reviewed_at = $6,
            completed_at = COALESCE($7, completed_at)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(new_status)
    .bind(reviewed_by)
    .bind(admin_note)
    .bind(tx_hash)
    .bind(now)
    .bind(completed_at)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("manual_tx {id}")))?;
    Ok(row)
}

#[allow(dead_code)]
pub async fn list_pending_by_class(
    pool: &PgPool,
    asset_class: ManualTxAssetClass,
    limit: i64,
) -> AppResult<Vec<ManualTransaction>> {
    let rows = sqlx::query_as::<_, ManualTransaction>(
        r#"SELECT * FROM manual_transactions
           WHERE status IN ('pending','under_review') AND asset_class = $1
           ORDER BY created_at ASC LIMIT $2"#,
    )
    .bind(asset_class)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
