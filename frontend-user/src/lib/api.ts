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
    return t === 'deposit' ? 'Deposit' : 'Withdrawal';
}

export { API_BASE };
