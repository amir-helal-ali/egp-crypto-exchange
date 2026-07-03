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

<svelte:head><title>Orders · Admin</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">All Orders</h1>
            <p class="text-sm text-text-secondary mt-1">Most recent 500 orders across all users</p>
        </div>
        <div class="text-xs">
            <span class="pill-warning">{openCount} open</span>
            <span class="ml-2 text-text-tertiary">{items.length} total shown</span>
        </div>
    </div>

    {#if loading}
        <div class="text-center py-8 text-text-tertiary">Loading...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>Order ID</th>
                        <th>User</th>
                        <th>Pair</th>
                        <th>Side</th>
                        <th>Type</th>
                        <th class="text-right">Price</th>
                        <th class="text-right">Qty</th>
                        <th class="text-right">Filled</th>
                        <th>Status</th>
                        <th class="text-right">Created</th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as o}
                        <tr>
                            <td class="num-cell text-text-tertiary text-xs">{o.id.slice(0, 8)}…</td>
                            <td class="num-cell text-text-tertiary text-xs">{o.user_id.slice(0, 8)}…</td>
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
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(o.created_at)}>{fmtRelative(o.created_at)}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
