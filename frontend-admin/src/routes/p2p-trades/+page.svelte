<script lang="ts">
    import { onMount } from 'svelte';
    import { fmtEgp, fmtQty, fmtDate, fmtRelative } from '$lib/format';

    let trades: any[] = [];
    let loading = true;
    let error = '';
    let filter: 'all' | 'pending' | 'paid' | 'released' | 'cancelled' | 'disputed' | 'completed' = 'all';

    const API_BASE: string = (import.meta.env.VITE_API_URL as string) || 'http://localhost:8080';

    function authHeaders(): Record<string, string> {
        const token = typeof localStorage !== 'undefined' ? localStorage.getItem('admin_access_token') : null;
        return {
            'Content-Type': 'application/json',
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
        };
    }

    async function load() {
        try {
            const res = await fetch(`${API_BASE}/api/admin/p2p/trades`, { headers: authHeaders() });
            if (!res.ok) throw new Error('فشل تحميل الصفقات');
            trades = await res.json();
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    function statusLabel(s: string): string {
        const map: Record<string, string> = {
            pending: 'بانتظار الدفع',
            paid: 'تم الدفع',
            released: 'تم الإطلاق',
            cancelled: 'ملغاة',
            disputed: 'نزاع',
            completed: 'مكتملة',
        };
        return map[s] || s;
    }

    function statusPill(s: string): string {
        const map: Record<string, string> = {
            pending: 'pill-warning',
            paid: 'pill-info',
            released: 'pill-info',
            cancelled: 'pill-muted',
            disputed: 'pill-danger',
            completed: 'pill-success',
        };
        return map[s] || 'pill-muted';
    }

    function paymentLabel(id: string): string {
        const map: Record<string, string> = {
            bank_transfer: 'تحويل بنكي', vodafone_cash: 'فودافون كاش',
            instapay: 'إنستا باي', fawry: 'فوري',
            etisalat_cash: 'اتصالات كاش', orange_cash: 'أورانج كاش',
            we_pay: 'وي باي', cash_deposit: 'إيداع نقدي',
        };
        return map[id] || id;
    }

    $: filtered = filter === 'all' ? trades : trades.filter((t) => t.status === filter);
</script>

<svelte:head><title>صفقات P2P · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">صفقات التداول بين الأفراد</h1>
        <p class="text-sm text-text-secondary mt-1">مراقبة كل صفقات P2P في النظام</p>
    </div>

    <div class="flex items-center gap-2 text-xs flex-wrap">
        {#each ['all', 'pending', 'paid', 'released', 'cancelled', 'disputed', 'completed'] as f}
            <button
                class="px-3 py-1.5 rounded-md font-medium {filter === f ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:bg-base-700/50'}"
                on:click={() => (filter = f as any)}>
                {f === 'all' ? 'الكل' : statusLabel(f)}
            </button>
        {/each}
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if filtered.length === 0}
        <div class="card-default text-center py-12 text-text-tertiary text-sm">لا توجد صفقات</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>المعرف</th>
                        <th>المشتري</th>
                        <th>البائع</th>
                        <th>الأصل</th>
                        <th class="num-cell">الكمية</th>
                        <th class="num-cell">السعر</th>
                        <th class="num-cell">الإجمالي</th>
                        <th>طريقة الدفع</th>
                        <th>الحالة</th>
                        <th class="num-cell">منذ</th>
                    </tr>
                </thead>
                <tbody>
                    {#each filtered as t}
                        <tr>
                            <td class="num-cell text-text-tertiary text-xs">{t.id.slice(0, 8)}…</td>
                            <td class="num-cell text-text-tertiary text-xs">{t.buyer_id?.slice(0, 8)}…</td>
                            <td class="num-cell text-text-tertiary text-xs">{t.seller_id?.slice(0, 8)}…</td>
                            <td class="font-bold">{t.asset_symbol}</td>
                            <td class="num-cell">{fmtQty(t.amount, 8)}</td>
                            <td class="num-cell">{fmtEgp(t.price_egp)}</td>
                            <td class="num-cell">{fmtEgp(t.total_egp)}</td>
                            <td class="text-text-secondary text-xs">{paymentLabel(t.payment_method)}</td>
                            <td><span class={statusPill(t.status)}>{statusLabel(t.status)}</span></td>
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(t.created_at)}>{fmtRelative(t.created_at)}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
