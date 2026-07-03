<script lang="ts">
    import { onMount } from 'svelte';
    import { trading } from '$lib/api';
    import { myOrders, myTrades } from '$lib/stores';
    import { fmtPrice, fmtQty, fmtDate, fmtEgp } from '$lib/format';

    let loading = true;
    let error = '';
    let tab: 'orders' | 'trades' = 'orders';

    async function load() {
        try {
            const [orders, trades] = await Promise.all([trading.listOrders(), trading.listMyTrades()]);
            myOrders.set(orders);
            myTrades.set(trades);
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    async function cancel(id: string) {
        if (!confirm('هل تريد إلغاء هذا الأمر؟')) return;
        try {
            await trading.cancelOrder(id);
            // التحديث سيأتي عبر WebSocket
        } catch (e: any) {
            alert(e.message);
        }
    }
</script>

<svelte:head><title>السجل · منصة الجنيه</title></svelte:head>

<div class="space-y-6">
    <h1 class="text-2xl font-bold text-text-primary">السجل</h1>
    {#if error}<div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>{/if}

    <div class="border-b border-base-700 mb-2">
        <nav class="flex gap-4">
            <button class="py-2 text-sm font-medium border-b-2 {tab === 'orders' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (tab = 'orders')}>أوامري</button>
            <button class="py-2 text-sm font-medium border-b-2 {tab === 'trades' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (tab = 'trades')}>صفقاتي</button>
        </nav>
    </div>

    {#if loading}
        <div class="text-center py-8 text-text-tertiary">جارٍ التحميل...</div>
    {:else if tab === 'orders'}
        <div class="card-default overflow-x-auto">
            {#if $myOrders.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">لا توجد أوامر</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الزوج</th>
                            <th>الجهة</th>
                            <th>النوع</th>
                            <th class="num-cell">السعر</th>
                            <th class="num-cell">الكمية</th>
                            <th class="num-cell">المنفذ</th>
                            <th>الحالة</th>
                            <th class="num-cell">تاريخ الإنشاء</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myOrders as o}
                            <tr>
                                <td class="font-bold">{o.pair.replace('_', '/')}</td>
                                <td><span class={o.side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{o.side === 'buy' ? 'شراء' : 'بيع'}</span></td>
                                <td class="text-text-secondary">{o.order_type === 'limit' ? 'محدد' : 'سوقي'}</td>
                                <td class="num-cell">{o.price ? fmtPrice(o.price) : '—'}</td>
                                <td class="num-cell">{fmtQty(o.quantity)}</td>
                                <td class="num-cell text-text-secondary">{fmtQty(o.filled_quantity)}</td>
                                <td>
                                    {#if o.status === 'open'}<span class="pill-warning">مفتوح</span>
                                    {:else if o.status === 'partially_filled'}<span class="pill-info">منفذ جزئياً</span>
                                    {:else if o.status === 'filled'}<span class="pill-success">منفذ بالكامل</span>
                                    {:else if o.status === 'cancelled'}<span class="pill-muted">ملغى</span>
                                    {:else}<span class="pill-danger">{o.status}</span>{/if}
                                </td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(o.created_at)}</td>
                                <td>
                                    {#if o.status === 'open' || o.status === 'partially_filled'}
                                        <button class="text-accent-red text-xs hover:underline" on:click={() => cancel(o.id)}>إلغاء</button>
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
            {#if $myTrades.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">لا توجد صفقات</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الزوج</th>
                            <th>الجهة</th>
                            <th class="num-cell">السعر</th>
                            <th class="num-cell">الكمية</th>
                            <th class="num-cell">الرسوم</th>
                            <th class="num-cell">القيمة</th>
                            <th class="num-cell">الوقت</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myTrades as t}
                            <tr>
                                <td class="font-bold">{t.pair.replace('_', '/')}</td>
                                <td><span class={t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{t.taker_side === 'buy' ? 'شراء' : 'بيع'}</span></td>
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
