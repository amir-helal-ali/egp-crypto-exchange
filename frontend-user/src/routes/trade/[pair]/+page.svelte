<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { page } from '$app/stores';
    import { orderbooks, recentTrades, tickers, availablePairs, circuitOpen, myOrders, myTrades, connectMarketWs } from '$lib/stores';
    import { trading } from '$lib/api';
    import { fmtPrice, fmtQty, fmtEgp, fmtDate, pairToAssets } from '$lib/format';
    import type { Order, Trade } from '$lib/types';
    import OrderBook from '$lib/components/OrderBook.svelte';
    import RecentTrades from '$lib/components/RecentTrades.svelte';
    import CandlestickChart from '$lib/components/CandlestickChart.svelte';
    import OrderForm from '$lib/components/OrderForm.svelte';

    $: pair = $page.params.pair || 'BTC_EGP';
    $: { base, quote } = pairToAssets(pair);
    $: ob = $orderbooks[pair];
    $: rt = $recentTrades[pair] || [];
    $: ticker = $tickers[pair];

    let loading = true;

    onMount(async () => {
        try {
            await connectMarketWs();
        } finally {
            loading = false;
        }
    });

    onDestroy(() => {});

    async function cancel(id: string) {
        if (!confirm('هل تريد إلغاء هذا الأمر؟')) return;
        try {
            await trading.cancelOrder(id);
        } catch (e: any) {
            alert(e.message);
        }
    }

    function handlePlaced() {
        // سيتم تحديث الأوامر تلقائياً عبر WebSocket
    }

    $: pairs = $availablePairs.length ? $availablePairs : ['BTC_EGP', 'ETH_EGP', 'USDT_EGP'];
    $: myOpenOrders = $myOrders
        .filter((o) => o.pair === pair && (o.status === 'open' || o.status === 'partially_filled'))
        .slice(0, 10);
    $: myRecentTradesForPair = $myTrades
        .filter((t) => t.pair === pair)
        .slice(0, 20);
</script>

<svelte:head><title>تداول {pair.replace('_', '/')} · منصة الجنيه</title></svelte:head>

<div class="space-y-4">
    <!-- اختيار الزوج -->
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
                    <span class="text-text-tertiary">آخر سعر:</span>
                    <span class="text-text-primary font-mono mr-1">{fmtEgp(ticker.derived_egp_price)}</span>
                </div>
                <div>
                    <span class="text-text-tertiary">طلب:</span>
                    <span class="text-accent-green font-mono mr-1">{fmtEgp(ticker.bid)}</span>
                </div>
                <div>
                    <span class="text-text-tertiary">عرض:</span>
                    <span class="text-accent-red font-mono mr-1">{fmtEgp(ticker.ask)}</span>
                </div>
            </div>
        {/if}
    </div>

    <!-- الشبكة الرئيسية -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <!-- الرسم البياني (يسار) -->
        <div class="lg:col-span-6 xl:col-span-7 space-y-4">
            <CandlestickChart {pair} />

            <!-- أOrders المفتوحة -->
            <div class="card-default">
                <div class="flex items-center justify-between mb-3">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">أوامري المفتوحة</h3>
                    <a href="/history" class="text-xs text-accent-blue hover:underline">كل الأوامر ←</a>
                </div>
                {#if loading}
                    <div class="text-center py-4 text-text-tertiary text-sm">جارٍ التحميل...</div>
                {:else if myOpenOrders.length === 0}
                    <div class="text-center py-4 text-text-tertiary text-sm">لا توجد أوامر مفتوحة</div>
                {:else}
                    <div class="overflow-x-auto">
                        <table class="table-pro">
                            <thead>
                                <tr>
                                    <th>الجهة</th><th>النوع</th>
                                    <th class="num-cell">السعر</th>
                                    <th class="num-cell">الكمية</th>
                                    <th class="num-cell">المنفذ</th>
                                    <th>الحالة</th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                                {#each myOpenOrders as o}
                                    <tr>
                                        <td><span class={o.side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{o.side === 'buy' ? 'شراء' : 'بيع'}</span></td>
                                        <td class="text-text-secondary">{o.order_type === 'limit' ? 'محدد' : 'سوقي'}</td>
                                        <td class="num-cell">{o.price ? fmtPrice(o.price) : '—'}</td>
                                        <td class="num-cell">{fmtQty(o.quantity)}</td>
                                        <td class="num-cell text-text-secondary">{fmtQty(o.filled_quantity)}</td>
                                        <td><span class="pill-warning">{o.status === 'partially_filled' ? 'منفذ جزئياً' : 'مفتوح'}</span></td>
                                        <td><button class="text-accent-red text-xs hover:underline" on:click={() => cancel(o.id)}>إلغاء</button></td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {/if}
            </div>

            <!-- صفقاتي الأخيرة -->
            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">صفقاتي الأخيرة</h3>
                {#if myRecentTradesForPair.length === 0}
                    <div class="text-center py-4 text-text-tertiary text-sm">لا توجد صفقات بعد</div>
                {:else}
                    <div class="overflow-x-auto">
                        <table class="table-pro">
                            <thead>
                                <tr>
                                    <th>الجهة</th>
                                    <th class="num-cell">السعر</th>
                                    <th class="num-cell">الكمية</th>
                                    <th class="num-cell">الرسوم</th>
                                    <th class="num-cell">الوقت</th>
                                </tr>
                            </thead>
                            <tbody>
                                {#each myRecentTradesForPair as t}
                                    <tr>
                                        <td><span class={t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{t.taker_side === 'buy' ? 'شراء' : 'بيع'}</span></td>
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

        <!-- العمود الأيمن: نموذج الأمر + دفتر الأوامر + آخر الصفقات -->
        <div class="lg:col-span-6 xl:col-span-5 space-y-4">
            <OrderForm {pair} on:placed={handlePlaced} />
            {#if ob}
                <OrderBook bids={ob.bids} asks={ob.asks} lastPrice={ob.last_price} {pair} />
            {:else}
                <div class="card-default text-center py-12 text-text-tertiary text-sm">جارٍ تحميل دفتر الأوامر...</div>
            {/if}
            <RecentTrades trades={rt} {pair} />
        </div>
    </div>
</div>
