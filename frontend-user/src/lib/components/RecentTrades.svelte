<script lang="ts">
    import type { Trade } from '$lib/types';
    import { fmtPrice, fmtQty, fmtTime, pairToAssets } from '$lib/format';

    export let trades: Trade[];
    export let pair: string;
    const { base, quote } = pairToAssets(pair);
</script>

<div class="card-default">
    <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">Recent Trades</h3>
        <span class="text-xs text-text-tertiary">{base}/{quote}</span>
    </div>

    <div class="grid grid-cols-3 gap-2 text-xs text-text-tertiary uppercase font-medium mb-1.5 px-1">
        <div>Price ({quote})</div>
        <div class="text-right">Amount ({base})</div>
        <div class="text-right">Time</div>
    </div>

    <div class="max-h-96 overflow-y-auto space-y-0.5">
        {#if trades.length === 0}
            <div class="text-center text-text-tertiary text-sm py-8">No trades yet</div>
        {:else}
            {#each trades.slice(0, 50) as t}
                <div class="grid grid-cols-3 gap-2 px-1 py-0.5 text-sm">
                    <div class="num-cell {t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}">{fmtPrice(t.price)}</div>
                    <div class="num-cell">{fmtQty(t.quantity)}</div>
                    <div class="num-cell text-text-secondary text-xs">{fmtTime(t.executed_at)}</div>
                </div>
            {/each}
        {/if}
    </div>
</div>
