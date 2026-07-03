<script lang="ts">
    import { onMount } from 'svelte';
    import { tickers, availablePairs, circuitOpen } from '$lib/stores';
    import { wallet, trading } from '$lib/api';
    import { fmtEgp, fmtPrice, fmtQty, fmtAsset } from '$lib/format';
    import type { Wallet } from '$lib/types';
    import CircuitBanner from '$lib/components/CircuitBanner.svelte';

    let wallets: Wallet[] = [];
    let loading = true;
    let error = '';

    async function loadWallets() {
        try {
            wallets = await wallet.list();
        } catch (e: any) {
            if (e.status !== 401) error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadWallets();
    });

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

<svelte:head><title>Dashboard · EGP Exchange</title></svelte:head>

<div class="space-y-6">
    <!-- Hero stats -->
    <section class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div class="card-default">
            <div class="text-xs text-text-tertiary uppercase tracking-wider mb-1">Portfolio Value</div>
            <div class="text-2xl font-mono font-bold text-text-primary">{fmtEgp(totalValue.toFixed(2))}</div>
            <div class="text-xs text-text-secondary mt-1">EGP + Crypto combined</div>
        </div>
        <div class="card-default">
            <div class="text-xs text-text-tertiary uppercase tracking-wider mb-1">EGP Balance</div>
            <div class="text-2xl font-mono font-bold text-accent-green">{fmtEgp(egpBalance)}</div>
            <div class="text-xs text-text-secondary mt-1">Available for trading</div>
        </div>
        <div class="card-default">
            <div class="text-xs text-text-tertiary uppercase tracking-wider mb-1">Crypto Holdings</div>
            <div class="text-2xl font-mono font-bold text-accent-blue">{fmtEgp(cryptoValue.toFixed(2))}</div>
            <div class="text-xs text-text-secondary mt-1">Mark-to-market value</div>
        </div>
    </section>

    <!-- Market overview -->
    <section>
        <div class="flex items-center justify-between mb-3">
            <h2 class="text-lg font-semibold text-text-primary">Markets</h2>
            <a href="/trade/BTC_EGP" class="text-xs text-accent-blue hover:underline">View trade screen →</a>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            {#each pairCards as card}
                <a href="/trade/{card.pair}" class="card-default hover:border-accent-blue transition-colors block">
                    <div class="flex items-center justify-between mb-2">
                        <div class="flex items-center gap-2">
                            <div class="w-8 h-8 rounded-full bg-base-700 flex items-center justify-center text-xs font-bold text-accent-blue">
                                {card.pair.split('_')[0].slice(0, 3)}
                            </div>
                            <div>
                                <div class="text-sm font-semibold text-text-primary">{card.pair.replace('_', '/')}</div>
                                <div class="text-xs text-text-tertiary">vs EGP</div>
                            </div>
                        </div>
                    </div>
                    {#if card.ticker}
                        <div class="text-2xl font-mono font-bold text-text-primary">{fmtEgp(card.ticker.derived_egp_price)}</div>
                        <div class="flex items-center gap-3 mt-2 text-xs">
                            <span class="text-text-secondary">Bid: <span class="text-accent-green num-cell">{fmtEgp(card.ticker.bid)}</span></span>
                            <span class="text-text-secondary">Ask: <span class="text-accent-red num-cell">{fmtEgp(card.ticker.ask)}</span></span>
                        </div>
                    {:else}
                        <div class="text-2xl font-mono font-bold text-text-tertiary">—</div>
                        <div class="text-xs text-text-tertiary mt-2">Waiting for feed...</div>
                    {/if}
                </a>
            {/each}
        </div>
    </section>

    <!-- Wallets summary -->
    <section>
        <div class="flex items-center justify-between mb-3">
            <h2 class="text-lg font-semibold text-text-primary">Your Wallets</h2>
            <a href="/wallet" class="text-xs text-accent-blue hover:underline">Manage →</a>
        </div>
        <div class="card-default overflow-x-auto">
            {#if loading}
                <div class="text-center py-8 text-text-tertiary text-sm">Loading wallets...</div>
            {:else if wallets.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">
                    No wallets found. <a href="/login" class="text-accent-blue hover:underline">Sign in</a> to view balances.
                </div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>Asset</th>
                            <th>Type</th>
                            <th class="text-right">Available</th>
                            <th class="text-right">Locked</th>
                            <th class="text-right">Total</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each wallets as w}
                            <tr>
                                <td class="font-medium">{w.asset_symbol}</td>
                                <td><span class="pill-muted">{w.wallet_type}</span></td>
                                <td class="num-cell">{fmtQty(w.balance, w.wallet_type === 'fiat' ? 2 : 8)}</td>
                                <td class="num-cell text-text-secondary">{fmtQty(w.locked_balance, w.wallet_type === 'fiat' ? 2 : 8)}</td>
                                <td class="num-cell">{fmtQty((Number(w.balance) + Number(w.locked_balance)).toString(), w.wallet_type === 'fiat' ? 2 : 8)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    </section>
</div>
