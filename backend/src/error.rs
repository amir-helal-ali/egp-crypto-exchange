//! Centralized error handling.
//!
//! All fallible operations return `Result<T, AppError>`. The `?` operator
//! propagates `sqlx::Error`, `redis::RedisError`, `jsonwebtoken::errors::Error`,
//! and other common error types automatically.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

/// The single error type used throughout the application.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("rate limited: {0}")]
    RateLimited(String),

    #[error("circuit breaker open: binance feed unavailable")]
    CircuitBreakerOpen,

    #[error("insufficient balance: need {required}, have {available}")]
    InsufficientBalance {
        required: String,
        available: String,
    },

    #[error("order rejected: {0}")]
    OrderRejected(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    /// Map the error to an HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) | AppError::OrderRejected(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::CircuitBreakerOpen => StatusCode::SERVICE_UNAVAILABLE,
            AppError::InsufficientBalance { .. } => StatusCode::PAYMENT_REQUIRED,
            AppError::Internal(_) | AppError::Database(_) | AppError::Anyhow(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::Redis(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
            AppError::Bcrypt(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SerdeJson(_) => StatusCode::BAD_REQUEST,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Stable error code (string) the frontend can switch on.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "not_found",
            AppError::Unauthorized(_) => "unauthorized",
            AppError::Forbidden(_) => "forbidden",
            AppError::BadRequest(_) => "bad_request",
            AppError::Conflict(_) => "conflict",
            AppError::RateLimited(_) => "rate_limited",
            AppError::CircuitBreakerOpen => "circuit_breaker_open",
            AppError::InsufficientBalance { .. } => "insufficient_balance",
            AppError::OrderRejected(_) => "order_rejected",
            AppError::Internal(_) => "internal",
            AppError::Database(_) => "database_error",
            AppError::Redis(_) => "redis_error",
            AppError::Jwt(_) => "jwt_error",
            AppError::Bcrypt(_) => "bcrypt_error",
            AppError::SerdeJson(_) => "serde_error",
            AppError::Io(_) => "io_error",
            AppError::Anyhow(_) => "anyhow",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code();
        // Don't leak internal error details in production for 5xx.
        let message = match &self {
            AppError::Internal(_)
            | AppError::Database(_)
            | AppError::Redis(_)
            | AppError::Bcrypt(_)
            | AppError::Io(_)
            | AppError::Anyhow(_) => "internal server error".to_string(),
            other => other.to_string(),
        };

        // Log the full error server-side.
        match status.is_server_error() {
            true => tracing::error!(error = ?self, "server error"),
            false => tracing::warn!(error = ?self, code = code, "client error"),
        }

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message,
            }
        }));
        (status, body).into_response()
    }
}

/// Convenience type alias used everywhere.
pub type AppResult<T> = Result<T, AppError>;

/// Helper to convert a SQLx `RowNotFound` into `AppError::NotFound`.
pub fn not_found<T>(msg: impl Into<String>) -> AppResult<T> {
    Err(AppError::NotFound(msg.into()))
}
