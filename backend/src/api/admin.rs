//! Admin endpoints: manual tx review, user management, liquidity monitor.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::AdminUser;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{
    AdminReviewRequest, ManualTransaction, ManualTxAssetClass, ManualTxStatus,
    ManualTxType, Order, Trade, UpdateUserStatusRequest, User, UserStatus,
};
use crate::redis::QueueEvent;
use crate::AppState;

// --- overview ---------------------------------------------------------------

pub async fn overview(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    let total_users: (i64,) = sqlx::query_as("SELECT count(*) FROM users")
        .fetch_one(&state.db).await?;
    let total_orders: (i64,) = sqlx::query_as("SELECT count(*) FROM orders")
        .fetch_one(&state.db).await?;
    let open_orders: (i64,) =
        sqlx::query_as("SELECT count(*) FROM orders WHERE status IN ('open','partially_filled')")
            .fetch_one(&state.db).await?;
    let total_trades: (i64,) = sqlx::query_as("SELECT count(*) FROM trades")
        .fetch_one(&state.db).await?;
    let pending_deposits: (i64,) =
        sqlx::query_as("SELECT count(*) FROM manual_transactions WHERE tx_type='deposit' AND status IN ('pending','under_review')")
            .fetch_one(&state.db).await?;
    let pending_withdrawals: (i64,) =
        sqlx::query_as("SELECT count(*) FROM manual_transactions WHERE tx_type='withdrawal' AND status IN ('pending','under_review')")
            .fetch_one(&state.db).await?;

    let liquidity = db::wallets::system_liquidity(&state.db).await?;

    Ok(Json(json!({
        "users": total_users.0,
        "orders": {
            "total": total_orders.0,
            "open": open_orders.0,
        },
        "trades": total_trades.0,
        "pending": {
            "deposits": pending_deposits.0,
            "withdrawals": pending_withdrawals.0,
        },
        "liquidity": liquidity.into_iter().map(|(s, bal, lock)| json!({
            "asset": s, "balance": bal, "locked": lock
        })).collect::<Vec<_>>(),
        "circuit_breaker_open": state.binance.breaker.is_open(),
    })))
}

// --- user management --------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<UserStatus>,
}

pub async fn list_users(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListUsersQuery>,
) -> AppResult<Json<Value>> {
    let offset = q.offset.unwrap_or(0).max(0);
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    let (users, total) = db::users::list_paginated(&state.db, offset, limit, q.status).await?;
    Ok(Json(json!({ "users": users, "total": total, "offset": offset, "limit": limit })))
}

