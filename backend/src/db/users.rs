//! `users` table queries.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{not_found, AppError, AppResult};
use crate::models::{User, UserRole, UserStatus};

pub async fn create(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    full_name: &str,
    phone: Option<&str>,
    role: UserRole,
) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, full_name, phone, role, status, country)
        VALUES ($1, $2, $3, $4, $5, 'active', 'EG')
        RETURNING *
        "#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(full_name)
    .bind(phone)
    .bind(role)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE lower(email) = lower($1)")
        .bind(email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("user {email}")))?;
    Ok(user)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("user {id}")))?;
    Ok(user)
}

pub async fn list_paginated(
    pool: &PgPool,
    offset: i64,
    limit: i64,
    status_filter: Option<UserStatus>,
) -> AppResult<(Vec<User>, i64)> {
    let total: (i64,) = if let Some(s) = status_filter {
        sqlx::query_as("SELECT count(*) FROM users WHERE status = $1")
            .bind(s)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_as("SELECT count(*) FROM users")
            .fetch_one(pool)
            .await?
    };

    let users = if let Some(s) = status_filter {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE status = $1
            ORDER BY created_at DESC
            OFFSET $2 LIMIT $3
            "#,
        )
        .bind(s)
        .bind(offset)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            ORDER BY created_at DESC
            OFFSET $1 LIMIT $2
            "#,
        )
        .bind(offset)
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    Ok((users, total.0))
}

pub async fn update_status(
    pool: &PgPool,
    user_id: Uuid,
    status: UserStatus,
    kyc_level: Option<i16>,
) -> AppResult<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET status = $2,
            kyc_level = COALESCE($3, kyc_level)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(status)
    .bind(kyc_level)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("user {user_id}")))?;
    Ok(user)
}

pub async fn touch_last_login(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE users SET last_login_at = $2 WHERE id = $1")
        .bind(user_id)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn ensure_admin_bootstrap(
    pool: &PgPool,
    admin_email: &str,
) -> AppResult<()> {
    let existing: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE lower(email) = lower($1) AND role = 'admin'")
        .bind(admin_email)
        .fetch_optional(pool)
        .await?;
    if existing.is_some() {
        return Ok(());
    }

    let password_hash = bcrypt::hash("ChangeMe!Admin2024", bcrypt::DEFAULT_COST)?;
    sqlx::query(
        r#"
        INSERT INTO users (email, password_hash, full_name, role, status, country, kyc_level)
        VALUES ($1, $2, 'System Administrator', 'admin', 'active', 'EG', 3)
        "#,
    )
    .bind(admin_email)
    .bind(password_hash)
    .execute(pool)
    .await?;
    tracing::info!(email = admin_email, "bootstrapped admin user");
    Ok(())
}
