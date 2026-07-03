//! Domain models that map 1:1 to database tables.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Suspended,
    Banned,
    PendingKyc,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "wallet_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum WalletType {
    Fiat,
    Crypto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "order_side", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "order_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "manual_tx_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ManualTxType {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "manual_tx_asset_class", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ManualTxAssetClass {
    Fiat,
    Crypto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "manual_tx_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ManualTxStatus {
    Pending,
    UnderReview,
    Approved,
    Rejected,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "trade_side", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub phone: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub kyc_level: i16,
    pub country: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_logins: i16,
    pub locked_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub asset_symbol: String,
    pub wallet_type: WalletType,
    pub balance: Decimal,
    pub locked_balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Trade {
    pub id: Uuid,
    pub pair: String,
    pub taker_order_id: Uuid,
    pub maker_order_id: Uuid,
    pub taker_user_id: Uuid,
    pub maker_user_id: Uuid,
    pub taker_side: TradeSide,
    pub price: Decimal,
    pub quantity: Decimal,
    pub taker_fee: Decimal,
    pub maker_fee: Decimal,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ManualTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tx_type: ManualTxType,
    pub asset_class: ManualTxAssetClass,
    pub asset_symbol: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub status: ManualTxStatus,
    pub reference: Option<String>,
    pub destination: Option<String>,
    pub tx_hash: Option<String>,
    pub receipt_url: Option<String>,
    pub admin_note: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WalletLedgerEntry {
    pub id: i64,
    pub wallet_id: Uuid,
    pub user_id: Uuid,
    pub delta: Decimal,
    pub balance_after: Decimal,
    pub reason: String,
    pub ref_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SystemLiquidity {
    pub asset_symbol: String,
    pub total_balance: Decimal,
    pub total_locked: Decimal,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Request/Response DTOs (used by the API layer)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, validator::Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 1, max = 200))]
    pub full_name: String,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserPublic,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub kyc_level: i16,
    pub country: String,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            role: u.role,
            status: u.status,
            kyc_level: u.kyc_level,
            country: u.country,
        }
    }
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct PlaceOrderRequest {
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<Decimal>,
    #[validate(custom(function = "validate_positive"))]
    pub quantity: Decimal,
}

fn validate_positive(v: &Decimal) -> Result<(), validator::ValidationError> {
    if *v <= Decimal::ZERO {
        return Err(validator::ValidationError::new("must be positive"));
    }
    Ok(())
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct ManualDepositRequest {
    #[validate(length(min = 1, max = 64))]
    pub reference: String,
    #[validate(custom(function = "validate_positive"))]
    pub amount: Decimal,
    pub receipt_url: Option<String>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CryptoWithdrawalRequest {
    #[validate(length(min = 4, max = 256))]
    pub destination: String,
    #[validate(custom(function = "validate_positive"))]
    pub amount: Decimal,
    pub asset_symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminReviewRequest {
    pub status: ManualTxStatus, // Approved | Rejected | Completed | Failed | UnderReview
    pub admin_note: Option<String>,
    pub tx_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserStatusRequest {
    pub status: UserStatus,
    pub kyc_level: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct OrderBookSnapshot {
    pub pair: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub last_price: Option<Decimal>,
    pub circuit_breaker_open: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderBookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Serialize, Clone)]
pub struct TradeTick {
    pub pair: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub taker_side: TradeSide,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TickerUpdate {
    pub binance_symbol: String,
    pub bid: Decimal,
    pub ask: Decimal,
    pub derived_egp_price: Decimal,
    pub ts: DateTime<Utc>,
}
