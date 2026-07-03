<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { circuitOpen, tickers } from '$lib/stores';
    import { fmtEgp, fmtQty, pairToAssets, mulDecimal } from '$lib/format';
    import type { OrderSide, OrderType } from '$lib/types';

    export let pair: string;
    const { base, quote } = pairToAssets(pair);

    let side: OrderSide = 'buy';
    let orderType: OrderType = 'limit';
    let price = '';
    let quantity = '';
    let total = '';
    let submitting = false;
    let error = '';
    let success = '';

    const dispatch = createEventDispatcher();

    $: ticker = $tickers[pair];
    $: estimatedTotal = price && quantity ? mulDecimal(price, quantity) : '';

    function setSide(s: OrderSide) {
        side = s;
        error = '';
        success = '';
    }
    function setType(t: OrderType) {
        orderType = t;
        if (t === 'market' && ticker) {
            price = side === 'buy' ? ticker.ask : ticker.bid;
        }
        error = '';
    }
    function computeTotalFromQty() {
        if (price && quantity) total = mulDecimal(price, quantity);
    }
    function usePercent(p: number) {
        // Placeholder: would need wallet balance to compute real percent.
        // For now just multiplies a constant.
        if (!price || !quantity) return;
        total = mulDecimal(price, quantity);
    }

    async function submit() {
        if ($circuitOpen) {
            error = 'Trading halted: Binance feed is down';
            return;
        }
        if (!quantity || Number(quantity) <= 0) {
            error = 'Quantity must be greater than 0';
            return;
        }
        if (orderType === 'limit' && (!price || Number(price) <= 0)) {
            error = 'Limit orders require a price';
            return;
        }
        submitting = true;
        error = '';
        success = '';
        try {
            const { trading } = await import('$lib/api');
            const res = await trading.placeOrder({
                pair,
                side,
                order_type: orderType,
                price: orderType === 'limit' ? price : undefined,
                quantity,
            });
            success = `Order placed — ${res.trades.length} trades matched, ${res.remaining} remaining`;
            dispatch('placed', res);
            quantity = '';
            total = '';
        } catch (e: any) {
            error = e.message || 'Order failed';
        } finally {
            submitting = false;
        }
    }
</script>

<div class="card-default">
    <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">Place Order</h3>
        <span class="text-xs text-text-tertiary">{base}/{quote}</span>
    </div>

    <!-- Side toggle -->
    <div class="grid grid-cols-2 gap-2 mb-3">
        <button
            class="btn {side === 'buy' ? 'bg-accent-green text-base-900 hover:bg-emerald-500' : 'bg-base-700 text-text-secondary hover:bg-base-600'}"
            on:click={() => setSide('buy')}>
            Buy {base}
        </button>
        <button
            class="btn {side === 'sell' ? 'bg-accent-red text-white hover:bg-red-600' : 'bg-base-700 text-text-secondary hover:bg-base-600'}"
            on:click={() => setSide('sell')}>
            Sell {base}
        </button>
    </div>

    <!-- Order type -->
    <div class="flex gap-1 mb-3 text-xs">
        <button class="flex-1 px-2 py-1 rounded {orderType === 'limit' ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}" on:click={() => setType('limit')}>Limit</button>
        <button class="flex-1 px-2 py-1 rounded {orderType === 'market' ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}" on:click={() => setType('market')}>Market</button>
    </div>

    <!-- Inputs -->
    <div class="space-y-3">
        {#if orderType === 'limit'}
            <div>
                <label class="label" for="price">Price ({quote})</label>
                <input id="price" type="number" step="any" bind:value={price} on:input={computeTotalFromQty} class="input" placeholder="0.00" />
            </div>
        {/if}
        <div>
            <label class="label" for="qty">Amount ({base})</label>
            <input id="qty" type="number" step="any" bind:value={quantity} on:input={computeTotalFromQty} class="input" placeholder="0.00" />
        </div>
        <div>
            <label class="label" for="total">Total ({quote})</label>
            <input id="total" type="number" step="any" value={estimatedTotal} disabled class="input opacity-70 cursor-not-allowed" placeholder="0.00" />
        </div>
    </div>

    <!-- Error/Success -->
    {#if error}
        <div class="mt-3 text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-2.5 py-1.5">{error}</div>
    {/if}
    {#if success}
        <div class="mt-3 text-xs text-accent-green bg-accent-green/10 border border-accent-green/30 rounded px-2.5 py-1.5">{success}</div>
    {/if}

    <!-- Submit -->
    <button
        class="mt-4 w-full btn {side === 'buy' ? 'bg-accent-green hover:bg-emerald-500 text-base-900' : 'bg-accent-red hover:bg-red-600 text-white'}"
        on:click={submit}
        disabled={submitting || $circuitOpen}>
        {#if submitting}
            Submitting...
        {:else if $circuitOpen}
            Trading Halted
        {:else}
            {side === 'buy' ? 'Buy' : 'Sell'} {base}
        {/if}
    </button>

    {#if ticker}
        <div class="mt-3 text-xs text-text-tertiary flex justify-between">
            <span>Best Bid: <span class="text-accent-green num-cell">{fmtEgp(ticker.bid)}</span></span>
            <span>Best Ask: <span class="text-accent-red num-cell">{fmtEgp(ticker.ask)}</span></span>
        </div>
    {/if}
</div>
