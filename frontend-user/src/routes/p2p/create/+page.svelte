<script lang="ts">
    import { onMount } from 'svelte';
    import { navigate } from '$app/navigation';
    import { p2p } from '$lib/api';
    import { tickers } from '$lib/stores';
    import { fmtEgp } from '$lib/format';
    import type { P2POffer } from '$lib/types';

    let side: 'buy' | 'sell' = 'buy';
    let asset = 'USDT';
    let priceMargin = 0;
    let minAmount = '100';
    let maxAmount = '5000';
    let timeLimit = 30;
    let selectedPayments: string[] = ['bank_transfer'];
    let error = '';
    let submitting = false;

    const paymentMethods = [
        { id: 'bank_transfer', label: 'تحويل بنكي' },
        { id: 'vodafone_cash', label: 'فودافون كاش' },
        { id: 'instapay', label: 'إنستا باي' },
        { id: 'fawry', label: 'فوري' },
        { id: 'etisalat_cash', label: 'اتصالات كاش' },
        { id: 'orange_cash', label: 'أورانج كاش' },
        { id: 'we_pay', label: 'وي باي' },
        { id: 'cash_deposit', label: 'إيداع نقدي' },
    ];

    $: ticker = $tickers[`${asset}_EGP`];
    $: estimatedPrice = (() => {
        if (!ticker) return 0;
        const base = Number(ticker.derived_egp_price);
        return base * (1 + priceMargin / 100);
    })();

    function togglePayment(id: string) {
        if (selectedPayments.includes(id)) {
            selectedPayments = selectedPayments.filter((p) => p !== id);
        } else {
            selectedPayments = [...selectedPayments, id];
        }
    }

    async function submit() {
        if (selectedPayments.length === 0) {
            error = 'اختر طريقة دفع واحدة على الأقل';
            return;
        }
        submitting = true;
        error = '';
        try {
            await p2p.createOffer({
                side,
                asset_symbol: asset,
                price_margin_pct: priceMargin,
                min_amount_egp: minAmount,
                max_amount_egp: maxAmount,
                payment_methods: selectedPayments,
                time_limit_min: timeLimit,
            });
            await navigate('/p2p', { replace: true });
        } catch (e: any) {
            error = e.message;
        } finally {
            submitting = false;
        }
    }
</script>

<svelte:head><title>إنشاء عرض P2P · منصة الجنيه</title></svelte:head>

<div class="max-w-2xl mx-auto space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">إنشاء عرض جديد</h1>
        <p class="text-sm text-text-secondary mt-1">أنشئ عرضاً لبيع أو شراء عملة رقمية مقابل الجنيه</p>
    </div>

    <div class="card-default space-y-4">
        <!-- الجهة -->
        <div>
            <label class="label">الجهة</label>
            <div class="grid grid-cols-2 gap-2">
                <button class="btn {side === 'buy' ? 'bg-accent-green text-base-900 hover:bg-emerald-500' : 'bg-base-700 text-text-secondary hover:bg-base-600'}" on:click={() => (side = 'buy')}>
                    شراء بالجنيه
                </button>
                <button class="btn {side === 'sell' ? 'bg-accent-red text-white hover:bg-red-600' : 'bg-base-700 text-text-secondary hover:bg-base-600'}" on:click={() => (side = 'sell')}>
                    بيع مقابل جنيه
                </button>
            </div>
            <p class="text-xs text-text-tertiary mt-1">
                {side === 'buy' ? 'أنت تشتري العملة الرقمية وتدفع بالجنيه' : 'أنت تبيع العملة الرقمية وتستلم الجنيه'}
            </p>
        </div>

        <!-- العملة -->
        <div>
            <label class="label" for="asset">العملة</label>
            <select id="asset" bind:value={asset} class="input">
                <option value="BTC">BTC</option>
                <option value="ETH">ETH</option>
                <option value="USDT">USDT</option>
            </select>
        </div>

        <!-- هامش السعر -->
        <div>
            <div class="flex items-center justify-between mb-1">
                <label class="label mb-0" for="margin">هامش السعر (%)</label>
                <span class="text-sm font-mono font-bold text-accent-blue">{priceMargin > 0 ? '+' : ''}{priceMargin}%</span>
            </div>
            <input id="margin" type="number" step="0.1" min="-50" max="50" bind:value={priceMargin} class="input" />
            <p class="text-xs text-text-tertiary mt-1">
                {#if ticker}
                    السعر التقديري: <span class="num-cell text-text-primary font-semibold">{fmtEgp(estimatedPrice.toFixed(2))}</span>
                    (سعر السوق: {fmtEgp(ticker.derived_egp_price)})
                {:else}
                    بانتظار سعر السوق...
                {/if}
            </p>
        </div>

        <!-- الحدود -->
        <div class="grid grid-cols-2 gap-3">
            <div>
                <label class="label" for="min">الحد الأدنى (جنيه)</label>
                <input id="min" type="number" step="any" bind:value={minAmount} class="input" />
            </div>
            <div>
                <label class="label" for="max">الحد الأقصى (جنيه)</label>
                <input id="max" type="number" step="any" bind:value={maxAmount} class="input" />
            </div>
        </div>

        <!-- المهلة الزمنية -->
        <div>
            <label class="label" for="time">المهلة الزمنية للدفع (دقائق)</label>
            <select id="time" bind:value={timeLimit} class="input">
                <option value={15}>15 دقيقة</option>
                <option value={30}>30 دقيقة</option>
                <option value={60}>60 دقيقة</option>
                <option value={120}>120 دقيقة</option>
            </select>
        </div>

        <!-- طرق الدفع -->
        <div>
            <label class="label">طرق الدفع المقبولة</label>
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
                {#each paymentMethods as pm}
                    <button
                        type="button"
                        class="px-3 py-2 rounded-md text-xs font-medium border transition-colors {selectedPayments.includes(pm.id) ? 'bg-accent-blue/10 border-accent-blue text-accent-blue' : 'bg-base-900 border-base-700 text-text-secondary hover:border-base-500'}"
                        on:click={() => togglePayment(pm.id)}>
                        {pm.label}
                    </button>
                {/each}
            </div>
        </div>

        {#if error}
            <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
        {/if}

        <div class="flex gap-2">
            <a href="/p2p" class="btn-ghost flex-1 text-sm">إلغاء</a>
            <button class="btn-primary flex-1 text-sm" on:click={submit} disabled={submitting}>
                {submitting ? 'جارٍ الإنشاء...' : 'إنشاء العرض'}
            </button>
        </div>
    </div>
</div>
