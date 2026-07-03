import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import type { UserPublic, TickerUpdate, OrderBookSnapshot, Trade, CircuitStatus, WsMessage, Order, Wallet, ManualTransaction, Position, P2POffer } from './types';

// --- متاجر المصادقة ---
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

// --- متاجر بيانات السوق (تتحدث عبر WebSocket فقط) ---
export const tickers = writable<Record<string, TickerUpdate>>({});
export const orderbooks = writable<Record<string, OrderBookSnapshot>>({});
export const recentTrades = writable<Record<string, Trade[]>>({});
export const circuitOpen = writable<boolean>(false);
export const availablePairs = writable<string[]>(['BTC_EGP', 'ETH_EGP', 'USDT_EGP']);

// --- متاجر بيانات المستخدم (تتحدث عبر WebSocket فقط) ---
export const myOrders = writable<Order[]>([]);
export const myWallets = writable<Wallet[]>([]);
export const myTrades = writable<Trade[]>([]);
export const myDeposits = writable<ManualTransaction[]>([]);
export const myWithdrawals = writable<ManualTransaction[]>([]);
export const myPositions = writable<Position[]>([]);
export const myP2PTrades = writable<any[]>([]);

// --- متاجر العقود الآجلة ---
export const futuresMarkPrice = writable<Record<string, string>>({});
export const fundingRate = writable<Record<string, string>>({});

// --- متجر عروض P2P ---
export const p2pOffers = writable<P2POffer[]>([]);

// --- اتصال WebSocket المركزي (يمنع استخدام polling تماماً) ---
let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let messageHandlers: Array<(msg: WsMessage) => void> = [];

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

    ws.onopen = () => {
        console.info('[WS] تم الاتصال بالخادم');
        // أعد تحميل البيانات الأولية عند إعادة الاتصال
        refreshInitialData();
    };
    ws.onmessage = (ev) => {
        try {
            const msg: WsMessage = JSON.parse(ev.data as string);
            handleWsMessage(msg);
            messageHandlers.forEach((h) => h(msg));
        } catch (e) {
            console.warn('[WS] فشل في تحليل الرسالة', e);
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

async function refreshInitialData() {
    if (!browser) return;
    const token = get(accessToken);
    if (!token) return;
    try {
        const { trading, wallet } = await import('$lib/api');
        const [orders, trades, wallets, deps, wds] = await Promise.all([
            trading.listOrders(),
            trading.listMyTrades(),
            wallet.list(),
            wallet.listDeposits(),
            wallet.listWithdrawals(),
        ]);
        myOrders.set(orders);
        myTrades.set(trades);
        myWallets.set(wallets);
        myDeposits.set(deps);
        myWithdrawals.set(wds);
    } catch (e) {
        console.warn('فشل تحميل البيانات الأولية', e);
    }
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
        case 'order_update':
            // تحديث أمر المستخدم لحظياً
            myOrders.update((orders) => {
                const idx = orders.findIndex((o) => o.id === (msg as any).order.id);
                if (idx >= 0) {
                    const next = [...orders];
                    next[idx] = (msg as any).order;
                    return next;
                }
                return [(msg as any).order, ...orders].slice(0, 100);
            });
            break;
        case 'wallet_update':
            // تحديث رصيد المستخدم لحظياً
            myWallets.update((wallets) => {
                const idx = wallets.findIndex((w) => w.id === (msg as any).wallet.id);
                if (idx >= 0) {
                    const next = [...wallets];
                    next[idx] = (msg as any).wallet;
                    return next;
                }
                return [...wallets, (msg as any).wallet];
            });
            break;
        case 'manual_tx_update':
            // تحديث طلب الإيداع/السحب لحظياً
            const tx = (msg as any).tx;
            if (tx.tx_type === 'deposit') {
                myDeposits.update((list) => {
                    const idx = list.findIndex((t) => t.id === tx.id);
                    if (idx >= 0) {
                        const next = [...list];
                        next[idx] = tx;
                        return next;
                    }
                    return [tx, ...list];
                });
            } else {
                myWithdrawals.update((list) => {
                    const idx = list.findIndex((t) => t.id === tx.id);
                    if (idx >= 0) {
                        const next = [...list];
                        next[idx] = tx;
                        return next;
                    }
                    return [tx, ...list];
                });
            }
            break;
        case 'position_update':
            // تحديث مركز العقود الآجلة لحظياً
            myPositions.update((positions) => {
                const idx = positions.findIndex((p) => p.id === (msg as any).position.id);
                if (idx >= 0) {
                    const next = [...positions];
                    next[idx] = (msg as any).position;
                    return next;
                }
                return [...positions, (msg as any).position];
            });
            break;
        case 'p2p_offer_update':
            // تحديث عرض P2P
            p2pOffers.update((offers) => {
                const idx = offers.findIndex((o) => o.id === (msg as any).offer.id);
                if (idx >= 0) {
                    const next = [...offers];
                    next[idx] = (msg as any).offer;
                    return next;
                }
                return [(msg as any).offer, ...offers];
            });
            break;
    }
}

export function subscribeToMessages(handler: (msg: WsMessage) => void): () => void {
    messageHandlers.push(handler);
    return () => {
        messageHandlers = messageHandlers.filter((h) => h !== handler);
    };
}

export function disconnectMarketWs() {
    if (reconnectTimer) clearTimeout(reconnectTimer);
    if (ws) {
        ws.onclose = null;
        try { ws.close(); } catch { /* ignore */ }
        ws = null;
    }
}
