<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { circuitOpen, tickers, myWallets, pushNotification } from '$lib/stores';
    import { fmtEgp, fmtQty, pairToAssets, mulDecimal, divDecimal } from '$lib/format';
    import type { OrderSide, OrderType } from '$lib/types';

    export let pair: string;
    const { base, quote } = pairToAssets(pair);

    let side: OrderSide = 'buy';
    let orderType: OrderType = 'limit';
    let price = '';
    let quantity = '';
    let submitting = false;
    let error = '';
    let success = '';

    const dispatch = createEventDispatcher();

    $: ticker = $tickers[pair];
    $: baseWallet = $myWallets.find((w) => w.asset_symbol === base);
    $: quoteWallet = $myWallets.find((w) => w.asset_symbol === quote);
    $: availableBalance = side === 'buy' ? (quoteWallet?.balance || '0') : (baseWallet?.balance || '0');
    $: estimatedTotal = price && quantity ? mulDecimal(price, quantity) : '';
    $: percentUsed = (() => {
        if (!estimatedTotal || !availableBalance || Number(availableBalance) === 0) return 0;
        return Math.min(100, (Number(estimatedTotal) / Number(availableBalance)) * 100);
    })();

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
    function setPercent(p: number) {
        if (!availableBalance || Number(availableBalance) === 0) return;
        if (side === 'buy') {
            const usable = Number(availableBalance) * (p / 100);
            if (price && Number(price) > 0) {
                quantity = divDecimal(usable.toFixed(8), price);
            }
        } else {
            quantity = (Number(availableBalance) * (p / 100)).toFixed(8);
        }
    }

    async function submit() {
        if ($circuitOpen) {
            error = 'التداول متوقف: تدفق الأسعار متوقف';
            return;
        }
        if (!quantity || Number(quantity) <= 0) {
            error = 'الكمية يجب أن تكون أكبر من صفر';
            return;
        }
        if (orderType === 'limit' && (!price || Number(price) <= 0)) {
            error = 'الأوامر المحددة تتطلب سعراً';
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
            success = `تم تنفيذ الأمر — ${res.trades.length} صفقة، ${res.remaining} متبقي`;
            dispatch('placed', res);
            pushNotification({
                type: 'success',
                title: 'تم تنفيذ الأمر',
                message: `${side === 'buy' ? 'شراء' : 'بيع'} ${quantity} ${base} — ${res.trades.length} صفقة`,
            });
            quantity = '';
        } catch (e: any) {
            error = e.message || 'فشل تنفيذ الأمر';
        } finally {
            submitting = false;
        }
    }
</script>

<div class="card-default">
    <div class="flex items-center justify-between mb-3">
        <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">تنفيذ أمر</h3>
        <span class="text-xs text-text-tertiary">{base}/{quote}</span>
    </div>

    <!-- تبديل الجهة -->
    <div class="grid grid-cols-2 gap-2 mb-3">
        <button
            class="btn {side === 'buy' ? 'bg-accent-green text-base-900 hover:bg-emerald-500' : 'bg-base-700 text-text-secondary hover:bg-base-600'}"
            on:click={() => setSide('buy')}>
            شراء {base}
        </button>
        <button
            class="btn {side === 'sell' ? 'bg-accent-red text-white hover:bg-red-600' : 'bg-base-700 text-text-secondary hover:bg-base-600'}"
            on:click={() => setSide('sell')}>
            بيع {base}
        </button>
    </div>

    <!-- نوع الأمر -->
    <div class="flex gap-1 mb-3 text-xs">
        <button class="flex-1 px-2 py-1 rounded {orderType === 'limit' ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}" on:click={() => setType('limit')}>محدد</button>
        <button class="flex-1 px-2 py-1 rounded {orderType === 'market' ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}" on:click={() => setType('market')}>سوقي</button>
    </div>

    <!-- الرصيد المتاح -->
    <div class="bg-base-900 border border-base-700 rounded-md px-3 py-1.5 mb-3 text-xs flex items-center justify-between">
        <span class="text-text-tertiary">الرصيد المتاح</span>
        <span class="num-cell font-semibold text-text-primary">{side === 'buy' ? fmtEgp(availableBalance) : fmtQty(availableBalance, 8)} {side === 'buy' ? quote : base}</span>
    </div>

    <!-- الحقول -->
    <div class="space-y-3">
        {#if orderType === 'limit'}
            <div>
                <label class="label" for="price">السعر ({quote})</label>
                <input id="price" type="number" step="any" bind:value={price} class="input" placeholder="0.00" />
            </div>
        {/if}
        <div>
            <label class="label" for="qty">الكمية ({base})</label>
            <input id="qty" type="number" step="any" bind:value={quantity} class="input" placeholder="0.00000000" />
        </div>

        <!-- أزرار النسبة المئوية -->
        <div class="grid grid-cols-4 gap-1">
            {#each [25, 50, 75, 100] as p}
                <button class="px-2 py-1 text-xs rounded bg-base-900 border border-base-700 hover:border-accent-blue hover:text-accent-blue text-text-secondary transition-colors" on:click={() => setPercent(p)}>
                    {p}%
                </button>
            {/each}
        </div>

        {#if estimatedTotal}
            <div class="bg-base-900 border border-base-700 rounded-md px-3 py-2">
                <div class="text-xs text-text-tertiary mb-0.5">الإجمالي</div>
                <div class="num-cell text-base font-bold text-text-primary">{fmtEgp(estimatedTotal)} {quote}</div>
            </div>
        {/if}
    </div>

    <!-- خطأ / نجاح -->
    {#if error}
        <div class="mt-3 text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-2.5 py-1.5">{error}</div>
    {/if}
    {#if success}
        <div class="mt-3 text-xs text-accent-green bg-accent-green/10 border border-accent-green/30 rounded px-2.5 py-1.5">{success}</div>
    {/if}

    <!-- زر التنفيذ -->
    <button
        class="mt-4 w-full btn {side === 'buy' ? 'bg-accent-green hover:bg-emerald-500 text-base-900' : 'bg-accent-red hover:bg-red-600 text-white'}"
        on:click={submit}
        disabled={submitting || $circuitOpen}>
        {#if submitting}
            جارٍ التنفيذ...
        {:else if $circuitOpen}
            التداول متوقف
        {:else}
            {side === 'buy' ? 'شراء' : 'بيع'} {base}
        {/if}
    </button>

    {#if ticker}
        <div class="mt-3 text-xs text-text-tertiary flex justify-between">
            <span>أعلى طلب: <span class="text-accent-green num-cell">{fmtEgp(ticker.bid)}</span></span>
            <span>أقل عرض: <span class="text-accent-red num-cell">{fmtEgp(ticker.ask)}</span></span>
        </div>
    {/if}
</div>
