<script lang="ts">
    import { onMount } from 'svelte';
    import { tickers, availablePairs, connectMarketWs } from '$lib/stores';
    import { fmtEgp, fmtPrice } from '$lib/format';

    onMount(() => {
        connectMarketWs();
    });

    $: pairs = ($availablePairs.length ? $availablePairs : ['BTC_EGP', 'ETH_EGP', 'USDT_EGP']);
    $: cards = pairs.map((p) => ({
        pair: p,
        ticker: $tickers[p],
        base: p.split('_')[0],
        quote: p.split('_')[1] || 'EGP',
    }));
</script>

<svelte:head><title>الأسواق · منصة الجنيه</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">الأسواق</h1>
        <p class="text-sm text-text-secondary mt-1">كل أزواج التداول المتاحة بأسعار لحظية</p>
    </div>

    <div class="card-default overflow-x-auto">
        <table class="table-pro">
            <thead>
                <tr>
                    <th>الزوج</th>
                    <th class="num-cell">آخر سعر</th>
                    <th class="num-cell">أعلى طلب</th>
                    <th class="num-cell">أقل عرض</th>
                    <th class="num-cell">الفرق</th>
                    <th></th>
                </tr>
            </thead>
            <tbody>
                {#each cards as card}
                    {@const spread = card.ticker ? (Number(card.ticker.ask) - Number(card.ticker.bid)).toFixed(2) : '—'}
                    <tr class="cursor-pointer hover:bg-base-700/50">
                        <td>
                            <div class="flex items-center gap-2">
                                <div class="w-9 h-9 rounded-full bg-gradient-to-br from-base-700 to-base-600 flex items-center justify-center text-xs font-bold text-accent-blue">
                                    {card.base.slice(0, 3)}
                                </div>
                                <div>
                                    <div class="font-bold text-text-primary">{card.base}/{card.quote}</div>
                                    <div class="text-xs text-text-tertiary">مقابل الجنيه</div>
                                </div>
                            </div>
                        </td>
                        <td class="num-cell font-semibold text-text-primary">
                            {card.ticker ? fmtEgp(card.ticker.derived_egp_price) : '—'}
                        </td>
                        <td class="num-cell text-accent-green">{card.ticker ? fmtEgp(card.ticker.bid) : '—'}</td>
                        <td class="num-cell text-accent-red">{card.ticker ? fmtEgp(card.ticker.ask) : '—'}</td>
                        <td class="num-cell text-text-tertiary">{spread}</td>
                        <td>
                            <div class="flex gap-2 justify-end">
                                <a href="/trade/{card.pair}" class="text-xs text-accent-blue hover:underline">تداول ←</a>
                                <a href="/futures/{card.pair}" class="text-xs text-accent-purple hover:underline">عقود ←</a>
                            </div>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>

    {#if Object.keys($tickers).length === 0}
        <div class="card-default text-center py-12">
            <div class="text-text-tertiary text-sm">بانتظار تدفق الأسعار من Binance...</div>
            <div class="mt-4">
                <div class="inline-block w-6 h-6 border-2 border-accent-blue border-t-transparent rounded-full animate-spin"></div>
            </div>
        </div>
    {/if}
</div>
