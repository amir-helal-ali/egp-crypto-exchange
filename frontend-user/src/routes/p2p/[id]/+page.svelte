<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';
    import { p2p } from '$lib/api';
    import { tickers } from '$lib/stores';
    import { fmtEgp, fmtQty, fmtDate, fmtRelative } from '$lib/format';
    import type { P2POffer } from '$lib/types';

    let offer: P2POffer | null = null;
    let loading = true;
    let error = '';
    let amount = '';
    let selectedPayment = '';
    let starting = false;

    $: id = $page.params.id;
    $: ticker = offer ? $tickers[`${offer.asset_symbol}_EGP`] : null;
    $: estimatedPrice = (() => {
        if (!offer || !ticker) return 0;
        const base = Number(ticker.derived_egp_price);
        return base * (1 + offer.price_margin_pct / 100);
    })();
    $: totalEgp = amount && estimatedPrice ? (Number(amount) * estimatedPrice).toFixed(2) : '0';

    async function load() {
        try {
            offer = await p2p.getOffer(id);
            if (offer.payment_methods.length > 0) selectedPayment = offer.payment_methods[0];
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    async function startTrade() {
        if (!offer) return;
        if (!amount || Number(amount) <= 0) {
            error = 'أدخل مبلغ صحيح';
            return;
        }
        starting = true;
        error = '';
        try {
            const trade = await p2p.startTrade({
                offer_id: offer.id,
                amount,
                payment_method: selectedPayment,
            });
            await goto(`/p2p/trade/${trade.id}`, { replace: true });
        } catch (e: any) {
            error = e.message;
        } finally {
            starting = false;
        }
    }
</script>

<svelte:head><title>عرض P2P · منصة الجنيه</title></svelte:head>

<div class="max-w-3xl mx-auto space-y-4">
    <a href="/p2p" class="text-xs text-accent-blue hover:inline">← العودة للسوق</a>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if offer}
        <div class="card-default">
            <!-- رأس العرض -->
            <div class="flex items-center justify-between mb-4 pb-4 border-b border-base-700">
                <div class="flex items-center gap-3">
                    <div class="w-12 h-12 rounded-full bg-gradient-to-br from-base-700 to-base-600 flex items-center justify-center text-sm font-bold {offer.side === 'buy' ? 'text-accent-green' : 'text-accent-red'}">
                        {offer.asset_symbol.slice(0, 3)}
                    </div>
                    <div>
                        <div class="text-lg font-bold text-text-primary">
                            {offer.side === 'buy' ? 'شراء' : 'بيع'} {offer.asset_symbol}
                        </div>
                        <div class="text-xs text-text-tertiary">صاحب العرض: {offer.user_email || offer.user_id.slice(0, 8)}</div>
                    </div>
                </div>
                <div class="text-left">
                    <div class="text-xs text-text-tertiary">سعر التقديري</div>
                    <div class="text-lg font-mono font-bold text-text-primary">{fmtEgp(estimatedPrice.toFixed(2))}</div>
                </div>
            </div>

            <!-- تفاصيل -->
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-4 text-sm">
                <div>
                    <div class="text-xs text-text-tertiary">الحد الأدنى</div>
                    <div class="num-cell text-text-primary">{fmtEgp(offer.min_amount_egp)}</div>
                </div>
                <div>
                    <div class="text-xs text-text-tertiary">الحد الأقصى</div>
                    <div class="num-cell text-text-primary">{fmtEgp(offer.max_amount_egp)}</div>
                </div>
                <div>
                    <div class="text-xs text-text-tertiary">معدل الإكمال</div>
                    <div class="num-cell text-accent-green">{offer.completion_rate}%</div>
                </div>
                <div>
                    <div class="text-xs text-text-tertiary">عدد الصفقات</div>
                    <div class="num-cell text-text-primary">{offer.total_trades}</div>
                </div>
            </div>

            <!-- نموذج بدء الصفقة -->
            <div class="border-t border-base-700 pt-4 space-y-3">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">بدء صفقة</h3>
                <div>
                    <label class="label" for="amount">المبلغ ({offer.asset_symbol})</label>
                    <input id="amount" type="number" step="any" bind:value={amount} class="input" placeholder="0.00" />
                    <div class="text-xs text-text-tertiary mt-1">
                        الإجمالي بالجنيه: <span class="num-cell font-bold text-text-primary">{fmtEgp(totalEgp)}</span>
                    </div>
                </div>

                <div>
                    <label class="label" for="payment">طريقة الدفع</label>
                    <select id="payment" bind:value={selectedPayment} class="input">
                        {#each offer.payment_methods as pm}
                            <option value={pm}>{pm}</option>
                        {/each}
                    </select>
                </div>

                {#if error}
                    <div class="text-xs text-accent-red">{error}</div>
                {/if}

                <button class="w-full btn-primary text-sm" on:click={startTrade} disabled={starting || !amount}>
                    {starting ? 'جارٍ بدء الصفقة...' : 'بدء الصفقة'}
                </button>

                <div class="text-xs text-text-tertiary text-center">
                    ستتم حماية الصفقة بنظام الضمان - يتم حجز العملات الرقمية حتى تأكيد الدفع
                </div>
            </div>
        </div>
    {/if}
</div>
