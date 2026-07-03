<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { page } from '$app/stores';
    import { tickers, availablePairs, circuitOpen, myPositions, myWallets, futuresMarkPrice, connectMarketWs } from '$lib/stores';
    import { fmtEgp, fmtPrice, fmtQty, pairToAssets, mulDecimal, divDecimal } from '$lib/format';
    import CandlestickChart from '$lib/components/CandlestickChart.svelte';
    import type { PositionSide, MarginMode, Position } from '$lib/types';

    $: pair = $page.params.pair || 'BTC_EGP';
    $: { base, quote } = pairToAssets(pair);
    $: ticker = $tickers[pair];
    $: markPrice = ticker?.derived_egp_price || $futuresMarkPrice[pair] || '0';

    let side: PositionSide = 'long';
    let marginMode: MarginMode = 'isolated';
    let leverage = 10;
    let margin = '';
    let submitting = false;
    let error = '';
    let success = '';

    $: quantity = margin && leverage ? mulDecimal(margin, String(leverage)) : '';
    $: estimatedLiquidation = (() => {
        if (!margin || !leverage || !markPrice || Number(markPrice) === 0) return '';
        const lev = Number(leverage);
        const entry = Number(markPrice);
        if (side === 'long') {
            const liq = entry * (1 - 1 / lev);
            return liq.toFixed(2);
        } else {
            const liq = entry * (1 + 1 / lev);
            return liq.toFixed(2);
        }
    })();

    $: myOpenPositions = $myPositions.filter((p) => p.pair === pair && p.status === 'open');

    async function openPosition() {
        if ($circuitOpen) {
            error = 'التداول متوقف: تدفق الأسعار متوقف';
            return;
        }
        if (!margin || Number(margin) <= 0) {
            error = 'الهامش يجب أن يكون أكبر من صفر';
            return;
        }
        submitting = true;
        error = '';
        success = '';
        try {
            const { futures } = await import('$lib/api');
            await futures.openPosition({
                pair,
                side,
                margin_mode: marginMode,
                leverage,
                margin,
            });
            success = 'تم فتح المركز بنجاح';
            margin = '';
        } catch (e: any) {
            error = e.message || 'فشل فتح المركز';
        } finally {
            submitting = false;
        }
    }

    async function closePosition(id: string) {
        if (!confirm('هل تريد إغلاق هذا المركز؟')) return;
        try {
            const { futures } = await import('$lib/api');
            await futures.closePosition(id);
        } catch (e: any) {
            alert(e.message);
        }
    }

    onMount(() => {
        connectMarketWs();
    });
</script>

<svelte:head><title>عقود آجلة {pair.replace('_', '/')} · منصة الجنيه</title></svelte:head>

