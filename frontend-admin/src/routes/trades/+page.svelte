<script lang="ts">
    import { onMount } from 'svelte';
    import { trades } from '$lib/api';
    import { fmtPrice, fmtQty, fmtEgp, fmtDate, fmtRelative } from '$lib/format';
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
</script>

<svelte:head><title>Trades · Admin</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">All Trades</h1>
            <p class="text-sm text-text-secondary mt-1">Most recent 500 trades across all pairs</p>
        </div>
        <div class="text-xs">
            <span class="text-text-tertiary">24h volume:</span>
            <span class="num-cell text-accent-green font-semibold ml-1">{fmtEgp(volume24h.toFixed(2))}</span>
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
                        <th>Trade ID</th>
                        <th>Pair</th>
                        <th>Side</th>
                        <th class="text-right">Price</th>
                        <th class="text-right">Qty</th>
                        <th class="text-right">Value</th>
                        <th class="text-right">Taker Fee</th>
                        <th class="text-right">Maker Fee</th>
                        <th class="text-right">Time</th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as t}
                        <tr>
                            <td class="num-cell text-text-tertiary text-xs">{t.id.slice(0, 8)}…</td>
                            <td class="font-medium">{t.pair.replace('_', '/')}</td>
                            <td><span class={t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{t.taker_side}</span></td>
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
