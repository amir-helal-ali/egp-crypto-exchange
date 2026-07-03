<script lang="ts">
    import { onMount } from 'svelte';
    import { orders } from '$lib/api';
    import { fmtPrice, fmtQty, fmtDate, fmtRelative } from '$lib/format';
    import type { Order } from '$lib/types';

    let items: Order[] = [];
    let loading = true;
    let error = '';

    async function load() {
        try {
            items = await orders.list(500);
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    $: openCount = items.filter((o) => o.status === 'open' || o.status === 'partially_filled').length;
</script>

<svelte:head><title>الأوامر · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">كل الأوامر</h1>
            <p class="text-sm text-text-secondary mt-1">أحدث 500 أمر عبر كل المستخدمين</p>
        </div>
        <div class="text-xs">
            <span class="pill-warning">{openCount} مفتوح</span>
            <span class="ml-2 text-text-tertiary">{items.length} إجمالي</span>
        </div>
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
                        <th>معرف الأمر</th>
                        <th>المستخدم</th>
                        <th>الزوج</th>
                        <th>الجهة</th>
                        <th>النوع</th>
                        <th class="num-cell">السعر</th>
                        <th class="num-cell">الكمية</th>
                        <th class="num-cell">المنفذ</th>
                        <th>الحالة</th>
                        <th class="num-cell">منذ</th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as o}
                        <tr>
                            <td class="num-cell text-text-tertiary text-xs">{o.id.slice(0, 8)}…</td>
                            <td class="num-cell text-text-tertiary text-xs">{o.user_id.slice(0, 8)}…</td>
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
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(o.created_at)}>{fmtRelative(o.created_at)}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