<div class="space-y-4">
    <!-- اختيار الزوج -->
    <div class="card-compact flex items-center justify-between flex-wrap gap-2">
        <div class="flex items-center gap-2 flex-wrap">
            {#each ($availablePairs.length ? $availablePairs : ['BTC_EGP', 'ETH_EGP', 'USDT_EGP']) as p}
                <a href="/futures/{p}" class="px-3 py-1.5 rounded-md text-sm font-medium transition-colors {p === pair ? 'bg-base-600 text-text-primary' : 'text-text-secondary hover:bg-base-700/50'}">
                    {p.replace('_', '/')}
                </a>
            {/each}
        </div>
        {#if ticker}
            <div class="text-sm">
                <span class="text-text-tertiary">سعر السوق:</span>
                <span class="text-text-primary font-mono mr-1">{fmtEgp(markPrice)}</span>
            </div>
        {/if}
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <!-- الرسم + المراكز -->
        <div class="lg:col-span-8 space-y-4">
            <CandlestickChart {pair} />

            <!-- تحذير العقود الآجلة -->
            <div class="bg-accent-yellow/10 border border-accent-yellow/30 text-accent-yellow rounded-md px-4 py-2.5 text-xs">
                <strong>تحذير:</strong> تداول العقود الآجلة ينطوي على مخاطر عالية. الرافعة المالية تضخم الأرباح والخسائر. قد تخسر كامل رأس مالك.
            </div>

            <!-- مراكزي المفتوحة -->
            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">مراكزي المفتوحة</h3>
                {#if myOpenPositions.length === 0}
                    <div class="text-center py-6 text-text-tertiary text-sm">لا توجد مراكز مفتوحة</div>
                {:else}
                    <div class="overflow-x-auto">
                        <table class="table-pro">
                            <thead>
                                <tr>
                                    <th>الجهة</th>
                                    <th>الرافعة</th>
                                    <th class="num-cell">الهامش</th>
                                    <th class="num-cell">الحجم</th>
                                    <th class="num-cell">سعر الدخول</th>
                                    <th class="num-cell">سعر السوق</th>
                                    <th class="num-cell">سعر التصفية</th>
                                    <th class="num-cell">PnL</th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                                {#each myOpenPositions as p}
                                    {@const pnl = Number(p.unrealized_pnl)}
                                    <tr>
                                        <td><span class={p.side === 'long' ? 'text-accent-green' : 'text-accent-red'}>{p.side === 'long' ? 'شراء' : 'بيع'} {p.leverage}x</span></td>
                                        <td class="text-text-secondary">{p.margin_mode === 'isolated' ? 'معزول' : 'متقاطع'}</td>
                                        <td class="num-cell">{fmtEgp(p.margin)}</td>
                                        <td class="num-cell">{fmtQty(p.quantity, 8)}</td>
                                        <td class="num-cell">{fmtPrice(p.entry_price)}</td>
                                        <td class="num-cell">{fmtPrice(p.mark_price)}</td>
                                        <td class="num-cell text-accent-yellow">{fmtPrice(p.liquidation_price)}</td>
                                        <td class="num-cell {pnl >= 0 ? 'text-accent-green' : 'text-accent-red'}">{pnl >= 0 ? '+' : ''}{fmtEgp(pnl.toFixed(2))}</td>
                                        <td><button class="text-accent-red text-xs hover:underline" on:click={() => closePosition(p.id)}>إغلاق</button></td>
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {/if}
            </div>
        </div>

        <!-- لوحة فتح المركز -->
        <div class="lg:col-span-4">
            <div class="card-default sticky top-20">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">فتح مركز</h3>

                <!-- الجهة -->
                <div class="grid grid-cols-2 gap-2 mb-3">
                    <button
                        class="btn {side === 'long' ? 'bg-accent-green text-base-900 hover:bg-emerald-500' : 'bg-base-700 text-text-secondary hover:bg-base-600'}"
                        on:click={() => (side = 'long')}>
                        شراء (Long)
                    </button>
                    <button
                        class="btn {side === 'short' ? 'bg-accent-red text-white hover:bg-red-600' : 'bg-base-700 text-text-secondary hover:bg-base-600'}"
                        on:click={() => (side = 'short')}>
                        بيع (Short)
                    </button>
                </div>

                <!-- نوع الهامش -->
                <div class="flex gap-1 mb-3 text-xs">
                    <button class="flex-1 px-2 py-1 rounded {marginMode === 'isolated' ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}" on:click={() => (marginMode = 'isolated')}>معزول</button>
                    <button class="flex-1 px-2 py-1 rounded {marginMode === 'cross' ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}" on:click={() => (marginMode = 'cross')}>متقاطع</button>
                </div>

                <!-- الرافعة -->
                <div class="mb-3">
                    <div class="flex items-center justify-between mb-1">
                        <label class="label mb-0">الرافعة المالية</label>
                        <span class="text-sm font-mono font-bold text-accent-blue">{leverage}x</span>
                    </div>
                    <input type="range" min="1" max="125" bind:value={leverage} class="w-full accent-accent-blue" />
                    <div class="flex justify-between text-[10px] text-text-tertiary mt-0.5">
                        <span>1x</span>
                        <span>25x</span>
                        <span>50x</span>
                        <span>75x</span>
                        <span>125x</span>
                    </div>
                </div>

                <!-- الهامش -->
                <div class="mb-3">
                    <label class="label" for="margin">الهامش ({quote})</label>
                    <input id="margin" type="number" step="any" min="0" bind:value={margin} class="input" placeholder="0.00" />
                </div>

                <!-- ملخص -->
                <div class="bg-base-900 border border-base-700 rounded-md p-3 mb-3 space-y-1.5 text-xs">
                    <div class="flex justify-between">
                        <span class="text-text-tertiary">حجم المركز</span>
                        <span class="num-cell font-semibold text-text-primary">{quantity ? fmtEgp(quantity) : '—'}</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-text-tertiary">سعر الدخول المتوقع</span>
                        <span class="num-cell text-text-primary">{fmtPrice(markPrice)}</span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-text-tertiary">سعر التصفية المتوقع</span>
                        <span class="num-cell text-accent-yellow">{estimatedLiquidation || '—'}</span>
                    </div>
                </div>

                {#if error}<div class="text-xs text-accent-red mb-2">{error}</div>{/if}
                {#if success}<div class="text-xs text-accent-green mb-2">{success}</div>{/if}

                <button
                    class="w-full btn {side === 'long' ? 'bg-accent-green hover:bg-emerald-500 text-base-900' : 'bg-accent-red hover:bg-red-600 text-white'}"
                    on:click={openPosition}
                    disabled={submitting || $circuitOpen}>
                    {#if submitting}
                        جارٍ الفتح...
                    {:else if $circuitOpen}
                        التداول متوقف
                    {:else}
                        {side === 'long' ? 'فتح مركز شراء' : 'فتح مركز بيع'} ({leverage}x)
                    {/if}
                </button>
            </div>
        </div>
    </div>
</div>
