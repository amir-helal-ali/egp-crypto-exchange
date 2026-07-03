<script lang="ts">
    import { onMount } from 'svelte';
    import { wallet, txStatusPill, txTypeLabel } from '$lib/api';
    import { fmtEgp, fmtQty, fmtDate } from '$lib/format';
    import type { Wallet, ManualTransaction } from '$lib/types';

    let wallets: Wallet[] = [];
    let deposits: ManualTransaction[] = [];
    let withdrawals: ManualTransaction[] = [];
    let loading = true;
    let error = '';

    // Deposit form
    let depAmount = '';
    let depReference = '';
    let depSubmitting = false;
    let depError = '';
    let depSuccess = '';

    // Withdrawal form
    let wdAsset = 'BTC';
    let wdAmount = '';
    let wdDestination = '';
    let wdSubmitting = false;
    let wdError = '';
    let wdSuccess = '';

    let activeTab: 'deposit' | 'withdraw' | 'history' = 'deposit';

    async function loadAll() {
        try {
            const [ws, ds, wds] = await Promise.all([
                wallet.list(),
                wallet.listDeposits(),
                wallet.listWithdrawals(),
            ]);
            wallets = ws;
            deposits = ds;
            withdrawals = wds;
        } catch (e: any) {
            if (e.status !== 401) error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(() => loadAll());

    async function submitDeposit() {
        depSubmitting = true;
        depError = '';
        depSuccess = '';
        try {
            const tx = await wallet.requestDeposit(depReference, depAmount);
            depSuccess = `Deposit request submitted (ID: ${tx.id.slice(0, 8)}). Position in queue will be visible here shortly.`;
            depAmount = '';
            depReference = '';
            await loadAll();
        } catch (e: any) {
            depError = e.message;
        } finally {
            depSubmitting = false;
        }
    }

    async function submitWithdrawal() {
        wdSubmitting = true;
        wdError = '';
        wdSuccess = '';
        try {
            const tx = await wallet.requestWithdrawal(wdDestination, wdAmount, wdAsset);
            wdSuccess = `Withdrawal request submitted (ID: ${tx.id.slice(0, 8)}). Funds are locked until admin approval.`;
            wdAmount = '';
            wdDestination = '';
            await loadAll();
        } catch (e: any) {
            wdError = e.message;
        } finally {
            wdSubmitting = false;
        }
    }

    $: allTx = [...deposits, ...withdrawals].sort((a, b) =>
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
    );
</script>

<svelte:head><title>Wallet · EGP Exchange</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-text-primary">Wallet</h1>
    </div>

    {#if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {/if}

    <!-- Balances -->
    <section>
        <h2 class="text-sm font-semibold text-text-secondary uppercase tracking-wider mb-3">Balances</h2>
        <div class="card-default overflow-x-auto">
            {#if loading}
                <div class="text-center py-8 text-text-tertiary">Loading...</div>
            {:else if wallets.length === 0}
                <div class="text-center py-8 text-text-tertiary">No wallets</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr><th>Asset</th><th>Type</th><th class="text-right">Available</th><th class="text-right">Locked</th><th class="text-right">Total</th></tr>
                    </thead>
                    <tbody>
                        {#each wallets as w}
                            <tr>
                                <td class="font-semibold">{w.asset_symbol}</td>
                                <td><span class="pill-muted">{w.wallet_type}</span></td>
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

    <!-- Tabs -->
    <section>
        <div class="border-b border-base-700 mb-4">
            <nav class="flex gap-4">
                <button class="py-2 text-sm font-medium border-b-2 {activeTab === 'deposit' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (activeTab = 'deposit')}>Deposit EGP</button>
                <button class="py-2 text-sm font-medium border-b-2 {activeTab === 'withdraw' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (activeTab = 'withdraw')}>Withdraw Crypto</button>
                <button class="py-2 text-sm font-medium border-b-2 {activeTab === 'history' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (activeTab = 'history')}>Transaction History</button>
            </nav>
        </div>

        {#if activeTab === 'deposit'}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="card-default">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">New EGP Deposit</h3>
                    <p class="text-xs text-text-secondary mb-4 leading-relaxed">
                        Transfer EGP to our bank account, then submit a deposit request with the bank reference. An administrator will verify and credit your wallet. This is a manual process — requests appear in the admin queue and update in real-time below.
                    </p>
                    <div class="space-y-3">
                        <div>
                            <label class="label" for="dep-amount">Amount (EGP)</label>
                            <input id="dep-amount" type="number" step="0.01" min="100" bind:value={depAmount} class="input" placeholder="100.00" />
                        </div>
                        <div>
                            <label class="label" for="dep-ref">Bank Reference / Receipt #</label>
                            <input id="dep-ref" type="text" bind:value={depReference} class="input" placeholder="TRX20240115001" />
                        </div>
                        {#if depError}<div class="text-xs text-accent-red">{depError}</div>{/if}
                        {#if depSuccess}<div class="text-xs text-accent-green">{depSuccess}</div>{/if}
                        <button class="w-full btn-success" on:click={submitDeposit} disabled={depSubmitting}>
                            {depSubmitting ? 'Submitting...' : 'Submit Deposit Request'}
                        </button>
                    </div>
                </div>

                <div class="card-default">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">Bank Transfer Instructions</h3>
                    <div class="text-xs space-y-2 text-text-secondary">
                        <div><span class="text-text-tertiary">Bank:</span> Banque Misr</div>
                        <div><span class="text-text-tertiary">Account Name:</span> EGP Exchange LLC</div>
                        <div><span class="text-text-tertiary">Account Number:</span> 1000-2000-3000-4000</div>
                        <div><span class="text-text-tertiary">IBAN:</span> EG00 1000 2000 3000 4000 5000</div>
                        <div><span class="text-text-tertiary">SWIFT:</span> BMISEGCX</div>
                        <div><span class="text-text-tertiary">Reference:</span> Use your email or deposit reference</div>
                    </div>
                    <div class="mt-4 text-xs text-accent-yellow bg-accent-yellow/10 border border-accent-yellow/30 rounded px-3 py-2">
                        <strong>Important:</strong> Always include your reference. Without it, we cannot match your transfer.
                    </div>
                </div>
            </div>
        {:else if activeTab === 'withdraw'}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="card-default">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">New Crypto Withdrawal</h3>
                    <p class="text-xs text-text-secondary mb-4 leading-relaxed">
                        Crypto withdrawals are processed manually by our admin team after a security review. Your funds will be locked immediately upon submission and released if the request is rejected.
                    </p>
                    <div class="space-y-3">
                        <div>
                            <label class="label" for="wd-asset">Asset</label>
                            <select id="wd-asset" bind:value={wdAsset} class="input">
                                <option value="BTC">BTC</option>
                                <option value="ETH">ETH</option>
                                <option value="USDT">USDT</option>
                            </select>
                        </div>
                        <div>
                            <label class="label" for="wd-amount">Amount</label>
                            <input id="wd-amount" type="number" step="any" min="0" bind:value={wdAmount} class="input" placeholder="0.00" />
                        </div>
                        <div>
                            <label class="label" for="wd-dest">Destination Address</label>
                            <input id="wd-dest" type="text" bind:value={wdDestination} class="input" placeholder="bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" />
                        </div>
                        {#if wdError}<div class="text-xs text-accent-red">{wdError}</div>{/if}
                        {#if wdSuccess}<div class="text-xs text-accent-green">{wdSuccess}</div>{/if}
                        <button class="w-full btn-danger" on:click={submitWithdrawal} disabled={wdSubmitting}>
                            {wdSubmitting ? 'Submitting...' : 'Submit Withdrawal Request'}
                        </button>
                    </div>
                </div>

                <div class="card-default">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">Security Notes</h3>
                    <ul class="text-xs text-text-secondary space-y-2 list-disc list-inside">
                        <li>Withdrawals are reviewed within 24 hours during business days.</li>
                        <li>Always double-check the destination address — blockchain transactions are irreversible.</li>
                        <li>For large withdrawals, KYC level 2 may be required.</li>
                        <li>Network fees are deducted from the withdrawn amount.</li>
                        <li>Suspicious activity may be flagged for additional verification.</li>
                    </ul>
                </div>
            </div>
        {:else}
            <div class="card-default overflow-x-auto">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">Transaction History</h3>
                {#if allTx.length === 0}
                    <div class="text-center py-8 text-text-tertiary text-sm">No transactions yet</div>
                {:else}
                    <table class="table-pro">
                        <thead>
                            <tr>
                                <th>Type</th>
                                <th>Asset</th>
                                <th class="text-right">Amount</th>
                                <th class="text-right">Fee</th>
                                <th>Status</th>
                                <th class="text-right">Reference / Destination</th>
                                <th class="text-right">Created</th>
                                <th class="text-right">Completed</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each allTx as tx}
                                <tr>
                                    <td><span class={tx.tx_type === 'deposit' ? 'pill-success' : 'pill-info'}>{txTypeLabel(tx.tx_type)}</span></td>
                                    <td class="font-medium">{tx.asset_symbol}</td>
                                    <td class="num-cell">{fmtQty(tx.amount, tx.asset_class === 'fiat' ? 2 : 8)}</td>
                                    <td class="num-cell text-text-secondary">{fmtQty(tx.fee, 8)}</td>
                                    <td><span class={txStatusPill(tx.status)}>{tx.status.replace('_', ' ')}</span></td>
                                    <td class="text-xs num-cell text-text-secondary">{tx.reference || tx.destination || '—'}</td>
                                    <td class="num-cell text-text-tertiary text-xs">{fmtDate(tx.created_at)}</td>
                                    <td class="num-cell text-text-tertiary text-xs">{fmtDate(tx.completed_at)}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                {/if}
            </div>
        {/if}
    </section>
</div>
