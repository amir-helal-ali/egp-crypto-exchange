//! `settings` table queries (key/value JSONB).

use serde_json::Value;
use sqlx::PgPool;

use crate::error::AppResult;

pub async fn get(pool: &PgPool, key: &str) -> AppResult<Value> {
    let row: Option<(Value,)> = sqlx::query_as("SELECT value FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.0).unwrap_or(Value::Null))
}

pub async fn set(pool: &PgPool, key: &str, value: &Value) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO settings (key, value)
        VALUES ($1, $2)
        ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = now()
        "#,
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}
