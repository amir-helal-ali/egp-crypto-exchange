//! EGP Exchange Backend - Application configuration
//!
//! All settings are loaded from environment variables (12-factor).

use std::env;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub log_level: String,

    pub database_url: String,
    pub database_max_connections: u32,

    pub redis_url: String,

    pub jwt_secret: String,
    pub jwt_access_ttl_hours: i64,
    pub jwt_refresh_ttl_days: i64,
    pub admin_bootstrap_email: String,

    pub binance_ws_url: String,
    pub binance_rest_url: String,
    pub binance_streams: String,
    pub egp_usd_rate: rust_decimal::Decimal,
    pub circuit_breaker_timeout_secs: u64,
    pub circuit_breaker_max_failures: u32,

    pub user_frontend_origin: String,
    pub admin_frontend_origin: String,

    pub queue_prefix: String,
}

impl Config {
    /// Load configuration strictly from environment variables.
    ///
    /// Returns `Err` if any required variable is missing or malformed — we
    /// never silently fall back to insecure defaults in production.
    pub fn from_env() -> Result<Self> {
        // Helper closures to keep reading terse and uniform.
        let req = |key: &str| -> Result<String> {
            env::var(key).with_context(|| format!("missing env var {key}"))
        };
        let opt_or = |key: &str, default: &str| -> String {
            env::var(key).unwrap_or_else(|_| default.to_string())
        };
        let parse = |key: &str, val: String| -> Result<String> {
            if val.trim().is_empty() {
                anyhow::bail!("env var {key} is empty")
            }
            Ok(val)
        };

        let server_host = opt_or("SERVER_HOST", "0.0.0.0");
        let server_port: u16 = opt_or("SERVER_PORT", "8080")
            .parse()
            .context("SERVER_PORT must be a u16")?;
        let log_level = opt_or("LOG_LEVEL", "info");

        let database_url = parse("DATABASE_URL", req("DATABASE_URL")?)?;
        let database_max_connections: u32 = opt_or("DATABASE_MAX_CONNECTIONS", "20")
            .parse()
            .context("DATABASE_MAX_CONNECTIONS must be u32")?;

        let redis_url = parse("REDIS_URL", req("REDIS_URL")?)?;

        let jwt_secret = {
            let s = req("JWT_SECRET")?;
            if s.len() < 32 {
                anyhow::bail!("JWT_SECRET must be at least 32 chars");
            }
            s
        };
        let jwt_access_ttl_hours: i64 = opt_or("JWT_ACCESS_TTL_HOURS", "24")
            .parse()
            .context("JWT_ACCESS_TTL_HOURS must be i64")?;
        let jwt_refresh_ttl_days: i64 = opt_or("JWT_REFRESH_TTL_DAYS", "30")
            .parse()
            .context("JWT_REFRESH_TTL_DAYS must be i64")?;
        let admin_bootstrap_email = req("ADMIN_BOOTSTRAP_EMAIL")?;

        let binance_ws_url = opt_or(
            "BINANCE_WS_URL",
            "wss://stream.binance.com:9443/stream",
        );
        let binance_rest_url = opt_or("BINANCE_REST_URL", "https://api.binance.com");
        let binance_streams = opt_or(
            "BINANCE_STREAMS",
            "btcusdt@bookTicker,ethusdt@bookTicker,usdtusdc@bookTicker",
        );
        let egp_usd_rate: rust_decimal::Decimal = opt_or("EGP_USD_RATE", "48.5")
            .parse()
            .context("EGP_USD_RATE must be a decimal")?;
        let circuit_breaker_timeout_secs: u64 = opt_or("CIRCUIT_BREAKER_TIMEOUT_SECS", "30")
            .parse()
            .context("CIRCUIT_BREAKER_TIMEOUT_SECS must be u64")?;
        let circuit_breaker_max_failures: u32 = opt_or("CIRCUIT_BREAKER_MAX_FAILURES", "5")
            .parse()
            .context("CIRCUIT_BREAKER_MAX_FAILURES must be u32")?;

        let user_frontend_origin = opt_or("USER_FRONTEND_ORIGIN", "http://localhost:3000");
        let admin_frontend_origin = opt_or("ADMIN_FRONTEND_ORIGIN", "http://localhost:3001");
        let queue_prefix = opt_or("QUEUE_PREFIX", "egp_exchange");

        Ok(Self {
            server_host,
            server_port,
            log_level,
            database_url,
            database_max_connections,
            redis_url,
            jwt_secret,
            jwt_access_ttl_hours,
            jwt_refresh_ttl_days,
            admin_bootstrap_email,
            binance_ws_url,
            binance_rest_url,
            binance_streams,
            egp_usd_rate,
            circuit_breaker_timeout_secs,
            circuit_breaker_max_failures,
            user_frontend_origin,
            admin_frontend_origin,
            queue_prefix,
        })
    }

    pub fn circuit_breaker_timeout(&self) -> Duration {
        Duration::from_secs(self.circuit_breaker_timeout_secs)
    }

    pub fn binance_combined_stream_url(&self) -> String {
        // Binance combined stream URL: wss://stream.binance.com:9443/stream?streams=a,b,c
        let streams = self.binance_streams.replace(',', "/");
        format!("{}?streams={}", self.binance_ws_url, streams)
    }
}
