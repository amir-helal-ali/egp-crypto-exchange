<script lang="ts">
    import { onMount } from 'svelte';
    import { trading } from '$lib/api';
    import { fmtPrice, fmtQty, fmtDate, fmtEgp } from '$lib/format';
    import type { Order, Trade } from '$lib/types';

    let orders: Order[] = [];
    let trades: Trade[] = [];
    let loading = true;
    let error = '';
    let tab: 'orders' | 'trades' = 'orders';

    async function load() {
        try {
            [orders, trades] = await Promise.all([trading.listOrders(), trading.listMyTrades()]);
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    async function cancel(id: string) {
        if (!confirm('Cancel this order?')) return;
        try {
            await trading.cancelOrder(id);
            await load();
        } catch (e: any) {
            alert(e.message);
        }
    }
</script>

<svelte:head><title>History · EGP Exchange</title></svelte:head>

<div class="space-y-6">
    <h1 class="text-2xl font-bold text-text-primary">History</h1>
    {#if error}<div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>{/if}

    <div class="border-b border-base-700 mb-2">
        <nav class="flex gap-4">
            <button class="py-2 text-sm font-medium border-b-2 {tab === 'orders' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (tab = 'orders')}>My Orders</button>
            <button class="py-2 text-sm font-medium border-b-2 {tab === 'trades' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (tab = 'trades')}>My Trades</button>
        </nav>
    </div>

    {#if loading}
        <div class="text-center py-8 text-text-tertiary">Loading...</div>
    {:else if tab === 'orders'}
        <div class="card-default overflow-x-auto">
            {#if orders.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">No orders</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>Pair</th><th>Side</th><th>Type</th>
                            <th class="text-right">Price</th>
                            <th class="text-right">Qty</th>
                            <th class="text-right">Filled</th>
                            <th>Status</th>
                            <th class="text-right">Created</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each orders as o}
                            <tr>
                                <td class="font-medium">{o.pair.replace('_', '/')}</td>
                                <td><span class={o.side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{o.side}</span></td>
                                <td class="text-text-secondary">{o.order_type}</td>
                                <td class="num-cell">{o.price ? fmtPrice(o.price) : '—'}</td>
                                <td class="num-cell">{fmtQty(o.quantity)}</td>
                                <td class="num-cell text-text-secondary">{fmtQty(o.filled_quantity)}</td>
                                <td>
                                    {#if o.status === 'open'}<span class="pill-warning">open</span>
                                    {:else if o.status === 'partially_filled'}<span class="pill-info">partial</span>
                                    {:else if o.status === 'filled'}<span class="pill-success">filled</span>
                                    {:else if o.status === 'cancelled'}<span class="pill-muted">cancelled</span>
                                    {:else}<span class="pill-danger">{o.status}</span>{/if}
                                </td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(o.created_at)}</td>
                                <td>
                                    {#if o.status === 'open' || o.status === 'partially_filled'}
                                        <button class="text-accent-red text-xs hover:underline" on:click={() => cancel(o.id)}>Cancel</button>
                                    {/if}
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {:else}
        <div class="card-default overflow-x-auto">
            {#if trades.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">No trades</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>Pair</th><th>Side</th>
                            <th class="text-right">Price</th>
                            <th class="text-right">Qty</th>
                            <th class="text-right">Fee</th>
                            <th class="text-right">Value</th>
                            <th class="text-right">Time</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each trades as t}
                            <tr>
                                <td class="font-medium">{t.pair.replace('_', '/')}</td>
                                <td><span class={t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{t.taker_side}</span></td>
                                <td class="num-cell">{fmtPrice(t.price)}</td>
                                <td class="num-cell">{fmtQty(t.quantity)}</td>
                                <td class="num-cell text-text-secondary">{fmtQty(t.taker_fee)}</td>
                                <td class="num-cell">{fmtEgp((Number(t.price) * Number(t.quantity)).toString())}</td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(t.executed_at)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {/if}
</div>
