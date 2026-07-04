// Admin frontend types

export type UserRole = 'user' | 'admin';
export type UserStatus = 'active' | 'suspended' | 'banned' | 'pending_kyc';
export type ManualTxType = 'deposit' | 'withdrawal';
export type ManualTxStatus = 'pending' | 'under_review' | 'approved' | 'rejected' | 'completed' | 'failed';
export type OrderStatus = 'open' | 'partially_filled' | 'filled' | 'cancelled' | 'rejected';

export interface UserPublic {
    id: string;
    email: string;
    full_name: string;
    role: UserRole;
    status: UserStatus;
    kyc_level: number;
    country: string;
}

export interface UserFull extends UserPublic {
    phone: string | null;
    password_hash?: string;
    created_at: string;
    updated_at: string;
    last_login_at: string | null;
    failed_logins: number;
    locked_until: string | null;
}

export interface ManualTransaction {
    id: string;
    user_id: string;
    tx_type: ManualTxType;
    asset_class: 'fiat' | 'crypto';
    asset_symbol: string;
    amount: string;
    fee: string;
    status: ManualTxStatus;
    reference: string | null;
    destination: string | null;
    tx_hash: string | null;
    receipt_url: string | null;
    admin_note: string | null;
    reviewed_by: string | null;
    created_at: string;
    reviewed_at: string | null;
    completed_at: string | null;
}

export interface ManualTxWithUser extends ManualTransaction {
    user_email?: string;
    queue_position: number;
}

export interface Order {
    id: string;
    user_id: string;
    pair: string;
    side: 'buy' | 'sell';
    order_type: 'limit' | 'market';
    price: string | null;
    quantity: string;
    filled_quantity: string;
    status: OrderStatus;
    created_at: string;
    updated_at: string;
}

export interface Trade {
    id: string;
    pair: string;
    taker_order_id: string;
    maker_order_id: string;
    taker_user_id: string;
    maker_user_id: string;
    taker_side: 'buy' | 'sell';
    price: string;
    quantity: string;
    taker_fee: string;
    maker_fee: string;
    executed_at: string;
}

export interface LiquidityRow {
    asset: string;
    balance: string;
    locked: string;
    available: string;
}

export interface Overview {
    users: number;
    orders: { total: number; open: number };
    trades: number;
    pending: { deposits: number; withdrawals: number };
    liquidity: Array<{ asset: string; balance: string; locked: string }>;
    circuit_breaker_open: boolean;
}

export interface AuthResponse {
    access_token: string;
    refresh_token: string;
    user: UserPublic;
}

export interface AdminListResponse<T> {
    items?: T[];
    users?: UserFull[];
    total: number;
    offset: number;
    limit: number;
}

export interface AdminReviewRequest {
    status: ManualTxStatus;
    admin_note?: string;
    tx_hash?: string;
}

// --- العملات والأزواج ---
export type CurrencyType = 'fiat' | 'crypto';

export interface Currency {
    id: string;
    symbol: string;
    name: string;
    type: CurrencyType;
    precision: number;
    withdraw_fee: string;
    min_withdrawal: string;
    network: string | null;
    is_active: boolean;
    created_at: string;
    updated_at: string;
}

export interface TradingPair {
    id: string;
    pair: string;
    base_asset: string;
    quote_asset: string;
    binance_symbol: string;
    is_spot_active: boolean;
    is_futures_active: boolean;
    maker_fee_bps: number;
    taker_fee_bps: number;
    min_order_qty: string;
    price_precision: number;
    qty_precision: number;
    sort_order: number;
    is_active: boolean;
    created_at: string;
    updated_at: string;
}
