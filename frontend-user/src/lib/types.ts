// Shared types — match backend models exactly.

export type UserRole = 'user' | 'admin';
export type UserStatus = 'active' | 'suspended' | 'banned' | 'pending_kyc';
export type OrderSide = 'buy' | 'sell';
export type OrderType = 'limit' | 'market';
export type OrderStatus = 'open' | 'partially_filled' | 'filled' | 'cancelled' | 'rejected';
export type ManualTxType = 'deposit' | 'withdrawal';
export type ManualTxStatus = 'pending' | 'under_review' | 'approved' | 'rejected' | 'completed' | 'failed';

export interface UserPublic {
    id: string;
    email: string;
    full_name: string;
    role: UserRole;
    status: UserStatus;
    kyc_level: number;
    country: string;
}

export interface AuthResponse {
    access_token: string;
    refresh_token: string;
    user: UserPublic;
}

export interface Wallet {
    id: string;
    user_id: string;
    asset_symbol: string;
    wallet_type: 'fiat' | 'crypto';
    balance: string;
    locked_balance: string;
    created_at: string;
    updated_at: string;
}

export interface Order {
    id: string;
    user_id: string;
    pair: string;
    side: OrderSide;
    order_type: OrderType;
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

export interface OrderBookLevel {
    price: string;
    quantity: string;
}

export interface OrderBookSnapshot {
    pair: string;
    bids: OrderBookLevel[];
    asks: OrderBookLevel[];
    last_price: string | null;
    circuit_breaker_open: boolean;
}

export interface TickerUpdate {
    binance_symbol: string;
    bid: string;
    ask: string;
    derived_egp_price: string;
    ts: string;
}

export interface TradeTick {
    pair: string;
    price: string;
    quantity: string;
    taker_side: 'buy' | 'sell';
    executed_at: string;
}

export interface QueueEvent {
    manual_tx_id: string;
    user_id: string;
    status: ManualTxStatus;
    queue_position: number;
    ts: string;
}

export interface PlaceOrderRequest {
    pair: string;
    side: OrderSide;
    order_type: OrderType;
    price?: string;
    quantity: string;
}

export interface PlaceOrderResponse {
    order: Order;
    trades: Trade[];
    remaining: string;
}

export interface CircuitStatus {
    open: boolean;
    state: string;
}

export interface WsHello {
    type: 'hello';
    circuit_open: boolean;
    pairs: string[];
}

export interface WsTicker {
    type: 'ticker';
    pair: string;
    binance_symbol: string;
    bid: string;
    ask: string;
    derived_egp_price: string;
    ts: string;
}

export interface WsTrade {
    type: 'trade';
    pair: string;
    price: string;
    quantity: string;
    taker_side: 'buy' | 'sell';
    ts: string;
}

export interface WsOrderbook {
    type: 'orderbook';
    pair: string;
    snapshot: OrderBookSnapshot;
}

export interface WsCircuitBreaker {
    type: 'circuit_breaker';
    open: boolean;
}

export type WsMessage = WsHello | WsTicker | WsTrade | WsOrderbook | WsCircuitBreaker;
