import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import type { UserPublic, TickerUpdate, OrderBookSnapshot, Trade, CircuitStatus, WsMessage } from './types';

// --- Auth store ------------------------------------------------------------
const emptyUser: UserPublic | null = null;
export const user = writable<UserPublic | null>(emptyUser);
export const accessToken = writable<string>('');
export const isAuthenticated = derived(user, ($u) => $u !== null);

if (browser) {
    const storedToken = localStorage.getItem('access_token');
    const storedUser = localStorage.getItem('user');
    if (storedToken) accessToken.set(storedToken);
    if (storedUser) {
        try { user.set(JSON.parse(storedUser)); } catch { /* ignore */ }
    }

    accessToken.subscribe((v) => {
        if (v) localStorage.setItem('access_token', v);
        else localStorage.removeItem('access_token');
    });
    user.subscribe((v) => {
        if (v) localStorage.setItem('user', JSON.stringify(v));
        else localStorage.removeItem('user');
    });
}

export function setSession(token: string, u: UserPublic) {
    accessToken.set(token);
    user.set(u);
}

export function clearSession() {
    accessToken.set('');
    user.set(null);
    if (browser) {
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        localStorage.removeItem('user');
    }
}

// --- Market data stores ----------------------------------------------------
export const tickers = writable<Record<string, TickerUpdate>>({});
export const orderbooks = writable<Record<string, OrderBookSnapshot>>({});
export const recentTrades = writable<Record<string, Trade[]>>({});
export const circuitOpen = writable<boolean>(false);
export const availablePairs = writable<string[]>(['BTC_EGP', 'ETH_EGP', 'USDT_EGP']);

// --- Live WebSocket connection ---------------------------------------------
let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

export function connectMarketWs() {
    if (!browser) return;
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return;

    const wsBase: string = (import.meta.env.VITE_WS_URL as string) || 'ws://localhost:8080';
    const token = get(accessToken);
    const url = `${wsBase}/api/market/ws${token ? `?token=${encodeURIComponent(token)}` : ''}`;
    try {
        ws = new WebSocket(url);
    } catch (e) {
        scheduleReconnect();
        return;
    }

    ws.onopen = () => console.info('[ws] connected');
    ws.onmessage = (ev) => {
        try {
            const msg: WsMessage = JSON.parse(ev.data as string);
            handleWsMessage(msg);
        } catch (e) {
            console.warn('[ws] failed to parse', e);
        }
    };
    ws.onclose = () => {
        ws = null;
        scheduleReconnect();
    };
    ws.onerror = () => {
        try { ws?.close(); } catch { /* ignore */ }
    };
}

function scheduleReconnect() {
    if (!browser) return;
    if (reconnectTimer) clearTimeout(reconnectTimer);
    reconnectTimer = setTimeout(() => connectMarketWs(), 2000);
}

function handleWsMessage(msg: WsMessage) {
    switch (msg.type) {
        case 'hello':
            circuitOpen.set(msg.circuit_open);
            availablePairs.set(msg.pairs);
            break;
        case 'ticker':
            tickers.update((map) => ({ ...map, [msg.pair]: {
                binance_symbol: msg.binance_symbol,
                bid: msg.bid,
                ask: msg.ask,
                derived_egp_price: msg.derived_egp_price,
                ts: msg.ts,
            }}));
            break;
        case 'orderbook':
            orderbooks.update((m) => ({ ...m, [msg.pair]: msg.snapshot }));
            break;
        case 'trade':
            recentTrades.update((m) => {
                const list = m[msg.pair] || [];
                const next = [{
                    id: '',
                    pair: msg.pair,
                    taker_order_id: '',
                    maker_order_id: '',
                    taker_user_id: '',
                    maker_user_id: '',
                    taker_side: msg.taker_side,
                    price: msg.price,
                    quantity: msg.quantity,
                    taker_fee: '0',
                    maker_fee: '0',
                    executed_at: msg.ts,
                }, ...list].slice(0, 50);
                return { ...m, [msg.pair]: next };
            });
            break;
        case 'circuit_breaker':
            circuitOpen.set(msg.open);
            break;
    }
}

export function disconnectMarketWs() {
    if (reconnectTimer) clearTimeout(reconnectTimer);
    if (ws) {
        ws.onclose = null;
        try { ws.close(); } catch { /* ignore */ }
        ws = null;
    }
}
