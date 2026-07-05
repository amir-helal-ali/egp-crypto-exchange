<script lang="ts">
    import { onMount } from 'svelte';
    import { trades } from '$lib/api';
    import { fmtPrice, fmtQty, fmtEgp, fmtDate, fmtRelative } from '$lib/format';
    import { exportToCSV } from '$lib/csv';
    import type { Trade } from '$lib/types';

    let items: Trade[] = [];
    let loading = true;
    let error = '';

    async function load() {
        try {
            items = await trades.list(500);
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    $: volume24h = items
        .filter((t) => Date.now() - new Date(t.executed_at).getTime() < 86400000)
        .reduce((sum, t) => sum + Number(t.price) * Number(t.quantity), 0);

    function exportCSV() {
        exportToCSV('الصفقات', [
            'المعرف', 'الزوج', 'الجهة', 'السعر', 'الكمية', 'القيمة', 'رسوم المنفذ', 'رسوم الصانع', 'الوقت'
        ], items.map((t) => [
            t.id, t.pair, t.taker_side, t.price, t.quantity,
            (Number(t.price) * Number(t.quantity)).toFixed(2),
            t.taker_fee, t.maker_fee,
            new Date(t.executed_at).toLocaleString('ar-EG')
        ]));
    }
</script>

<svelte:head><title>الصفقات · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">كل الصفقات</h1>
            <p class="text-sm text-text-secondary mt-1">أحدث 500 صفقة عبر كل الأزواج</p>
        </div>
        <div class="text-xs">
            <span class="text-text-tertiary">حجم 24 ساعة:</span>
            <span class="num-cell text-accent-green font-semibold mr-1">{fmtEgp(volume24h.toFixed(2))}</span>
        </div>
        <button class="btn-ghost text-xs" on:click={exportCSV} disabled={items.length === 0}>
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" /></svg>
            تصدير CSV
        </button>
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>الصفقة</th>
                        <th>الزوج</th>
                        <th>الجهة</th>
                        <th class="num-cell">السعر</th>
                        <th class="num-cell">الكمية</th>
                        <th class="num-cell">القيمة</th>
                        <th class="num-cell">رسوم المنفذ</th>
                        <th class="num-cell">رسوم الصانع</th>
                        <th class="num-cell">الوقت</th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as t}
                        <tr>
                            <td class="num-cell text-text-tertiary text-xs">{t.id.slice(0, 8)}…</td>
                            <td class="font-bold">{t.pair.replace('_', '/')}</td>
                            <td><span class={t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{t.taker_side === 'buy' ? 'شراء' : 'بيع'}</span></td>
                            <td class="num-cell">{fmtPrice(t.price)}</td>
                            <td class="num-cell">{fmtQty(t.quantity)}</td>
                            <td class="num-cell">{fmtEgp((Number(t.price) * Number(t.quantity)).toString())}</td>
                            <td class="num-cell text-text-secondary">{fmtQty(t.taker_fee)}</td>
                            <td class="num-cell text-text-secondary">{fmtQty(t.maker_fee)}</td>
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(t.executed_at)}>{fmtRelative(t.executed_at)}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
