import type {
    AuthResponse,
    LoginRequest,
    ManualTransaction,
    ManualTxStatus,
    ManualTxType,
    Order,
    OrderBookSnapshot,
    PlaceOrderRequest,
    PlaceOrderResponse,
    RegisterRequest,
    Trade,
    TickerUpdate,
    Wallet,
    Position,
    PositionSide,
    MarginMode,
    P2POffer,
    P2PTrade,
    P2PMessage,
} from './types';
import { browser } from '$app/environment';

const API_BASE: string = (import.meta.env.VITE_API_URL as string) || 'http://localhost:8080';

function authHeaders(): Record<string, string> {
    if (!browser) return { 'Content-Type': 'application/json' };
    const token = localStorage.getItem('access_token');
    return {
        'Content-Type': 'application/json',
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
    };
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
    const res = await fetch(`${API_BASE}${path}`, {
        ...init,
        headers: { ...authHeaders(), ...(init?.headers || {}) },
    });
    if (!res.ok) {
        let detail = res.statusText;
        try {
            const body = await res.json();
            detail = body?.error?.message ?? detail;
        } catch { /* ignore */ }
        throw new ApiError(res.status, detail);
    }
    if (res.status === 204) return undefined as T;
    return (await res.json()) as T;
}

export class ApiError extends Error {
    status: number;
    constructor(status: number, message: string) {
        super(message);
        this.status = status;
    }
}

// --- Auth ------------------------------------------------------------------
export const auth = {
    register: (req: RegisterRequest) =>
        request<AuthResponse>('/api/auth/register', {
            method: 'POST', body: JSON.stringify(req),
        }),
    login: (req: LoginRequest) =>
        request<AuthResponse>('/api/auth/login', {
            method: 'POST', body: JSON.stringify(req),
        }),
    refresh: (refreshToken: string) =>
        request<AuthResponse>('/api/auth/refresh', {
            method: 'POST', body: JSON.stringify({ refresh_token: refreshToken }),
        }),
    me: () => request<{ id: string; email: string; full_name: string; role: string; status: string; kyc_level: number; country: string }>('/api/user/me'),
};

// --- Wallet ----------------------------------------------------------------
export const wallet = {
    list: () => request<Wallet[]>('/api/user/wallets'),
    requestDeposit: (reference: string, amount: string, receiptUrl?: string) =>
        request<ManualTransaction>('/api/user/deposits', {
            method: 'POST',
            body: JSON.stringify({ reference, amount, receipt_url: receiptUrl }),
        }),
    requestWithdrawal: (destination: string, amount: string, assetSymbol: string) =>
        request<ManualTransaction>('/api/user/withdrawals', {
            method: 'POST',
            body: JSON.stringify({ destination, amount, asset_symbol: assetSymbol }),
        }),
    listDeposits: () => request<ManualTransaction[]>('/api/user/deposits'),
    listWithdrawals: () => request<ManualTransaction[]>('/api/user/withdrawals'),
    ledger: (asset?: string) =>
        request<{ wallets: Array<{ asset: string; balance: string; locked: string; ledger: any[] }> }>(
            asset ? `/api/user/ledger/${asset}` : '/api/user/ledger',
        ),
};

// --- Trading ---------------------------------------------------------------
export const trading = {
    listOrders: () => request<Order[]>('/api/user/orders'),
    placeOrder: (req: PlaceOrderRequest) =>
        request<PlaceOrderResponse>('/api/user/orders', {
            method: 'POST', body: JSON.stringify(req),
        }),
    cancelOrder: (id: string) =>
        request<{ cancelled: string }>(`/api/user/orders/${id}`, { method: 'DELETE' }),
    listMyTrades: () => request<Trade[]>('/api/user/trades'),
    orderbook: (pair: string) =>
        request<OrderBookSnapshot>(`/api/market/orderbook/${pair}`),
    tickers: () => request<TickerUpdate[]>('/api/market/tickers'),
    recentTrades: (pair: string, limit = 50) =>
        request<Trade[]>(`/api/market/trades/${pair}?limit=${limit}`),
    circuit: () => request<{ open: boolean; state: string }>('/api/market/circuit'),
};

