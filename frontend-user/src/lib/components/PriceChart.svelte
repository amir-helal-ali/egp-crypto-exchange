<script lang="ts">
    import { onMount } from 'svelte';
    import { tickers, circuitOpen } from '$lib/stores';
    import { fmtEgp, fmtPrice } from '$lib/format';
    import type { TickerUpdate } from '$lib/types';

    export let pair: string;

    // Lightweight canvas chart — renders price sparkline from accumulated ticks.
    let canvas: HTMLCanvasElement;
    let priceHistory: { ts: number; price: number }[] = [];
    let lastPrice = '';
    let prevPrice = '';
    let direction: 'up' | 'down' | 'flat' = 'flat';

    // Subscribe to ticker updates.
    const unsub = tickers.subscribe((map: Record<string, TickerUpdate>) => {
        const t = map[pair];
        if (!t) return;
        prevPrice = lastPrice;
        lastPrice = t.derived_egp_price;
        if (prevPrice && lastPrice) {
            if (Number(lastPrice) > Number(prevPrice)) direction = 'up';
            else if (Number(lastPrice) < Number(prevPrice)) direction = 'down';
            else direction = 'flat';
        }
        priceHistory.push({ ts: Date.now(), price: Number(t.derived_egp_price) });
        if (priceHistory.length > 200) priceHistory = priceHistory.slice(-200);
        render();
    });

    function render() {
        if (!canvas || priceHistory.length < 2) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;
        const w = canvas.width = canvas.offsetWidth * 2;
        const h = canvas.height = canvas.offsetHeight * 2;
        ctx.clearRect(0, 0, w, h);

        const prices = priceHistory.map((p) => p.price);
        const min = Math.min(...prices);
        const max = Math.max(...prices);
        const range = max - min || 1;
        const pad = 8 * 2;

        const color = direction === 'up' ? '#00d68f' : direction === 'down' ? '#ff5252' : '#3b82f6';
        ctx.strokeStyle = color;
        ctx.lineWidth = 2;
        ctx.beginPath();
        priceHistory.forEach((p, i) => {
            const x = (i / (priceHistory.length - 1)) * (w - pad * 2) + pad;
            const y = h - pad - ((p.price - min) / range) * (h - pad * 2);
            if (i === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
        });
        ctx.stroke();

        // Fill area below.
        ctx.lineTo(w - pad, h - pad);
        ctx.lineTo(pad, h - pad);
        ctx.closePath();
        ctx.fillStyle = color + '15';
        ctx.fill();
    }

    onMount(() => {
        const ro = new ResizeObserver(() => render());
        if (canvas) ro.observe(canvas);
        return () => { unsub(); ro.disconnect(); };
    });
</script>

<div class="card-default">
    <div class="flex items-center justify-between mb-3">
        <div>
            <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">{pair.replace('_', '/')}</h3>
            <div class="text-xs text-text-tertiary">Live price (derived from Binance)</div>
        </div>
        <div class="text-right">
            <div class="text-2xl font-mono font-bold {direction === 'up' ? 'text-accent-green' : direction === 'down' ? 'text-accent-red' : 'text-text-primary'}">
                {fmtEgp(lastPrice || '0')}
            </div>
            {#if prevPrice}
                <div class="text-xs {direction === 'up' ? 'text-accent-green' : direction === 'down' ? 'text-accent-red' : 'text-text-tertiary'}">
                    {direction === 'up' ? '▲' : direction === 'down' ? '▼' : '▬'} {fmtPrice(prevPrice)}
                </div>
            {/if}
        </div>
    </div>
    <div class="h-48 relative">
        <canvas bind:this={canvas} class="w-full h-full"></canvas>
        {#if $circuitOpen}
            <div class="absolute inset-0 bg-base-900/60 flex items-center justify-center text-accent-red text-sm font-semibold">
                Feed halted
            </div>
        {/if}
    </div>
</div>
