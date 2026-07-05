<script lang="ts">
    import { onMount } from 'svelte';
    import { manualTx, statusPill } from '$lib/api';
    import { fmtEgp, fmtDate, fmtRelative } from '$lib/format';
    import type { ManualTransaction, ManualTxStatus } from '$lib/types';

    let items: Array<{ tx: ManualTransaction; queue_position: number }> = [];
    let loading = true;
    let error = '';
    let filter: string = 'pending';
    let selected: ManualTransaction | null = null;
    let reviewNote = '';
    let reviewStatus: ManualTxStatus = 'approved';
    let reviewSubmitting = false;
    let reviewError = '';
    let reviewSuccess = '';

    async function load() {
        try {
            const params = filter === 'all' ? { tx_type: 'deposit' as const } : { tx_type: 'deposit' as const, status: filter as ManualTxStatus };
            const res = await manualTx.list(params);
            items = res.items;
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    function openReview(tx: ManualTransaction) {
        selected = tx;
        reviewNote = tx.admin_note || '';
        reviewStatus = tx.status === 'pending' ? 'under_review' : tx.status;
        reviewError = '';
        reviewSuccess = '';
    }

    async function submitReview() {
        if (!selected) return;
        reviewSubmitting = true;
        reviewError = '';
        try {
            await manualTx.review(selected.id, {
                status: reviewStatus,
                admin_note: reviewNote,
            });
            reviewSuccess = `تم تحديث الحالة إلى: ${reviewStatus}`;
            await load();
            setTimeout(() => { selected = null; reviewSuccess = ''; }, 1500);
        } catch (e: any) {
            reviewError = e.message;
        } finally {
            reviewSubmitting = false;
        }
    }
</script>

<svelte:head><title>طابور الإيداعات · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">طابور إيداعات الجنيه</h1>
            <p class="text-sm text-text-secondary mt-1">كل طلبات الإيداع اليدوية، الأقدم أولاً</p>
        </div>
        <div class="flex items-center gap-2 text-xs flex-wrap">
            {#each ['pending', 'under_review', 'approved', 'completed', 'rejected', 'all'] as f}
                <button
                    class="px-3 py-1.5 rounded-md font-medium {filter === f ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:bg-base-700/50'}"
                    on:click={() => { filter = f; load(); }}>
                    {f === 'all' ? 'الكل' : f === 'pending' ? 'معلق' : f === 'under_review' ? 'قيد المراجعة' : f === 'approved' ? 'موافق' : f === 'completed' ? 'مكتمل' : 'مرفوض'}
                </button>
            {/each}
        </div>
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if items.length === 0}
        <div class="card-default text-center py-12 text-text-tertiary text-sm">لا توجد طلبات إيداع في هذا العرض</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>الموقع</th>
                        <th>المستخدم</th>
                        <th class="num-cell">المبلغ</th>
                        <th>المرجع</th>
                        <th>الحالة</th>
                        <th class="num-cell">منذ</th>
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
                            <td class="num-cell font-semibold">{fmtEgp(item.tx.amount)}</td>
                            <td class="text-text-secondary text-sm">{item.tx.reference || '—'}</td>
                            <td><span class={statusPill(item.tx.status)}>{
                                item.tx.status === 'pending' ? 'معلق' :
                                item.tx.status === 'under_review' ? 'قيد المراجعة' :
                                item.tx.status === 'approved' ? 'موافق' :
                                item.tx.status === 'completed' ? 'مكتمل' :
                                item.tx.status === 'rejected' ? 'مرفوض' : 'فشل'
                            }</span></td>
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(item.tx.created_at)}>{fmtRelative(item.tx.created_at)}</td>
                            <td><button class="text-xs text-accent-blue hover:underline">مراجعة ←</button></td>
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
                <h3 class="text-lg font-semibold text-text-primary">مراجعة الإيداع</h3>
                <button on:click={() => (selected = null)} class="text-text-tertiary hover:text-text-primary">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>
                </button>
            </div>

            <div class="space-y-3 text-sm">
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <div class="text-xs text-text-tertiary uppercase">المستخدم</div>
                        <div class="num-cell text-text-secondary text-xs">{selected.user_id}</div>
                    </div>
                    <div>
                        <div class="text-xs text-text-tertiary uppercase">المبلغ</div>
                        <div class="num-cell text-accent-green font-semibold">{fmtEgp(selected.amount)}</div>
                    </div>
                    <div>
                        <div class="text-xs text-text-tertiary uppercase">المرجع</div>
                        <div class="text-text-primary">{selected.reference || '—'}</div>
                    </div>
                    <div>
                        <div class="text-xs text-text-tertiary uppercase">تاريخ الإنشاء</div>
                        <div class="text-text-secondary text-xs">{fmtDate(selected.created_at)}</div>
                    </div>
                </div>

                <div>
                    <label class="label" for="status">الحالة الجديدة</label>
                    <select id="status" bind:value={reviewStatus} class="input">
                        <option value="under_review">قيد المراجعة</option>
                        <option value="approved">موافق (بانتار الاعتماد)</option>
                        <option value="completed">مكتمل (اعتمد الرصيد الآن)</option>
                        <option value="rejected">مرفوض</option>
                        <option value="failed">فشل</option>
                    </select>
                </div>

                <div>
                    <label class="label" for="note">ملاحظة المدير</label>
                    <textarea id="note" bind:value={reviewNote} class="input min-h-[80px]" placeholder="ملاحظة اختيارية..."></textarea>
                </div>

                {#if reviewError}<div class="text-xs text-accent-red">{reviewError}</div>{/if}
                {#if reviewSuccess}<div class="text-xs text-accent-green">{reviewSuccess}</div>{/if}

                <div class="flex gap-2 pt-2">
                    <button class="btn-ghost flex-1" on:click={() => (selected = null)}>إلغاء</button>
                    <button class="btn-primary flex-1" on:click={submitReview} disabled={reviewSubmitting}>
                        {reviewSubmitting ? 'جارٍ الإرسال...' : 'إرسال المراجعة'}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}
