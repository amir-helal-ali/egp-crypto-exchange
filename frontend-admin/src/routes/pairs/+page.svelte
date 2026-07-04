<script lang="ts">
    import { onMount } from 'svelte';
    import { ApiError } from '$lib/api';
    import { fmtQty } from '$lib/format';
    import type { TradingPair } from '$lib/types';

    let items: TradingPair[] = [];
    let loading = true;
    let error = '';
    let showForm = false;
    let editing: TradingPair | null = null;

    let pair = '';
    let baseAsset = '';
    let quoteAsset = '';
    let binanceSymbol = '';
    let isSpotActive = true;
    let isFuturesActive = false;
    let makerFeeBps = 10;
    let takerFeeBps = 20;
    let minOrderQty = '0.0001';
    let pricePrecision = 2;
    let qtyPrecision = 8;
    let sortOrder = 0;
    let isActive = true;
    let saving = false;

    const API_BASE: string = (import.meta.env.VITE_API_URL as string) || 'http://localhost:8080';

    function authHeaders(): Record<string, string> {
        const token = typeof localStorage !== 'undefined' ? localStorage.getItem('admin_access_token') : null;
        return {
            'Content-Type': 'application/json',
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
        };
    }

    async function apiCall<T>(path: string, init?: RequestInit): Promise<T> {
        const res = await fetch(`${API_BASE}${path}`, {
            ...init,
            headers: { ...authHeaders(), ...(init?.headers || {}) },
        });
        if (!res.ok) {
            const body = await res.json().catch(() => ({}));
            throw new ApiError(res.status, body?.error?.message || res.statusText);
        }
        return res.json();
    }

    async function load() {
        try {
            items = await apiCall<TradingPair[]>('/api/admin/pairs');
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    function resetForm() {
        pair = '';
        baseAsset = '';
        quoteAsset = '';
        binanceSymbol = '';
        isSpotActive = true;
        isFuturesActive = false;
        makerFeeBps = 10;
        takerFeeBps = 20;
        minOrderQty = '0.0001';
        pricePrecision = 2;
        qtyPrecision = 8;
        sortOrder = 0;
        isActive = true;
        editing = null;
    }

    function openCreate() {
        resetForm();
        showForm = true;
    }

    function openEdit(p: TradingPair) {
        editing = p;
        pair = p.pair;
        baseAsset = p.base_asset;
        quoteAsset = p.quote_asset;
        binanceSymbol = p.binance_symbol;
        isSpotActive = p.is_spot_active;
        isFuturesActive = p.is_futures_active;
        makerFeeBps = p.maker_fee_bps;
        takerFeeBps = p.taker_fee_bps;
        minOrderQty = p.min_order_qty;
        pricePrecision = p.price_precision;
        qtyPrecision = p.qty_precision;
        sortOrder = p.sort_order;
        isActive = p.is_active;
        showForm = true;
    }

    async function save() {
        saving = true;
        error = '';
        try {
            if (editing) {
                await apiCall(`/api/admin/pairs/${editing.id}`, {
                    method: 'PUT',
                    body: JSON.stringify({
                        binance_symbol: binanceSymbol,
                        is_spot_active: isSpotActive,
                        is_futures_active: isFuturesActive,
                        maker_fee_bps: makerFeeBps,
                        taker_fee_bps: takerFeeBps,
                        min_order_qty: minOrderQty,
                        price_precision: pricePrecision,
                        qty_precision: qtyPrecision,
                        sort_order: sortOrder,
                        is_active: isActive,
                    }),
                });
            } else {
                await apiCall('/api/admin/pairs', {
                    method: 'POST',
                    body: JSON.stringify({
                        pair,
                        base_asset: baseAsset,
                        quote_asset: quoteAsset,
                        binance_symbol: binanceSymbol,
                        is_spot_active: isSpotActive,
                        is_futures_active: isFuturesActive,
                        maker_fee_bps: makerFeeBps,
                        taker_fee_bps: takerFeeBps,
                        min_order_qty: minOrderQty,
                        price_precision: pricePrecision,
                        qty_precision: qtyPrecision,
                        sort_order: sortOrder,
                    }),
                });
            }
            showForm = false;
            resetForm();
            await load();
        } catch (e: any) {
            error = e.message;
        } finally {
            saving = false;
        }
    }

    async function remove(p: TradingPair) {
        if (!confirm(`حذف زوج ${p.pair}؟`)) return;
        try {
            await apiCall(`/api/admin/pairs/${p.id}`, { method: 'DELETE' });
            await load();
        } catch (e: any) {
            alert(e.message);
        }
    }
</script>

<svelte:head><title>أزواج التداول · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">أزواج التداول</h1>
            <p class="text-sm text-text-secondary mt-1">التحكم الكامل في أزواج التداول والرسوم</p>
        </div>
        <button class="btn-primary text-sm" on:click={openCreate}>+ إضافة زوج</button>
    </div>

    {#if error}<div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>{/if}

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>الزوج</th>
                        <th>الأساس</th>
                        <th>التسعير</th>
                        <th>رمز بينانس</th>
                        <th>سبوت</th>
                        <th>عقود</th>
                        <th class="num-cell">رسوم الصانع</th>
                        <th class="num-cell">رسوم المنفذ</th>
                        <th>الحالة</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as p}
                        <tr>
                            <td class="font-bold">{p.pair.replace('_', '/')}</td>
                            <td>{p.base_asset}</td>
                            <td>{p.quote_asset}</td>
                            <td class="text-text-secondary num-cell text-xs">{p.binance_symbol}</td>
                            <td>{#if p.is_spot_active}<span class="pill-success">نعم</span>{:else}<span class="pill-muted">لا</span>{/if}</td>
                            <td>{#if p.is_futures_active}<span class="pill-success">نعم</span>{:else}<span class="pill-muted">لا</span>{/if}</td>
                            <td class="num-cell">{p.maker_fee_bps} bps</td>
                            <td class="num-cell">{p.taker_fee_bps} bps</td>
                            <td>{#if p.is_active}<span class="pill-success">نشط</span>{:else}<span class="pill-danger">معطّل</span>{/if}</td>
                            <td>
                                <div class="flex gap-2">
                                    <button class="text-xs text-accent-blue hover:underline" on:click={() => openEdit(p)}>تعديل</button>
                                    <button class="text-xs text-accent-red hover:underline" on:click={() => remove(p)}>حذف</button>
                                </div>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

{#if showForm}
    <div class="fixed inset-0 z-50 bg-base-900/70 flex items-center justify-center p-4" on:click|self={() => (showForm = false)}>
        <div class="bg-base-800 border border-base-600 rounded-lg w-full max-w-lg p-6 max-h-[90vh] overflow-y-auto">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold text-text-primary">{editing ? 'تعديل زوج' : 'إضافة زوج'}</h3>
                <button on:click={() => (showForm = false)} class="text-text-tertiary hover:text-text-primary">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>
                </button>
            </div>

            <div class="space-y-3 text-sm">
                <div class="grid grid-cols-3 gap-3">
                    <div>
                        <label class="label" for="pair">الزوج</label>
                        <input id="pair" type="text" bind:value={pair} class="input" disabled={!!editing} placeholder="BTC_EGP" dir="ltr" />
                    </div>
                    <div>
                        <label class="label" for="base">الأساس</label>
                        <input id="base" type="text" bind:value={baseAsset} class="input" disabled={!!editing} placeholder="BTC" dir="ltr" />
                    </div>
                    <div>
                        <label class="label" for="quote">التسعير</label>
                        <input id="quote" type="text" bind:value={quoteAsset} class="input" disabled={!!editing} placeholder="EGP" dir="ltr" />
                    </div>
                </div>
                <div>
                    <label class="label" for="bsym">رمز بينانس</label>
                    <input id="bsym" type="text" bind:value={binanceSymbol} class="input" placeholder="BTCUSDT" dir="ltr" />
                </div>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="label" for="mb">رسوم الصانع (bps)</label>
                        <input id="mb" type="number" min="0" max="1000" bind:value={makerFeeBps} class="input" dir="ltr" />
                    </div>
                    <div>
                        <label class="label" for="tb">رسوم المنفذ (bps)</label>
                        <input id="tb" type="number" min="0" max="1000" bind:value={takerFeeBps} class="input" dir="ltr" />
                    </div>
                </div>
                <div class="grid grid-cols-3 gap-3">
                    <div>
                        <label class="label" for="mq">أقل كمية</label>
                        <input id="mq" type="number" step="any" bind:value={minOrderQty} class="input" dir="ltr" />
                    </div>
                    <div>
                        <label class="label" for="pp">دقة السعر</label>
                        <input id="pp" type="number" min="0" max="8" bind:value={pricePrecision} class="input" dir="ltr" />
                    </div>
                    <div>
                        <label class="label" for="qp">دقة الكمية</label>
                        <input id="qp" type="number" min="0" max="8" bind:value={qtyPrecision} class="input" dir="ltr" />
                    </div>
                </div>
                <div>
                    <label class="label" for="so">ترتيب العرض</label>
                    <input id="so" type="number" min="0" bind:value={sortOrder} class="input" dir="ltr" />
                </div>
                <div class="grid grid-cols-3 gap-3 mt-2">
                    <label class="flex items-center gap-2">
                        <input type="checkbox" bind:checked={isSpotActive} class="accent-accent-blue" />
                        <span class="text-text-secondary text-xs">سبوت</span>
                    </label>
                    <label class="flex items-center gap-2">
                        <input type="checkbox" bind:checked={isFuturesActive} class="accent-accent-blue" />
                        <span class="text-text-secondary text-xs">عقود آجلة</span>
                    </label>
                    <label class="flex items-center gap-2">
                        <input type="checkbox" bind:checked={isActive} class="accent-accent-blue" />
                        <span class="text-text-secondary text-xs">نشط</span>
                    </label>
                </div>

                {#if error}<div class="text-xs text-accent-red">{error}</div>{/if}

                <div class="flex gap-2 pt-2">
                    <button class="btn-ghost flex-1" on:click={() => (showForm = false)}>إلغاء</button>
                    <button class="btn-primary flex-1" on:click={save} disabled={saving}>
                        {saving ? 'جارٍ الحفظ...' : 'حفظ'}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}
