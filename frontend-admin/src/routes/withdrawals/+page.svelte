<script lang="ts">
    import { onMount } from 'svelte';
    import { manualTx, statusPill } from '$lib/api';
    import { fmtQty, fmtDate, fmtRelative } from '$lib/format';
    import type { ManualTransaction, ManualTxStatus } from '$lib/types';

    let items: Array<{ tx: ManualTransaction; queue_position: number }> = [];
    let loading = true;
    let error = '';
    let filter: 'all' | ManualTxStatus = 'pending';
    let selected: ManualTransaction | null = null;
    let reviewNote = '';
    let reviewStatus: ManualTxStatus = 'under_review';
    let txHash = '';
    let reviewSubmitting = false;
    let reviewError = '';
    let reviewSuccess = '';

    async function load() {
        try {
            const params = filter === 'all' ? { tx_type: 'withdrawal' as const } : { tx_type: 'withdrawal' as const, status: filter };
            const res = await manualTx.list(params);
            items = res.items;
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

    function openReview(tx: ManualTransaction) {
        selected = tx;
        reviewNote = tx.admin_note || '';
        reviewStatus = tx.status === 'pending' ? 'under_review' : tx.status;
        txHash = tx.tx_hash || '';
        reviewError = '';
        reviewSuccess = '';
    }

    async function submitReview() {
        if (!selected) return;
        if (reviewStatus === 'completed' && !txHash) {
            reviewError = 'On-chain tx hash required to mark withdrawal as completed';
            return;
        }
        reviewSubmitting = true;
        reviewError = '';
        try {
            await manualTx.review(selected.id, {
                status: reviewStatus,
                admin_note: reviewNote,
                tx_hash: txHash || undefined,
            });
            reviewSuccess = `Marked as ${reviewStatus}`;
            await load();
            setTimeout(() => { selected = null; reviewSuccess = ''; }, 1500);
        } catch (e: any) {
            reviewError = e.message;
        } finally {
            reviewSubmitting = false;
        }
    }
</script>

<svelte:head><title>Withdrawals Queue · Admin</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">Crypto Withdrawals Queue</h1>
            <p class="text-sm text-text-secondary mt-1">Process withdrawal requests — broadcast on-chain tx, then mark completed</p>
        </div>
        <div class="flex items-center gap-2 text-xs flex-wrap">
            {#each ['pending', 'under_review', 'approved', 'completed', 'rejected', 'all'] as f}
                <button
                    class="px-3 py-1.5 rounded-md font-medium {filter === f ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:bg-base-700/50'}"
                    on:click={() => { filter = f as any; load(); }}>
                    {f.replace('_', ' ')}
                </button>
            {/each}
        </div>
    </div>

    {#if loading}
        <div class="text-center py-8 text-text-tertiary">Loading...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if items.length === 0}
        <div class="card-default text-center py-12 text-text-tertiary text-sm">No withdrawal requests in this view</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>Position</th>
                        <th>User</th>
                        <th>Asset</th>
                        <th class="text-right">Amount</th>
                        <th>Destination</th>
                        <th>Status</th>
                        <th class="text-right">Created</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as item}
                        <tr class="cursor-pointer" on:click={() => openReview(item.tx)}>
                            <td>
                                {#if item.queue_position > 0}
                                    <span class="pill-warning">#{item.queue_position}</span>
                                {:else}
                                    <span class="pill-muted">—</span>
                                {/if}
                            </td>
                            <td class="text-xs num-cell text-text-secondary">{item.tx.user_id.slice(0, 8)}…</td>
                            <td class="font-semibold">{item.tx.asset_symbol}</td>
                            <td class="num-cell font-semibold">{fmtQty(item.tx.amount, 8)}</td>
                            <td class="text-xs num-cell text-text-secondary max-w-[200px] truncate" title={item.tx.destination || ''}>{item.tx.destination || '—'}</td>
                            <td><span class={statusPill(item.tx.status)}>{item.tx.status.replace('_', ' ')}</span></td>
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(item.tx.created_at)}>{fmtRelative(item.tx.created_at)}</td>
                            <td><button class="text-xs text-accent-blue hover:underline">Review →</button></td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

{#if selected}
    <div class="fixed inset-0 z-50 bg-base-900/70 flex items-center justify-center p-4" on:click|self={() => (selected = null)}>
        <div class="bg-base-800 border border-base-600 rounded-lg w-full max-w-lg p-6">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold text-text-primary">Review Withdrawal</h3>
                <button on:click={() => (selected = null)} class="text-text-tertiary hover:text-text-primary">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>
                </button>
            </div>

            <div class="space-y-3 text-sm">
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <div class="text-xs text-text-tertiary uppercase">User ID</div>
                        <div class="num-cell text-text-secondary text-xs">{selected.user_id}</div>
                    </div>
                    <div>
                        <div class="text-xs text-text-tertiary uppercase">Amount</div>
                        <div class="num-cell text-accent-red font-semibold">{fmtQty(selected.amount, 8)} {selected.asset_symbol}</div>
                    </div>
                    <div class="col-span-2">
                        <div class="text-xs text-text-tertiary uppercase">Destination Address</div>
                        <div class="num-cell text-text-primary text-xs break-all">{selected.destination}</div>
                    </div>
                    <div>
                        <div class="text-xs text-text-tertiary uppercase">Created</div>
                        <div class="text-text-secondary text-xs">{fmtDate(selected.created_at)}</div>
                    </div>
                </div>

                <div>
                    <label class="label" for="status">New Status</label>
                    <select id="status" bind:value={reviewStatus} class="input">
                        <option value="under_review">Under Review</option>
                        <option value="approved">Approved (in progress)</option>
                        <option value="completed">Completed (broadcast on-chain)</option>
                        <option value="rejected">Rejected (refund locked funds)</option>
                        <option value="failed">Failed</option>
                    </select>
                </div>

                {#if reviewStatus === 'completed'}
                    <div>
                        <label class="label" for="hash">On-chain TX Hash <span class="text-accent-red">*</span></label>
                        <input id="hash" type="text" bind:value={txHash} class="input" placeholder="0x..." />
                    </div>
                {/if}

                <div>
                    <label class="label" for="note">Admin Note</label>
                    <textarea id="note" bind:value={reviewNote} class="input min-h-[80px]" placeholder="Optional note..."></textarea>
                </div>

                {#if reviewError}<div class="text-xs text-accent-red">{reviewError}</div>{/if}
                {#if reviewSuccess}<div class="text-xs text-accent-green">{reviewSuccess}</div>{/if}

                <div class="flex gap-2 pt-2">
                    <button class="btn-ghost flex-1" on:click={() => (selected = null)}>Cancel</button>
                    <button class="btn-primary flex-1" on:click={submitReview} disabled={reviewSubmitting}>
                        {reviewSubmitting ? 'Submitting...' : 'Submit Review'}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}
