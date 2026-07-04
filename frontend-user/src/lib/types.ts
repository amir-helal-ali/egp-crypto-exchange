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

// --- رسائل WebSocket إضافية لحظية (بدون polling) ---
export interface WsOrderUpdate {
    type: 'order_update';
    order: Order;
}
export interface WsWalletUpdate {
    type: 'wallet_update';
    wallet: Wallet;
}
export interface WsManualTxUpdate {
    type: 'manual_tx_update';
    tx: ManualTransaction;
}
export interface WsPositionUpdate {
    type: 'position_update';
    position: Position;
}
export interface WsP2POfferUpdate {
    type: 'p2p_offer_update';
    offer: P2POffer;
}
export interface WsP2PTradeUpdate {
    type: 'p2p_trade_update';
    trade: P2PTrade;
}
export interface WsP2PMessage {
    type: 'p2p_message';
    trade_id: string;
    message: P2PMessage;
}
export interface WsFuturesTick {
    type: 'futures_tick';
    pair: string;
    mark_price: string;
    funding_rate: string;
    next_funding: string;
}

export type WsMessage = WsHello | WsTicker | WsTrade | WsOrderbook | WsCircuitBreaker
    | WsOrderUpdate | WsWalletUpdate | WsManualTxUpdate | WsPositionUpdate
    | WsP2POfferUpdate | WsP2PTradeUpdate | WsP2PMessage | WsFuturesTick;

// --- العقود الآجلة ---
export type PositionSide = 'long' | 'short';
export type MarginMode = 'isolated' | 'cross';

export interface Position {
    id: string;
    user_id: string;
    pair: string;
    side: PositionSide;
    margin_mode: MarginMode;
    leverage: number;
    margin: string;
    quantity: string;
    entry_price: string;
    mark_price: string;
    liquidation_price: string;
    unrealized_pnl: string;
    realized_pnl: string;
    status: 'open' | 'closed' | 'liquidated';
    created_at: string;
    closed_at: string | null;
}

// --- P2P ---
export type P2PSide = 'buy' | 'sell';
export type P2POfferStatus = 'active' | 'paused' | 'closed';
export type P2PTradeStatus = 'pending' | 'paid' | 'released' | 'cancelled' | 'disputed' | 'completed';

export interface P2POffer {
    id: string;
    user_id: string;
    side: P2PSide;          // buy = شراء بالجنيه، sell = بيع مقابل جنيه
    asset_symbol: string;
    price_margin_pct: number;
    min_amount_egp: string;
    max_amount_egp: string;
    payment_methods: string[];
    time_limit_min: number;
    status: P2POfferStatus;
    total_trades: number;
    completion_rate: number;
    avg_release_min: number;
    created_at: string;
    user_email?: string;
}

export interface P2PTrade {
    id: string;
    offer_id: string;
    buyer_id: string;
    seller_id: string;
    asset_symbol: string;
    amount: string;
    price_egp: string;
    total_egp: string;
    payment_method: string;
    status: P2PTradeStatus;
    created_at: string;
    paid_at: string | null;
    released_at: string | null;
    cancelled_at: string | null;
    escrow_locked: boolean;
}

export interface P2PMessage {
    id: string;
    trade_id: string;
    sender_id: string;
    message: string;
    created_at: string;
}