pub async fn get_user(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<User>> {
    let user = db::users::find_by_id(&state.db, id).await?;
    Ok(Json(user))
}

pub async fn update_user_status(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserStatusRequest>,
) -> AppResult<Json<User>> {
    let user = db::users::update_status(&state.db, id, req.status, req.kyc_level).await?;

    sqlx::query(
        r#"INSERT INTO admin_audit_log (admin_id, action, target_type, target_id, details)
           VALUES ($1, 'update_user_status', 'user', $2, $3)"#,
    )
    .bind(admin.0.user_id)
    .bind(id)
    .bind(json!({ "status": req.status, "kyc_level": req.kyc_level }))
    .execute(&state.db)
    .await?;

    Ok(Json(user))
}

// --- manual transactions ----------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListManualTxQuery {
    pub tx_type: Option<ManualTxType>,
    pub status: Option<ManualTxStatus>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn list_manual_tx(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListManualTxQuery>,
) -> AppResult<Json<Value>> {
    let offset = q.offset.unwrap_or(0).max(0);
    let limit = q.limit.unwrap_or(50).clamp(1, 500);
    let (rows, total) = db::manual_transactions::list_for_admin(
        &state.db,
        q.status,
        q.tx_type,
        offset,
        limit,
    )
    .await?;

    // Attach queue position to each pending row.
    let mut enriched = Vec::with_capacity(rows.len());
    for r in rows {
        let position = if matches!(r.status, ManualTxStatus::Pending | ManualTxStatus::UnderReview) {
            let tx_type_str = match r.tx_type {
                ManualTxType::Deposit => "deposit",
                ManualTxType::Withdrawal => "withdrawal",
            };
            let asset_class_str = match r.asset_class {
                ManualTxAssetClass::Fiat => "fiat",
                ManualTxAssetClass::Crypto => "crypto",
            };
            state.queue.position(r.id, tx_type_str, asset_class_str).await.unwrap_or(0)
        } else {
            0
        };
        enriched.push(json!({
            "tx": r,
            "queue_position": position,
        }));
    }
    Ok(Json(json!({ "items": enriched, "total": total, "offset": offset, "limit": limit })))
}

pub async fn get_manual_tx(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ManualTransaction>> {
    let tx = db::manual_transactions::get(&state.db, id).await?;
    Ok(Json(tx))
}

pub async fn review_manual_tx(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<AdminReviewRequest>,
) -> AppResult<Json<ManualTransaction>> {
    let tx = db::manual_transactions::get(&state.db, id).await?;
    let prev_status = tx.status;

    let updated = db::manual_transactions::update_status(
        &state.db,
        id,
        req.status,
        admin.0.user_id,
        req.admin_note.as_deref(),
        req.tx_hash.as_deref(),
    )
    .await?;

    // Dequeue if final state.
    let is_final = matches!(
        req.status,
        ManualTxStatus::Completed | ManualTxStatus::Rejected | ManualTxStatus::Failed
    );
    if is_final {
        let tx_type_str = match tx.tx_type {
            ManualTxType::Deposit => "deposit",
            ManualTxType::Withdrawal => "withdrawal",
        };
        let asset_class_str = match tx.asset_class {
            ManualTxAssetClass::Fiat => "fiat",
            ManualTxAssetClass::Crypto => "crypto",
        };
        let _ = state.queue.dequeue(id, tx_type_str, asset_class_str).await;

        // Apply balance effects.
        match (tx.tx_type, req.status) {
            (ManualTxType::Deposit, ManualTxStatus::Completed) => {
                let _ = db::wallets::credit(
                    &state.db,
                    tx.user_id,
                    &tx.asset_symbol,
                    tx.amount,
                    Some(tx.id),
                    "deposit_completed",
                )
                .await;
            }
            (ManualTxType::Withdrawal, ManualTxStatus::Completed) => {
                // Balance was already locked at request time; now finalize the debit.
                let _ = db::wallets::unlock_balance(
                    &state.db,
                    tx.user_id,
                    &tx.asset_symbol,
                    tx.amount,
                    Some(tx.id),
                    "withdrawal_settle",
                )
                .await;
                let _ = db::wallets::debit(
                    &state.db,
                    tx.user_id,
                    &tx.asset_symbol,
                    tx.amount,
                    Some(tx.id),
                    "withdrawal_completed",
                )
                .await;
            }
            (ManualTxType::Withdrawal, ManualTxStatus::Rejected)
            | (ManualTxType::Withdrawal, ManualTxStatus::Failed) => {
                // Refund the locked balance.
                let _ = db::wallets::unlock_balance(
                    &state.db,
                    tx.user_id,
                    &tx.asset_symbol,
                    tx.amount,
                    Some(tx.id),
                    "withdrawal_refund",
                )
                .await;
            }
            _ => {}
        }
    }

    // Audit log.
    sqlx::query(
        r#"INSERT INTO admin_audit_log (admin_id, action, target_type, target_id, details)
           VALUES ($1, 'review_manual_tx', 'manual_transaction', $2, $3)"#,
    )
    .bind(admin.0.user_id)
    .bind(id)
    .bind(json!({
        "from": prev_status,
        "to": req.status,
        "note": req.admin_note,
        "tx_hash": req.tx_hash,
    }))
    .execute(&state.db)
    .await?;

    // Publish status update to the user's channel (Redis pub/sub).
    let event = QueueEvent {
        manual_tx_id: id,
        user_id: tx.user_id,
        status: req.status,
        queue_position: 0,
        ts: chrono::Utc::now(),
    };
    let _ = state.queue.publish_status(tx.user_id, &event).await;

    // --- بث عبر WebSocket للمستخدم (تحديث لحظي) ---
    state.ws_bus.emit_to_user(tx.user_id, json!({
        "type": "manual_tx_update",
        "tx": updated,
    }));
    // إذا اكتمل الإيداع/السحب، حدّث محفظة المستخدم أيضاً
    if is_final {
        if let Ok(w) = db::wallets::get(&state.db, tx.user_id, &tx.asset_symbol).await {
            state.ws_bus.emit_to_user(tx.user_id, json!({
                "type": "wallet_update",
                "wallet": w,
            }));
        }
    }

    Ok(Json(updated))
}

// --- liquidity --------------------------------------------------------------

pub async fn liquidity(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    let rows = db::wallets::system_liquidity(&state.db).await?;
    let items: Vec<_> = rows
        .into_iter()
        .map(|(s, bal, lock)| {
            json!({
                "asset": s,
                "balance": bal,
                "locked": lock,
                "available": bal - lock,
            })
        })
        .collect();
    Ok(Json(json!({ "liquidity": items })))
}

// --- orders & trades browser (admin) ---------------------------------------

pub async fn list_all_orders(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<Vec<Order>>> {
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let orders: Vec<Order> = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(orders))
}

pub async fn list_all_trades(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<Vec<Trade>>> {
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let trades: Vec<Trade> = sqlx::query_as::<_, Trade>(
        "SELECT * FROM trades ORDER BY executed_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(trades))
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn audit_log(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<Value>> {
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let offset = q.offset.unwrap_or(0).max(0);
    let rows: Vec<(i64, Uuid, String, Option<String>, Option<Uuid>, Value, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"SELECT id, admin_id, action, target_type::text, target_id, details, created_at
           FROM admin_audit_log
           ORDER BY created_at DESC
           OFFSET $1 LIMIT $2"#,
    )
    .bind(offset)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    let items: Vec<_> = rows
        .into_iter()
        .map(|(id, admin_id, action, target_type, target_id, details, ts)| {
            json!({
                "id": id,
                "admin_id": admin_id,
                "action": action,
                "target_type": target_type,
                "target_id": target_id,
                "details": details,
                "created_at": ts,
            })
        })
        .collect();
    Ok(Json(json!({ "items": items })))
}

// --- تعديل الأرصدة يدوياً - Manual wallet adjustment ---------------------

#[derive(Debug, Deserialize)]
pub struct AdjustWalletRequest {
    pub user_id: Uuid,
    pub asset: String,
    pub delta: Decimal,        // موجب = إضافة، سالب = خصم
    pub reason: String,
}

pub async fn adjust_wallet(
    admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<AdjustWalletRequest>,
) -> AppResult<Json<Value>> {
    if req.delta == Decimal::ZERO {
        return Err(AppError::BadRequest("المبلغ لا يمكن أن يكون صفراً".into()));
    }
    if req.reason.trim().is_empty() {
        return Err(AppError::BadRequest("السبب مطلوب".into()));
    }

    let wallet = if req.delta > Decimal::ZERO {
        db::wallets::credit(
            &state.db,
            req.user_id,
            &req.asset,
            req.delta,
            None,
            "admin_adjustment",
        )
        .await?
    } else {
        db::wallets::debit(
            &state.db,
            req.user_id,
            &req.asset,
            -req.delta,
            None,
            "admin_adjustment",
        )
        .await?
    };

    // سجل التدقيق
    sqlx::query(
        r#"INSERT INTO admin_audit_log (admin_id, action, target_type, target_id, details)
           VALUES ($1, 'adjust_wallet', 'wallet', $2, $3)"#,
    )
    .bind(admin.0.user_id)
    .bind(wallet.id)
    .bind(json!({
        "user_id": req.user_id,
        "asset": req.asset,
        "delta": req.delta,
        "reason": req.reason,
        "new_balance": wallet.balance,
    }))
    .execute(&state.db)
    .await?;

    // بث تحديث المحفظة للمستخدم
    state.ws_bus.emit_to_user(req.user_id, json!({
        "type": "wallet_update",
        "wallet": wallet,
    }));

    Ok(Json(json!({ "wallet": wallet, "adjusted": req.delta })))
}

/// Admin: قائمة محافظ مستخدم معين
pub async fn list_user_wallets(
    _admin: AdminUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<Vec<crate::models::Wallet>>> {
    let wallets = db::wallets::list_for_user(&state.db, user_id).await?;
    Ok(Json(wallets))
}
