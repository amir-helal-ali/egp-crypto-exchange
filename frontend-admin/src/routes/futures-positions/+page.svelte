<script lang="ts">
    import { onMount } from 'svelte';
    import { ApiError } from '$lib/api';
    import { fmtEgp, fmtQty, fmtDate, fmtRelative } from '$lib/format';

    let positions: any[] = [];
    let loading = true;
    let error = '';

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
            const res = await fetch(`${API_BASE}/api/admin/futures/positions`, { headers: authHeaders() });
            const data = await res.json();
            positions = data.positions || [];
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    async function forceClose(id: string) {
        if (!confirm('إجبار إغلاق هذا المركز؟')) return;
        try {
            await fetch(`${API_BASE}/api/futures/positions/${id}/close`, {
                method: 'POST',
                headers: authHeaders(),
            });
            await load();
        } catch (e: any) {
            alert(e.message);
        }
    }

    $: totalMargin = positions.reduce((sum, p) => sum + Number(p.margin), 0);
    $: totalPnl = positions.reduce((sum, p) => sum + Number(p.unrealized_pnl), 0);
    $: longCount = positions.filter((p) => p.side === 'long').length;
    $: shortCount = positions.filter((p) => p.side === 'short').length;
</script>

<svelte:head><title>مراكز العقود الآجلة · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">مراكز العقود الآجلة</h1>
        <p class="text-sm text-text-secondary mt-1">مراقبة كل المراكز المفتوحة في النظام</p>
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else}
        <!-- ملخص -->
        <section class="grid grid-cols-2 md:grid-cols-4 gap-3">
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">إجمالي المراكز</div>
                <div class="text-2xl font-mono font-bold text-text-primary mt-1">{positions.length}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">مراكز شراء</div>
                <div class="text-2xl font-mono font-bold text-accent-green mt-1">{longCount}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">مراكز بيع</div>
                <div class="text-2xl font-mono font-bold text-accent-red mt-1">{shortCount}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">إجمالي الهامش</div>
                <div class="text-2xl font-mono font-bold text-accent-blue mt-1">{fmtEgp(totalMargin.toFixed(2))}</div>
            </div>
        </section>

        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>المستخدم</th>
                        <th>الزوج</th>
                        <th>الجهة</th>
                        <th>الرافعة</th>
                        <th>نوع الهامش</th>
                        <th class="num-cell">الهامش</th>
                        <th class="num-cell">الحجم</th>
                        <th class="num-cell">سعر الدخول</th>
                        <th class="num-cell">سعر السوق</th>
                        <th class="num-cell">سعر التصفية</th>
                        <th class="num-cell">PnL غير محقق</th>
                        <th>منذ</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {#each positions as p}
                        {@const pnl = Number(p.unrealized_pnl)}
                        <tr>
                            <td class="num-cell text-text-tertiary text-xs">{p.user_id.slice(0, 8)}…</td>
                            <td class="font-bold">{p.pair.replace('_', '/')}</td>
                            <td><span class={p.side === 'long' ? 'text-accent-green' : 'text-accent-red'}>{p.side === 'long' ? 'شراء' : 'بيع'}</span></td>
                            <td class="num-cell">{p.leverage}x</td>
                            <td class="text-text-secondary text-xs">{p.margin_mode === 'isolated' ? 'معزول' : 'متقاطع'}</td>
                            <td class="num-cell">{fmtEgp(p.margin)}</td>
                            <td class="num-cell">{fmtQty(p.quantity, 8)}</td>
                            <td class="num-cell">{fmtEgp(p.entry_price)}</td>
                            <td class="num-cell">{fmtEgp(p.mark_price)}</td>
                            <td class="num-cell text-accent-yellow">{fmtEgp(p.liquidation_price)}</td>
                            <td class="num-cell {pnl >= 0 ? 'text-accent-green' : 'text-accent-red'} font-semibold">{pnl >= 0 ? '+' : ''}{fmtEgp(pnl.toFixed(2))}</td>
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(p.created_at)}>{fmtRelative(p.created_at)}</td>
                            <td><button class="text-accent-red text-xs hover:underline" on:click={() => forceClose(p.id)}>إغلاق إجباري</button></td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
