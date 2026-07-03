<script lang="ts">
    import { onMount } from 'svelte';
    import { overview } from '$lib/api';
    import { fmtEgp, fmtQty } from '$lib/format';
    import type { LiquidityRow } from '$lib/types';

    let rows: LiquidityRow[] = [];
    let loading = true;
    let error = '';

    async function load() {
        try {
            const res = await overview.liquidity();
            rows = res.liquidity;
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        load();
        const t = setInterval(load, 10000);
        return () => clearInterval(t);
    });

    // Bar chart scale
    $: maxBal = Math.max(...rows.map((r) => Number(r.balance)), 1);
</script>

<svelte:head><title>Liquidity · Admin</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">System Liquidity Monitor</h1>
        <p class="text-sm text-text-secondary mt-1">Aggregate user balances across all assets — updates every 10 seconds</p>
    </div>

    {#if loading}
        <div class="text-center py-8 text-text-tertiary">Loading...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else}
        <!-- Visual bars -->
        <section class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each rows as r}
                <div class="card-default">
                    <div class="flex items-center justify-between mb-2">
                        <div class="font-semibold text-text-primary">{r.asset}</div>
                        <span class="pill-muted">{r.asset === 'EGP' ? 'fiat' : 'crypto'}</span>
                    </div>
                    <div class="text-2xl font-mono font-bold text-accent-blue mb-2">
                        {r.asset === 'EGP' ? fmtEgp(r.balance) : fmtQty(r.balance, 8)}
                    </div>
                    <div class="relative h-2 bg-base-700 rounded overflow-hidden mb-2">
                        <div class="absolute inset-y-0 left-0 bg-gradient-to-r from-accent-blue to-accent-cyan" style="width: {(Number(r.balance) / maxBal) * 100}%"></div>
                    </div>
                    <div class="flex justify-between text-xs">
                        <span class="text-text-secondary">Locked: <span class="num-cell text-accent-yellow">{r.asset === 'EGP' ? fmtEgp(r.locked) : fmtQty(r.locked, 8)}</span></span>
                        <span class="text-text-secondary">Available: <span class="num-cell text-accent-green">{r.asset === 'EGP' ? fmtEgp(r.available) : fmtQty(r.available, 8)}</span></span>
                    </div>
                </div>
            {/each}
        </section>

        <!-- Detailed table -->
        <section>
            <h2 class="text-sm font-semibold text-text-secondary uppercase tracking-wider mb-3">Detailed Breakdown</h2>
            <div class="card-default overflow-x-auto">
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>Asset</th>
                            <th class="text-right">Total Balance</th>
                            <th class="text-right">Locked</th>
                            <th class="text-right">Available</th>
                            <th class="text-right">Lock Ratio</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each rows as r}
                            <tr>
                                <td class="font-semibold">{r.asset}</td>
                                <td class="num-cell">{r.asset === 'EGP' ? fmtEgp(r.balance) : fmtQty(r.balance, 8)}</td>
                                <td class="num-cell text-accent-yellow">{r.asset === 'EGP' ? fmtEgp(r.locked) : fmtQty(r.locked, 8)}</td>
                                <td class="num-cell text-accent-green">{r.asset === 'EGP' ? fmtEgp(r.available) : fmtQty(r.available, 8)}</td>
                                <td class="num-cell text-text-tertiary">
                                    {#if Number(r.balance) > 0}
                                        {((Number(r.locked) / Number(r.balance)) * 100).toFixed(2)}%
                                    {:else}
                                        —
                                    {/if}
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>
    {/if}
</div>
