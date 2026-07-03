<script lang="ts">
    import type { OrderBookLevel } from '$lib/types';
    import { fmtPrice, fmtQty, pairToAssets } from '$lib/format';

    export let bids: OrderBookLevel[];
    export let asks: OrderBookLevel[];
    export let lastPrice: string | null;
    export let pair: string;
    export let maxDepth = 12;

    const { base, quote } = pairToAssets(pair);

    $: topBids = bids.slice(0, maxDepth);
    $: topAsks = asks.slice(0, maxDepth).reverse();
    $: maxTotal = Math.max(
        ...topBids.map((b) => Number(b.quantity)),
        ...topAsks.map((a) => Number(a.quantity)),
        1,
    );
</script>

<div class="card-default">
    <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">دفتر الأوامر</h3>
        <span class="text-xs text-text-tertiary">{base}/{quote}</span>
    </div>

    <div class="grid grid-cols-3 gap-2 text-xs text-text-tertiary uppercase font-medium mb-1.5 px-1">
        <div>السعر ({quote})</div>
        <div class="num-cell">الكمية ({base})</div>
        <div class="num-cell">الإجمالي</div>
    </div>

    <!-- عروض البيع (حمراء) -->
    <div class="space-y-0.5">
        {#each topAsks as level}
            <div class="relative grid grid-cols-3 gap-2 px-1 py-0.5 text-sm">
                <div class="absolute inset-y-0 left-0 bg-accent-red/10" style="width: {(Number(level.quantity) / maxTotal) * 100}%"></div>
                <div class="relative num-cell text-accent-red">{fmtPrice(level.price)}</div>
                <div class="relative num-cell">{fmtQty(level.quantity)}</div>
                <div class="relative num-cell text-text-secondary">{fmtQty((Number(level.price) * Number(level.quantity)).toString())}</div>
            </div>
        {/each}
    </div>

    <!-- آخر سعر -->
    <div class="my-2 py-1.5 border-y border-base-700 flex items-center justify-between">
        <div class="text-xs text-text-tertiary">آخر سعر</div>
        <div class="text-base font-mono font-semibold {lastPrice ? 'text-text-primary' : 'text-text-tertiary'}">
            {fmtPrice(lastPrice)} <span class="text-xs text-text-tertiary">{quote}</span>
        </div>
    </div>

    <!-- عروض الشراء (خضراء) -->
    <div class="space-y-0.5">
        {#each topBids as level}
            <div class="relative grid grid-cols-3 gap-2 px-1 py-0.5 text-sm">
                <div class="absolute inset-y-0 left-0 bg-accent-green/10" style="width: {(Number(level.quantity) / maxTotal) * 100}%"></div>
                <div class="relative num-cell text-accent-green">{fmtPrice(level.price)}</div>
                <div class="relative num-cell">{fmtQty(level.quantity)}</div>
                <div class="relative num-cell text-text-secondary">{fmtQty((Number(level.price) * Number(level.quantity)).toString())}</div>
            </div>
        {/each}
    </div>
</div>
