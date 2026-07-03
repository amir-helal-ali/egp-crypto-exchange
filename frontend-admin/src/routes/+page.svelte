<script lang="ts">
    import { onMount } from 'svelte';
    import { overview } from '$lib/api';
    import { fmtEgp, fmtNum, fmtQty, fmtRelative } from '$lib/format';
    import type { Overview } from '$lib/types';

    let data: Overview | null = null;
    let loading = true;
    let error = '';

    async function load() {
        try {
            data = await overview.get();
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        load();
        const t = setInterval(load, 5000);
        return () => clearInterval(t);
    });
</script>

<svelte:head><title>Overview · Admin</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">Overview</h1>
        <p class="text-sm text-text-secondary mt-1">Real-time system status and key metrics</p>
    </div>

    {#if loading}
        <div class="text-center py-8 text-text-tertiary">Loading...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if data}
        <!-- Stat cards -->
        <section class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">Users</div>
                <div class="text-2xl font-mono font-bold text-text-primary mt-1">{fmtNum(data.users, 0)}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">Open Orders</div>
                <div class="text-2xl font-mono font-bold text-accent-blue mt-1">{fmtNum(data.orders.open, 0)}</div>
                <div class="text-xs text-text-tertiary">of {data.orders.total}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">Total Trades</div>
                <div class="text-2xl font-mono font-bold text-accent-green mt-1">{fmtNum(data.trades, 0)}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">Pending Deposits</div>
                <div class="text-2xl font-mono font-bold text-accent-yellow mt-1">{fmtNum(data.pending.deposits, 0)}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">Pending Withdrawals</div>
                <div class="text-2xl font-mono font-bold text-accent-yellow mt-1">{fmtNum(data.pending.withdrawals, 0)}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">Circuit Breaker</div>
                {#if data.circuit_breaker_open}
                    <div class="text-2xl font-bold text-accent-red mt-1">OPEN</div>
                    <div class="text-xs text-accent-red">Trading halted</div>
                {:else}
                    <div class="text-2xl font-bold text-accent-green mt-1">CLOSED</div>
                    <div class="text-xs text-accent-green">Feed healthy</div>
                {/if}
            </div>
        </section>

        <!-- Liquidity table -->
        <section>
            <h2 class="text-sm font-semibold text-text-secondary uppercase tracking-wider mb-3">System Liquidity</h2>
            <div class="card-default overflow-x-auto">
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>Asset</th>
                            <th class="text-right">Total Balance</th>
                            <th class="text-right">Locked</th>
                            <th class="text-right">Available</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each data.liquidity as l}
                            <tr>
                                <td class="font-semibold">{l.asset}</td>
                                <td class="num-cell">{l.asset === 'EGP' ? fmtEgp(l.balance) : fmtQty(l.balance, 8)}</td>
                                <td class="num-cell text-text-secondary">{l.asset === 'EGP' ? fmtEgp(l.locked) : fmtQty(l.locked, 8)}</td>
                                <td class="num-cell text-accent-green">{l.asset === 'EGP' ? fmtEgp((Number(l.balance) - Number(l.locked)).toString()) : fmtQty((Number(l.balance) - Number(l.locked)).toString(), 8)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>

        <!-- Quick links -->
        <section class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <a href="/deposits" class="card-default hover:border-accent-yellow transition-colors block">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold text-text-primary">Process Deposits</div>
                        <div class="text-xs text-text-tertiary mt-1">Verify EGP bank transfers and credit wallets</div>
                    </div>
                    <div class="text-2xl font-bold text-accent-yellow">{data.pending.deposits}</div>
                </div>
            </a>
            <a href="/withdrawals" class="card-default hover:border-accent-red transition-colors block">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold text-text-primary">Process Withdrawals</div>
                        <div class="text-xs text-text-tertiary mt-1">Release crypto withdrawals after review</div>
                    </div>
                    <div class="text-2xl font-bold text-accent-red">{data.pending.withdrawals}</div>
                </div>
            </a>
            <a href="/users" class="card-default hover:border-accent-blue transition-colors block">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold text-text-primary">Manage Users</div>
                        <div class="text-xs text-text-tertiary mt-1">View users, update KYC, suspend accounts</div>
                    </div>
                    <div class="text-2xl font-bold text-accent-blue">{data.users}</div>
                </div>
            </a>
        </section>
    {/if}
</div>
