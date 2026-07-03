<script lang="ts">
    import { onMount } from 'svelte';
    import { tickers, availablePairs, circuitOpen, myOrders, myWallets } from '$lib/stores';
    import { wallet } from '$lib/api';
    import { fmtEgp, fmtPrice, fmtQty } from '$lib/format';
    import type { Wallet } from '$lib/types';
    import CircuitBanner from '$lib/components/CircuitBanner.svelte';
    import { _t } from '$lib/i18n';

    let loading = true;
    let error = '';

    onMount(async () => {
        try {
            const ws = await wallet.list();
            myWallets.set(ws);
        } catch (e: any) {
            if (e.status !== 401) error = e.message;
        } finally {
            loading = false;
        }
    });

    $: wallets = $myWallets;
    $: egpBalance = wallets.find((w) => w.asset_symbol === 'EGP')?.balance || '0';
    $: cryptoValue = wallets
        .filter((w) => w.asset_symbol !== 'EGP')
        .reduce((sum, w) => {
            const ticker = $tickers[`${w.asset_symbol}_EGP`];
            if (!ticker) return sum;
            return sum + Number(w.balance) * Number(ticker.derived_egp_price);
        }, 0);
    $: totalValue = Number(egpBalance) + cryptoValue;

    $: pairCards = ($availablePairs.length ? $availablePairs : ['BTC_EGP', 'ETH_EGP', 'USDT_EGP']).map((p) => ({
        pair: p,
        ticker: $tickers[p],
    }));
</script>

<svelte:head><title>لوحة التحكم · منصة الجنيه</title></svelte:head>

<div class="space-y-6">
    <!-- العنوان -->
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">لوحة التحكم</h1>
            <p class="text-sm text-text-secondary mt-1">نظرة شاملة على محفظتك وأسواق التداول</p>
        </div>
    </div>

    <!-- بطاقات إحصائية -->
    <section class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div class="card-default relative overflow-hidden">
            <div class="absolute top-0 left-0 w-32 h-32 bg-accent-blue/5 rounded-full -translate-x-16 -translate-y-16"></div>
            <div class="relative">
                <div class="flex items-center justify-between mb-2">
                    <div class="text-xs text-text-tertiary uppercase tracking-wider">قيمة المحفظة</div>
                    <svg class="w-4 h-4 text-accent-blue" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" /></svg>
                </div>
                <div class="text-2xl font-mono font-bold text-text-primary">{fmtEgp(totalValue.toFixed(2))}</div>
                <div class="text-xs text-text-secondary mt-1">الجنيه والعملات الرقمية مجتمعة</div>
            </div>
        </div>
        <div class="card-default relative overflow-hidden">
            <div class="absolute top-0 left-0 w-32 h-32 bg-accent-green/5 rounded-full -translate-x-16 -translate-y-16"></div>
            <div class="relative">
                <div class="flex items-center justify-between mb-2">
                    <div class="text-xs text-text-tertiary uppercase tracking-wider">رصيد الجنيه</div>
                    <svg class="w-4 h-4 text-accent-green" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                </div>
                <div class="text-2xl font-mono font-bold text-accent-green">{fmtEgp(egpBalance)}</div>
                <div class="text-xs text-text-secondary mt-1">متاح للتداول</div>
            </div>
        </div>
        <div class="card-default relative overflow-hidden">
            <div class="absolute top-0 left-0 w-32 h-32 bg-accent-purple/5 rounded-full -translate-x-16 -translate-y-16"></div>
            <div class="relative">
                <div class="flex items-center justify-between mb-2">
                    <div class="text-xs text-text-tertiary uppercase tracking-wider">أرصدة العملات الرقمية</div>
                    <svg class="w-4 h-4 text-accent-purple" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" /></svg>
                </div>
                <div class="text-2xl font-mono font-bold text-accent-purple">{fmtEgp(cryptoValue.toFixed(2))}</div>
                <div class="text-xs text-text-secondary mt-1">القيمة السوقية</div>
            </div>
        </div>
    </section>

    <!-- الأسواق -->
    <section>
        <div class="flex items-center justify-between mb-3">
            <h2 class="text-lg font-semibold text-text-primary">الأسواق</h2>
            <a href="/trade/BTC_EGP" class="text-xs text-accent-blue hover:underline">عرض شاشة التداول ←</a>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            {#each pairCards as card}
                <a href="/trade/{card.pair}" class="card-default hover:border-accent-blue transition-all hover:shadow-lg hover:shadow-accent-blue/10 block group">
                    <div class="flex items-center justify-between mb-3">
                        <div class="flex items-center gap-2">
                            <div class="w-10 h-10 rounded-full bg-gradient-to-br from-base-700 to-base-600 flex items-center justify-center text-xs font-bold text-accent-blue group-hover:scale-110 transition-transform">
                                {card.pair.split('_')[0].slice(0, 3)}
                            </div>
                            <div>
                                <div class="text-sm font-semibold text-text-primary">{card.pair.replace('_', '/')}</div>
                                <div class="text-xs text-text-tertiary">مقابل الجنيه</div>
                            </div>
                        </div>
                        {#if card.ticker}
                            <div class="text-left">
                                <div class="text-xs text-text-tertiary">السعر</div>
                                <div class="text-sm font-mono font-bold text-text-primary">{fmtEgp(card.ticker.derived_egp_price)}</div>
                            </div>
                        {/if}
                    </div>
                    {#if card.ticker}
                        <div class="flex items-center justify-between text-xs">
                            <span class="text-text-secondary">أعلى طلب: <span class="text-accent-green num-cell">{fmtEgp(card.ticker.bid)}</span></span>
                            <span class="text-text-secondary">أقل عرض: <span class="text-accent-red num-cell">{fmtEgp(card.ticker.ask)}</span></span>
                        </div>
                    {:else}
                        <div class="text-xs text-text-tertiary">بانتظار تدفق الأسعار...</div>
                    {/if}
                </a>
            {/each}
        </div>
    </section>

    <!-- المحافظ -->
    <section>
        <div class="flex items-center justify-between mb-3">
            <h2 class="text-lg font-semibold text-text-primary">محافظك</h2>
            <a href="/wallet" class="text-xs text-accent-blue hover:underline">إدارة ←</a>
        </div>
        <div class="card-default overflow-x-auto">
            {#if loading}
                <div class="text-center py-8 text-text-tertiary text-sm">جارٍ التحميل...</div>
            {:else if wallets.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">
                    لا توجد محافظ. <a href="/login" class="text-accent-blue hover:underline">سجّل الدخول</a> لعرض الأرصدة.
                </div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الأصل</th>
                            <th>النوع</th>
                            <th class="num-cell">المتاح</th>
                            <th class="num-cell">المحجوز</th>
                            <th class="num-cell">الإجمالي</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each wallets as w}
                            <tr>
                                <td class="font-bold">{w.asset_symbol}</td>
                                <td><span class="pill-muted">{w.wallet_type === 'fiat' ? 'نقدي' : 'رقمي'}</span></td>
                                <td class="num-cell">{w.wallet_type === 'fiat' ? fmtEgp(w.balance) : fmtQty(w.balance, 8)}</td>
                                <td class="num-cell text-text-secondary">{w.wallet_type === 'fiat' ? fmtEgp(w.locked_balance) : fmtQty(w.locked_balance, 8)}</td>
                                <td class="num-cell">{w.wallet_type === 'fiat' ? fmtEgp((Number(w.balance) + Number(w.locked_balance)).toString()) : fmtQty((Number(w.balance) + Number(w.locked_balance)).toString(), 8)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    </section>
</div>
