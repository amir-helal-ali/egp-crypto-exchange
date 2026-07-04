//! User profile endpoints.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;

use crate::auth::AuthUser;
use crate::db;
use crate::error::AppResult;
use crate::models::UserPublic;
use crate::AppState;

pub async fn me(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<UserPublic>> {
    let user = db::users::find_by_id(&state.db, auth.user_id).await?;
    Ok(Json(UserPublic::from(user)))
}
