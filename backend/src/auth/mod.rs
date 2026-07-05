//! JWT authentication + Axum extractors.

use std::sync::Arc;

use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::HeaderValue;
use axum::response::Response;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::UserRole;

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_ttl_hours: i64,
    pub refresh_ttl_days: i64,
}

impl JwtConfig {
    pub fn new(secret: String, access_ttl_hours: i64, refresh_ttl_days: i64) -> Self {
        Self { secret, access_ttl_hours, refresh_ttl_days }
    }

    fn encode(&self, claims: &Claims) -> AppResult<String> {
        Ok(encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?)
    }

    pub fn issue_pair(&self, user_id: Uuid, role: UserRole) -> AppResult<(String, String)> {
        let now = Utc::now();
        let access_claims = Claims {
            sub: user_id.to_string(),
            role,
            typ: TokenType::Access,
            iat: now.timestamp(),
            exp: (now + Duration::hours(self.access_ttl_hours)).timestamp(),
        };
        let refresh_claims = Claims {
            sub: user_id.to_string(),
            role,
            typ: TokenType::Refresh,
            iat: now.timestamp(),
            exp: (now + Duration::days(self.refresh_ttl_days)).timestamp(),
        };
        Ok((self.encode(&access_claims)?, self.encode(&refresh_claims)?))
    }

    pub fn verify(&self, token: &str) -> AppResult<Claims> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: UserRole,
    pub typ: TokenType,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// Extracts the authenticated user from the `Authorization: Bearer <token>` header.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: UserRole,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<crate::AppState>: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = <Arc<crate::AppState> as axum::extract::FromRef<S>>::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h: &HeaderValue| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("invalid Authorization scheme".into()))?;

        let claims = app_state.jwt.verify(token)?;
        if claims.typ != TokenType::Access {
            return Err(AppError::Unauthorized("expected access token".into()));
        }
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("invalid subject".into()))?;
        Ok(AuthUser { user_id, role: claims.role })
    }
}

/// Extractor that requires an admin role.
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    Arc<crate::AppState>: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        if user.role != UserRole::Admin {
            return Err(AppError::Forbidden("admin role required".into()));
        }
        Ok(AdminUser(user))
    }
}

/// Middleware that ensures the request carries a valid (any-role) JWT.
pub async fn require_auth(req: Request, next: axum::middleware::Next) -> Response {
    next.run(req).await
}

