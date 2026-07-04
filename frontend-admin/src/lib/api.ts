import { browser } from '$app/environment';
import type {
    AdminListResponse, AuthResponse, LiquidityRow, ManualTransaction, ManualTxStatus,
    ManualTxType, Order, Overview, Trade, UserFull, AdminReviewRequest,
} from './types';

const API_BASE: string = (import.meta.env.VITE_API_URL as string) || 'http://localhost:8080';

function authHeaders(): Record<string, string> {
    if (!browser) return { 'Content-Type': 'application/json' };
    const token = localStorage.getItem('admin_access_token');
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
    login: (email: string, password: string) =>
        request<AuthResponse>('/api/auth/login', {
            method: 'POST', body: JSON.stringify({ email, password }),
        }),
};

// --- Overview --------------------------------------------------------------
export const overview = {
    get: () => request<Overview>('/api/admin/overview'),
    liquidity: () => request<{ liquidity: LiquidityRow[] }>('/api/admin/liquidity'),
};

// --- Users -----------------------------------------------------------------
export const users = {
    list: (params: { offset?: number; limit?: number; status?: string } = {}) => {
        const qs = new URLSearchParams();
        if (params.offset) qs.set('offset', String(params.offset));
        if (params.limit) qs.set('limit', String(params.limit));
        if (params.status) qs.set('status', params.status);
        const q = qs.toString();
        return request<AdminListResponse<UserFull>>(`/api/admin/users${q ? '?' + q : ''}`);
    },
    get: (id: string) => request<UserFull>(`/api/admin/users/${id}`),
    updateStatus: (id: string, status: string, kycLevel?: number) =>
        request<UserFull>(`/api/admin/users/${id}/status`, {
            method: 'PUT',
            body: JSON.stringify({ status, kyc_level: kycLevel }),
        }),
};

// --- Manual transactions ---------------------------------------------------
export const manualTx = {
    list: (params: { tx_type?: ManualTxType; status?: ManualTxStatus; offset?: number; limit?: number } = {}) => {
        const qs = new URLSearchParams();
        if (params.tx_type) qs.set('tx_type', params.tx_type);
        if (params.status) qs.set('status', params.status);
        if (params.offset) qs.set('offset', String(params.offset));
        if (params.limit) qs.set('limit', String(params.limit));
        const q = qs.toString();
        return request<{ items: Array<{ tx: ManualTransaction; queue_position: number }>; total: number; offset: number; limit: number }>(
            `/api/admin/manual_tx${q ? '?' + q : ''}`,
        );
    },
    get: (id: string) => request<ManualTransaction>(`/api/admin/manual_tx/${id}`),
    review: (id: string, req: AdminReviewRequest) =>
        request<ManualTransaction>(`/api/admin/manual_tx/${id}/review`, {
            method: 'POST',
            body: JSON.stringify(req),
        }),
};

// --- Orders / Trades / Audit ----------------------------------------------
export const orders = {
    list: (limit = 200) => request<Order[]>(`/api/admin/orders?limit=${limit}`),
};

export const trades = {
    list: (limit = 200) => request<Trade[]>(`/api/admin/trades?limit=${limit}`),
};

export const audit = {
    list: (limit = 100, offset = 0) =>
        request<{ items: any[] }>(`/api/admin/audit?limit=${limit}&offset=${offset}`),
};

// --- Helpers ---------------------------------------------------------------
export function statusPill(status: ManualTxStatus): string {
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

export { API_BASE };
