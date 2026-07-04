//! Wallet endpoints: deposit / withdrawal requests, balance list, ledger.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::auth::AuthUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{
    CryptoWithdrawalRequest, ManualDepositRequest, ManualTransaction, ManualTxStatus,
    ManualTxType, Wallet,
};
use crate::redis::QueueEvent;
use crate::AppState;

pub async fn list_wallets(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<Wallet>>> {
    let wallets = db::wallets::list_for_user(&state.db, auth.user_id).await?;
    Ok(Json(wallets))
}

pub async fn request_deposit(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ManualDepositRequest>,
) -> AppResult<Json<ManualTransaction>> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let min_deposit = state.fees.min_egp_deposit;
    if req.amount < min_deposit {
        return Err(AppError::BadRequest(format!(
            "minimum EGP deposit is {}",
            min_deposit
        )));
    }

    let tx = db::manual_transactions::create_deposit(
        &state.db,
        auth.user_id,
        "EGP",
        req.amount,
        &req.reference,
        req.receipt_url.as_deref(),
    )
    .await?;

    state
        .queue
        .enqueue(tx.id, auth.user_id, "deposit", "fiat")
        .await?;

    let position = state
        .queue
        .position(tx.id, "deposit", "fiat")
        .await
        .unwrap_or(0);

    let event = QueueEvent {
        manual_tx_id: tx.id,
        user_id: auth.user_id,
        status: ManualTxStatus::Pending,
        queue_position: position,
        ts: chrono::Utc::now(),
    };
    let _ = state.queue.publish_status(auth.user_id, &event).await;

    Ok(Json(tx))
}

pub async fn request_withdrawal(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CryptoWithdrawalRequest>,
) -> AppResult<Json<ManualTransaction>> {
    req.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    // Verify wallet has sufficient balance.
    let wallet = db::wallets::get(&state.db, auth.user_id, &req.asset_symbol).await?;
    if wallet.balance < req.amount {
        return Err(AppError::InsufficientBalance {
            required: req.amount.to_string(),
            available: wallet.balance.to_string(),
        });
    }

    // Lock the withdrawn amount until admin completes/rejects.
    let _ = db::wallets::lock_balance(
        &state.db,
        auth.user_id,
        &req.asset_symbol,
        req.amount,
        None,
        "withdrawal_lock",
    )
    .await?;

    let tx = db::manual_transactions::create_crypto_withdrawal(
        &state.db,
        auth.user_id,
        &req.asset_symbol,
        req.amount,
        &req.destination,
    )
    .await?;

    state
        .queue
        .enqueue(tx.id, auth.user_id, "withdrawal", "crypto")
        .await?;

    let position = state
        .queue
        .position(tx.id, "withdrawal", "crypto")
        .await
        .unwrap_or(0);

    let event = QueueEvent {
        manual_tx_id: tx.id,
        user_id: auth.user_id,
        status: ManualTxStatus::Pending,
        queue_position: position,
        ts: chrono::Utc::now(),
    };
    let _ = state.queue.publish_status(auth.user_id, &event).await;

    Ok(Json(tx))
}

#[derive(Debug, Deserialize)]
pub struct TxTypeQuery {
    pub tx_type: Option<String>,
}

pub async fn list_my_deposits(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<ManualTransaction>>> {
    let rows = db::manual_transactions::list_for_user(&state.db, auth.user_id, 200).await?;
    let deposits: Vec<_> = rows.into_iter().filter(|t| t.tx_type == ManualTxType::Deposit).collect();
    Ok(Json(deposits))
}

pub async fn list_my_withdrawals(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<ManualTransaction>>> {
    let rows = db::manual_transactions::list_for_user(&state.db, auth.user_id, 200).await?;
    let withdrawals: Vec<_> = rows.into_iter().filter(|t| t.tx_type == ManualTxType::Withdrawal).collect();
    Ok(Json(withdrawals))
}

/// WebSocket endpoint: live updates for the user's manual transactions.
/// Returns 200 with instructions (the actual upgrade is handled in ws.rs).
pub async fn withdrawal_status_ws(
    _auth: AuthUser,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    axum::response::Response::from(
        (
            axum::http::StatusCode::OK,
            Json(json!({
                "message": "use the /api/market/ws endpoint with auth token to subscribe to all events"
            })),
        )
            .into_response(),
    )
}

pub async fn ledger(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(asset): Option<Path<String>>,
) -> AppResult<Json<Value>> {
    let asset_filter = asset;
    let wallets = db::wallets::list_for_user(&state.db, auth.user_id).await?;

    let mut entries = Vec::new();
    for w in &wallets {
        if let Some(ref a) = asset_filter {
            if &w.asset_symbol != a {
                continue;
            }
        }
        let rows: Vec<crate::models::WalletLedgerEntry> = sqlx::query_as::<_, crate::models::WalletLedgerEntry>(
            r#"SELECT * FROM wallet_ledger
               WHERE wallet_id = $1
               ORDER BY created_at DESC
               LIMIT 100"#,
        )
        .bind(w.id)
        .fetch_all(&state.db)
        .await?;
        entries.push(json!({
            "asset": w.asset_symbol,
            "balance": w.balance,
            "locked": w.locked_balance,
            "ledger": rows,
        }));
    }
    Ok(Json(json!({ "wallets": entries })))
}

#[allow(dead_code)]
fn _unused(_: Decimal, _: Uuid) {}
