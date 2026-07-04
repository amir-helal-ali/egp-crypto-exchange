//! Authentication endpoints: register / login / refresh.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use uuid::Uuid;
use validator::Validate;

use crate::auth::{AuthUser, TokenType};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{
    AuthResponse, LoginRequest, RegisterRequest, UserPublic, UserRole,
};
use crate::AppState;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let existing = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT id FROM users WHERE lower(email) = lower($1)",
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await?;
    if existing.is_some() {
        return Err(AppError::Conflict("email already registered".into()));
    }

    let password_hash =
        bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)?;
    let user = db::users::create(
        &state.db,
        &req.email,
        &password_hash,
        &req.full_name,
        req.phone.as_deref(),
        UserRole::User,
    )
    .await?;

    // Bootstrap default wallets.
    let default_wallets: &[(&str, crate::models::WalletType)] = &[
        ("EGP", crate::models::WalletType::Fiat),
        ("BTC", crate::models::WalletType::Crypto),
        ("ETH", crate::models::WalletType::Crypto),
        ("USDT", crate::models::WalletType::Crypto),
    ];
    for (asset, wtype) in default_wallets {
        let _ = db::wallets::find_or_create(&state.db, user.id, asset, *wtype).await;
    }

    let (access, refresh) = state.jwt.issue_pair(user.id, user.role)?;
    db::users::touch_last_login(&state.db, user.id).await?;

    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh,
        user: UserPublic::from(user),
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let user = db::users::find_by_email(&state.db, &req.email).await?;

    if user.status != crate::models::UserStatus::Active {
        return Err(AppError::Forbidden(format!("account status: {:?}", user.status)));
    }

    if !bcrypt::verify(&req.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    db::users::touch_last_login(&state.db, user.id).await?;

    let (access, refresh) = state.jwt.issue_pair(user.id, user.role)?;
    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh,
        user: UserPublic::from(user),
    }))
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<AuthResponse>> {
    let token = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing refresh_token".into()))?;
    let claims = state.jwt.verify(token)?;
    if claims.typ != TokenType::Refresh {
        return Err(AppError::Unauthorized("expected refresh token".into()));
    }
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("invalid subject".into()))?;
    let user = db::users::find_by_id(&state.db, user_id).await?;

    let (access, refresh) = state.jwt.issue_pair(user.id, user.role)?;
    Ok(Json(AuthResponse {
        access_token: access,
        refresh_token: refresh,
        user: UserPublic::from(user),
    }))
}

pub async fn me(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<UserPublic>> {
    let user = db::users::find_by_id(&state.db, auth.user_id).await?;
    Ok(Json(UserPublic::from(user)))
}