// --- Manual tx status helpers ----------------------------------------------
export function txStatusPill(status: ManualTxStatus): string {
    switch (status) {
        case 'pending': return 'pill-warning';
        case 'under_review': return 'pill-info';
        case 'approved': return 'pill-info';
        case 'completed': return 'pill-success';
        case 'rejected': return 'pill-danger';
        case 'failed': return 'pill-danger';
        default: return 'pill-muted';
    }
}

export function txTypeLabel(t: ManualTxType): string {
    return t === 'deposit' ? 'إيداع' : 'سحب';
}

// --- العقود الآجلة - Futures -----------------------------------------------
export const futures = {
    listPositions: () => request<Position[]>('/api/futures/positions'),
    openPosition: (req: { pair: string; side: PositionSide; margin_mode: MarginMode; leverage: number; margin: string }) =>
        request<Position>('/api/futures/positions', {
            method: 'POST', body: JSON.stringify(req),
        }),
    closePosition: (id: string) =>
        request<{ closed: string }>(`/api/futures/positions/${id}/close`, { method: 'POST' }),
    setLeverage: (pair: string, leverage: number) =>
        request<{ ok: boolean }>(`/api/futures/leverage`, {
            method: 'POST', body: JSON.stringify({ pair, leverage }),
        }),
};

// --- التداول بين الأفراد - P2P ---------------------------------------------
export const p2p = {
    listOffers: (params: { side?: 'buy' | 'sell'; asset?: string; payment_method?: string } = {}) => {
        const qs = new URLSearchParams();
        if (params.side) qs.set('side', params.side);
        if (params.asset) qs.set('asset', params.asset);
        if (params.payment_method) qs.set('payment_method', params.payment_method);
        const q = qs.toString();
        return request<P2POffer[]>(`/api/p2p/offers${q ? '?' + q : ''}`);
    },
    listMyOffers: () => request<P2POffer[]>('/api/p2p/offers/mine'),
    getOffer: (id: string) => request<P2POffer>(`/api/p2p/offers/${id}`),
    updateOfferStatus: (id: string, status: 'active' | 'paused' | 'closed') =>
        request<P2POffer>(`/api/p2p/offers/${id}/status`, {
            method: 'POST', body: JSON.stringify({ status }),
        }),
    createOffer: (req: {
        side: 'buy' | 'sell';
        asset_symbol: string;
        price_margin_pct: number;
        min_amount_egp: string;
        max_amount_egp: string;
        payment_methods: string[];
        time_limit_min: number;
    }) => request<P2POffer>('/api/p2p/offers', {
        method: 'POST', body: JSON.stringify(req),
    }),
    startTrade: (req: { offer_id: string; amount: string; payment_method: string }) =>
        request<P2PTrade>('/api/p2p/trades', {
            method: 'POST', body: JSON.stringify(req),
        }),
    getTrade: (id: string) => request<P2PTrade>(`/api/p2p/trades/${id}`),
    listMyTrades: () => request<P2PTrade[]>('/api/p2p/trades'),
    confirmPaid: (id: string) =>
        request<P2PTrade>(`/api/p2p/trades/${id}/paid`, { method: 'POST' }),
    releaseCrypto: (id: string) =>
        request<P2PTrade>(`/api/p2p/trades/${id}/release`, { method: 'POST' }),
    cancelTrade: (id: string) =>
        request<P2PTrade>(`/api/p2p/trades/${id}/cancel`, { method: 'POST' }),
    listMessages: (tradeId: string) =>
        request<P2PMessage[]>(`/api/p2p/trades/${tradeId}/messages`),
    sendMessage: (tradeId: string, message: string) =>
        request<P2PMessage>(`/api/p2p/trades/${tradeId}/messages`, {
            method: 'POST', body: JSON.stringify({ message }),
        }),
};

export { API_BASE };
