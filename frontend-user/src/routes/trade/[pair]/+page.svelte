<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { page } from '$app/stores';
    import { orderbooks, recentTrades, tickers, availablePairs, circuitOpen } from '$lib/stores';
    import { trading } from '$lib/api';
    import { fmtPrice, fmtQty, fmtEgp, fmtDate, pairToAssets } from '$lib/format';
    import type { Order, Trade } from '$lib/types';
    import OrderBook from '$lib/components/OrderBook.svelte';
    import RecentTrades from '$lib/components/RecentTrades.svelte';
    import PriceChart from '$lib/components/PriceChart.svelte';
    import OrderForm from '$lib/components/OrderForm.svelte';

    let pair = 'BTC_EGP';
    let myOpenOrders: Order[] = [];
    let myRecentTrades: Trade[] = [];
    let loading = true;

    $: pair = $page.params.pair || 'BTC_EGP';
    $: { base, quote } = pairToAssets(pair);
    $: ob = $orderbooks[pair];
    $: rt = $recentTrades[pair] || [];
    $: ticker = $tickers[pair];

    async function refreshOrders() {
        try {
            const [orders, trades] = await Promise.all([trading.listOrders(), trading.listMyTrades()]);
            myOpenOrders = orders.filter((o) => o.pair === pair && (o.status === 'open' || o.status === 'partially_filled')).slice(0, 10);
            myRecentTrades = trades.filter((t) => t.pair === pair).slice(0, 20);
        } catch { /* ignore */ }
    }

    async function cancel(id: string) {
        if (!confirm('Cancel this order?')) return;
        try {
            await trading.cancelOrder(id);
            await refreshOrders();
        } catch (e: any) {
            alert(e.message);
        }
    }

    let timer: ReturnType<typeof setInterval>;
    onMount(() => {
        refreshOrders().finally(() => (loading = false));
        timer = setInterval(refreshOrders, 5000);
    });
    onDestroy(() => clearInterval(timer));

    function handlePlaced() {
        refreshOrders();
    }

    $: pairs = $availablePairs.length ? $availablePairs : ['BTC_EGP', 'ETH_EGP', 'USDT_EGP'];
</script>

<svelte:head><title>Trade {pair.replace('_', '/')} · EGP Exchange</title></svelte:head>

<div class="space-y-4">
    <!-- Pair selector -->
    <div class="card-compact flex items-center justify-between flex-wrap gap-2">
        <div class="flex items-center gap-2 flex-wrap">
            {#each pairs as p}
                <a href="/trade/{p}" class="px-3 py-1.5 rounded-md text-sm font-medium transition-colors {p === pair ? 'bg-base-600 text-text-primary' : 'text-text-secondary hover:bg-base-700/50'}">
                    {p.replace('_', '/')}
                </a>
            {/each}
        </div>
        {#if ticker}
            <div class="flex items-center gap-4 text-sm">
                <div>
                    <span class="text-text-tertiary">Last:</span>
                    <span class="text-text-primary font-mono ml-1">{fmtEgp(ticker.derived_egp_price)}</span>
                </div>
                <div>
                    <span class="text-text-tertiary">Bid:</span>
                    <span class="text-accent-green font-mono ml-1">{fmtEgp(ticker.bid)}</span>
                </div>
                <div>
                    <span class="text-text-tertiary">Ask:</span>
                    <span class="text-accent-red font-mono ml-1">{fmtEgp(ticker.ask)}</span>
                </div>
            </div>
        {/if}
    </div>

    <!-- Main grid -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <!-- Chart (left) -->
        <div class="lg:col-span-6 xl:col-span-7 space-y-4">
            <PriceChart {pair} />

            <!-- My open orders -->
            <div class="card-default">
                <div class="flex items-center justify-between mb-3">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">My Open Orders</h3>
                    <a href="/history" class="text-xs text-accent-blue hover:underline">All orders →</a>
                </div>
                {#if loading}
                    <div class="text-center py-4 text-text-tertiary text-sm">Loading...</div>
                {:else if myOpenOrders.length === 0}
                    <div class="text-center py-4 text-text-tertiary text-sm">No open orders</div>
                {:else}
                    <div class="overflow-x-auto">
                        <table class="table-pro">
                            <thead>
                                <tr><th>Side</th><th>Type</th><th class="text-right">Price</th><th class="text-right">Qty</th><th class="text-right">Filled</th><th class="text-right">Status</th><th></th></tr>
                            </thead>
                            <tbody>
                                {#each myOpenOrders as o}
                                    <tr>
                                        <td><span class={o.side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{o.side}</span></td>
                                        <td class="text-text-secondary">{o.order_type}</td>
                                        <td class="num-cell">{o.price ? fmtPrice(o.price) : '—'}</td>
                                        <td class="num-cell">{fmtQty(o.quantity)}</td>
                                        <td class="num-cell text-text-secondary">{fmtQty(o.filled_quantity)}</td>
                                        <td class="text-right"><span class="pill-warning">{o.status.replace('_', ' ')}</span></td>
                                        <td class="text-right"><button class="text-accent-red text-xs hover:underline" on:click={() => cancel(o.id)}>Cancel</button></td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {/if}
            </div>

            <!-- My recent trades -->
            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">My Recent Trades</h3>
                {#if myRecentTrades.length === 0}
                    <div class="text-center py-4 text-text-tertiary text-sm">No trades yet</div>
                {:else}
                    <div class="overflow-x-auto">
                        <table class="table-pro">
                            <thead>
                                <tr><th>Side</th><th class="text-right">Price</th><th class="text-right">Qty</th><th class="text-right">Fee</th><th class="text-right">Time</th></tr>
                            </thead>
                            <tbody>
                                {#each myRecentTrades as t}
                                    <tr>
                                        <td><span class={t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{t.taker_side}</span></td>
                                        <td class="num-cell">{fmtPrice(t.price)}</td>
                                        <td class="num-cell">{fmtQty(t.quantity)}</td>
                                        <td class="num-cell text-text-secondary">{fmtQty(t.taker_fee)}</td>
                                        <td class="num-cell text-text-tertiary text-xs">{fmtDate(t.executed_at)}</td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {/if}
            </div>
        </div>

        <!-- Right column: Order form + Order book + recent trades -->
        <div class="lg:col-span-6 xl:col-span-5 space-y-4">
            <OrderForm {pair} on:placed={handlePlaced} />
            {#if ob}
                <OrderBook bids={ob.bids} asks={ob.asks} lastPrice={ob.last_price} {pair} />
            {:else}
                <div class="card-default text-center py-12 text-text-tertiary text-sm">Loading order book...</div>
            {/if}
            <RecentTrades trades={rt} {pair} />
        </div>
    </div>
</div>
