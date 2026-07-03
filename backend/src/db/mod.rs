//! Database access module.
//!
//! Each submodule owns the queries for one table (or logical group). All
//! functions are async and return `AppResult<T>`.

pub mod users;
pub mod wallets;
pub mod orders;
pub mod trades;
pub mod manual_transactions;
pub mod settings;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::error::AppResult;

pub type Db = PgPool;

/// Build the connection pool with sane defaults.
pub async fn connect(database_url: &str, max_connections: u32) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(Some(std::time::Duration::from_secs(300)))
        .max_lifetime(Some(std::time::Duration::from_secs(1800)))
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Run SQLx migrations embedded at compile time.
pub async fn migrate(pool: &PgPool) -> AppResult<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
