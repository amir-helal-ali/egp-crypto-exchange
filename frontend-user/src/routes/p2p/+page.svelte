<script lang="ts">
    import { onMount } from 'svelte';
    import { p2p } from '$lib/api';
    import { fmtEgp, fmtQty, fmtRelative } from '$lib/format';
    import type { P2POffer } from '$lib/types';

    let offers: P2POffer[] = [];
    let loading = true;
    let error = '';
    let sideFilter: 'all' | 'buy' | 'sell' = 'all';
    let assetFilter = '';
    let paymentFilter = '';

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

    async function load() {
        try {
            offers = await p2p.listOffers({
                side: sideFilter === 'all' ? undefined : sideFilter,
                asset: assetFilter || undefined,
                payment_method: paymentFilter || undefined,
            });
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    function paymentLabel(id: string): string {
        return paymentMethods.find((p) => p.id === id)?.label || id;
    }
</script>

<svelte:head><title>التداول بين الأفراد · منصة الجنيه</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">سوق التداول بين الأفراد</h1>
            <p class="text-sm text-text-secondary mt-1">اشترِ وبِع العملات الرقمية مقابل الجنيه مباشرةً مع مستخدمين آخرين</p>
        </div>
        <a href="/p2p/create" class="btn-primary text-sm">+ إنشاء عرض</a>
    </div>

    <!-- الفلاتر -->
    <div class="card-compact flex items-center gap-2 flex-wrap text-sm">
        <div class="flex gap-1">
            {#each [['all', 'الكل'], ['buy', 'شراء بالجنيه'], ['sell', 'بيع مقابل جنيه']] as [v, label]}
                <button
                    class="px-3 py-1.5 rounded-md text-xs font-medium {sideFilter === v ? 'bg-base-600 text-text-primary' : 'text-text-secondary hover:bg-base-700/50'}"
                    on:click={() => { sideFilter = v as any; load(); }}>
                    {label}
                </button>
            {/each}
        </div>
        <select bind:value={assetFilter} on:change={load} class="input max-w-[120px] text-xs py-1.5">
            <option value="">كل العملات</option>
            <option value="BTC">BTC</option>
            <option value="ETH">ETH</option>
            <option value="USDT">USDT</option>
        </select>
        <select bind:value={paymentFilter} on:change={load} class="input max-w-[160px] text-xs py-1.5">
            <option value="">كل طرق الدفع</option>
            {#each paymentMethods as pm}
                <option value={pm.id}>{pm.label}</option>
            {/each}
        </select>
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if offers.length === 0}
        <div class="card-default text-center py-12">
            <div class="text-text-tertiary text-sm mb-4">لا توجد عروض مطابقة</div>
            <a href="/p2p/create" class="btn-primary text-sm">+ أنشئ أول عرض</a>
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {#each offers as offer}
                <a href="/p2p/{offer.id}" class="card-default hover:border-accent-blue transition-all block group">
                    <div class="flex items-center justify-between mb-3">
                        <div class="flex items-center gap-2">
                            <div class="w-10 h-10 rounded-full bg-gradient-to-br from-base-700 to-base-600 flex items-center justify-center text-xs font-bold {offer.side === 'buy' ? 'text-accent-green' : 'text-accent-red'}">
                                {offer.asset_symbol.slice(0, 3)}
                            </div>
                            <div>
                                <div class="text-sm font-bold text-text-primary">
                                    {offer.side === 'buy' ? 'شراء' : 'بيع'} {offer.asset_symbol}
                                </div>
                                <div class="text-xs text-text-tertiary">{paymentMethods.find((p) => offer.payment_methods.includes(p.id))?.label || offer.payment_methods.length} طرق دفع</div>
                            </div>
                        </div>
                        <div class="text-left">
                            <div class="text-xs text-text-tertiary">السعر</div>
                            <div class="text-sm font-mono font-bold text-text-primary">{fmtEgp('0')}*</div>
                        </div>
                    </div>

                    <div class="grid grid-cols-2 gap-2 text-xs mb-3">
                        <div class="bg-base-900 rounded p-2">
                            <div class="text-text-tertiary mb-0.5">الحد الأدنى</div>
                            <div class="num-cell text-text-primary">{fmtEgp(offer.min_amount_egp)}</div>
                        </div>
                        <div class="bg-base-900 rounded p-2">
                            <div class="text-text-tertiary mb-0.5">الحد الأقصى</div>
                            <div class="num-cell text-text-primary">{fmtEgp(offer.max_amount_egp)}</div>
                        </div>
                    </div>

                    <div class="flex items-center justify-between text-xs">
                        <div class="flex items-center gap-2">
                            <span class="pill-success">{offer.completion_rate}% إكمال</span>
                            <span class="pill-muted">{offer.total_trades} صفقة</span>
                        </div>
                        <span class="text-text-tertiary">{fmtRelative(offer.created_at)}</span>
                    </div>

                    <div class="mt-3 pt-3 border-t border-base-700 flex items-center justify-between">
                        <div class="flex gap-1 flex-wrap">
                            {#each offer.payment_methods.slice(0, 3) as pm}
                                <span class="pill-muted text-[10px]">{paymentLabel(pm)}</span>
                            {/each}
                            {#if offer.payment_methods.length > 3}
                                <span class="pill-muted text-[10px]">+{offer.payment_methods.length - 3}</span>
                            {/if}
                        </div>
                        <span class="text-accent-blue text-xs group-hover:underline">عرض ←</span>
                    </div>
                </a>
            {/each}
        </div>
        <div class="text-xs text-text-tertiary text-center">* السعر يتحدد عند فتح الصفقة بناءً على هامش السعر المحدد من صاحب العرض وسعر السوق الحالي</div>
    {/if}
</div>
