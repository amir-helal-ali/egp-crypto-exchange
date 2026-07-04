<script lang="ts">
    import { onMount } from 'svelte';
    import {
        myOrders, myTrades, myDeposits, myWithdrawals,
        myPositions, myP2PTrades, myWallets, connectMarketWs, pushNotification
    } from '$lib/stores';
    import { fmtEgp, fmtQty, fmtPrice, fmtDate, fmtRelative } from '$lib/format';

    let tab: 'overview' | 'orders' | 'trades' | 'deposits' | 'withdrawals' | 'positions' | 'p2p' = 'overview';
    let loading = true;

    onMount(async () => {
        try {
            await connectMarketWs();
        } finally {
            loading = false;
        }
    });

    $: totalValue = $myWallets.reduce((s, w) => {
        if (w.asset_symbol === 'EGP') return s + Number(w.balance);
        return s; // for crypto, would need ticker price — computed in dashboard
    }, 0);
    $: openOrdersCount = $myOrders.filter((o) => o.status === 'open' || o.status === 'partially_filled').length;
    $: openPositionsCount = $myPositions.filter((p) => p.status === 'open').length;
    $: pendingTxCount = [...$myDeposits, ...$myWithdrawals].filter((t) => t.status === 'pending' || t.status === 'under_review').length;
    $: activeP2PCount = $myP2PTrades.filter((t) => t.status === 'pending' || t.status === 'paid').length;

    function txStatusPill(s: string): string {
        const map: Record<string, string> = {
            pending: 'pill-warning', under_review: 'pill-info', approved: 'pill-info',
            completed: 'pill-success', rejected: 'pill-danger', failed: 'pill-danger',
        };
        return map[s] || 'pill-muted';
    }
    function txStatusLabel(s: string): string {
        const map: Record<string, string> = {
            pending: 'معلق', under_review: 'قيد المراجعة', approved: 'موافق',
            completed: 'مكتمل', rejected: 'مرفوض', failed: 'فشل',
        };
        return map[s] || s;
    }
    function p2pStatusPill(s: string): string {
        const map: Record<string, string> = {
            pending: 'pill-warning', paid: 'pill-info', released: 'pill-info',
            cancelled: 'pill-muted', disputed: 'pill-danger', completed: 'pill-success',
        };
        return map[s] || 'pill-muted';
    }
    function p2pStatusLabel(s: string): string {
        const map: Record<string, string> = {
            pending: 'بانتظار الدفع', paid: 'تم الدفع', released: 'تم الإطلاق',
            cancelled: 'ملغاة', disputed: 'نزاع', completed: 'مكتملة',
        };
        return map[s] || s;
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
</script>

<svelte:head><title>نشاطي · منصة الجنيه</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">نشاطي</h1>
        <p class="text-sm text-text-secondary mt-1">كل عملياتك في مكان واحد - محدثة لحظياً</p>
    </div>

    <!-- بطاقات الملخص -->
    <section class="grid grid-cols-2 md:grid-cols-4 gap-3">
        <a href="/wallet" class="card-compact hover:border-accent-blue transition-colors block">
            <div class="text-xs text-text-tertiary uppercase">رصيد الجنيه</div>
            <div class="text-xl font-mono font-bold text-accent-green mt-1">{fmtEgp(totalValue.toFixed(2))}</div>
        </a>
        <button class="card-compact hover:border-accent-blue transition-colors text-right" on:click={() => tab = 'orders'}>
            <div class="text-xs text-text-tertiary uppercase">أوامر مفتوحة</div>
            <div class="text-xl font-mono font-bold text-accent-blue mt-1">{openOrdersCount}</div>
        </button>
        <button class="card-compact hover:border-accent-purple transition-colors text-right" on:click={() => tab = 'positions'}>
            <div class="text-xs text-text-tertiary uppercase">مراكز مفتوحة</div>
            <div class="text-xl font-mono font-bold text-accent-purple mt-1">{openPositionsCount}</div>
        </button>
        <button class="card-compact hover:border-accent-yellow transition-colors text-right" on:click={() => tab = 'deposits'}>
            <div class="text-xs text-text-tertiary uppercase">عمليات معلقة</div>
            <div class="text-xl font-mono font-bold text-accent-yellow mt-1">{pendingTxCount}</div>
        </button>
    </section>

    <!-- التبويبات -->
    <div class="border-b border-base-700 overflow-x-auto">
        <nav class="flex gap-1 min-w-max">
            {#each [
                ['overview', 'نظرة عامة'],
                ['orders', 'الأوامر'],
                ['trades', 'الصفقات'],
                ['deposits', 'الإيداعات'],
                ['withdrawals', 'السحوبات'],
                ['positions', 'المراكز'],
                ['p2p', 'صفقات P2P'],
            ] as [key, label]}
                <button
                    class="py-2 px-3 text-sm font-medium border-b-2 whitespace-nowrap {tab === key ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}"
                    on:click={() => (tab = key as any)}>
                    {label}
                </button>
            {/each}
        </nav>
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if tab === 'overview'}
        <!-- نظرة عامة -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">آخر الأوامر</h3>
                {#if $myOrders.length === 0}
                    <div class="text-center py-4 text-text-tertiary text-sm">لا توجد أوامر</div>
                {:else}
                    <div class="space-y-2">
                        {#each $myOrders.slice(0, 5) as o}
                            <div class="flex items-center justify-between text-sm py-1">
                                <div>
                                    <span class={o.side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{o.side === 'buy' ? 'شراء' : 'بيع'}</span>
                                    <span class="text-text-secondary mr-2">{o.pair.replace('_', '/')}</span>
                                </div>
                                <div class="text-xs text-text-tertiary">{fmtRelative(o.created_at)}</div>
                            </div>
                        {/each}
                    </div>
                    <a href="/history" class="block text-xs text-accent-blue hover:underline mt-2 text-center">عرض الكل ←</a>
                {/if}
            </div>

            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">آخر الصفقات</h3>
                {#if $myTrades.length === 0}
                    <div class="text-center py-4 text-text-tertiary text-sm">لا توجد صفقات</div>
                {:else}
                    <div class="space-y-2">
                        {#each $myTrades.slice(0, 5) as t}
                            <div class="flex items-center justify-between text-sm py-1">
                                <div>
                                    <span class={t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{t.taker_side === 'buy' ? 'شراء' : 'بيع'}</span>
                                    <span class="num-cell mr-2">{fmtQty(t.quantity, 6)}</span>
                                    <span class="text-text-tertiary text-xs">{t.pair.split('_')[0]}</span>
                                </div>
                                <div class="num-cell text-xs text-text-tertiary">{fmtPrice(t.price)}</div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>

            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">آخر الإيداعات والسحوبات</h3>
                {#if [...$myDeposits, ...$myWithdrawals].length === 0}
                    <div class="text-center py-4 text-text-tertiary text-sm">لا توجد معاملات</div>
                {:else}
                    <div class="space-y-2">
                        {#each [...$myDeposits, ...$myWithdrawals].sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()).slice(0, 5) as tx}
                            <div class="flex items-center justify-between text-sm py-1">
                                <div class="flex items-center gap-2">
                                    <span class={tx.tx_type === 'deposit' ? 'pill-success' : 'pill-info'}>{tx.tx_type === 'deposit' ? 'إيداع' : 'سحب'}</span>
                                    <span class="text-text-secondary">{tx.asset_symbol}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class={txStatusPill(tx.status)}>{txStatusLabel(tx.status)}</span>
                                    <span class="text-xs text-text-tertiary">{fmtRelative(tx.created_at)}</span>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>

            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">صفقات P2P النشطة</h3>
                {#if $myP2PTrades.filter((t) => t.status === 'pending' || t.status === 'paid').length === 0}
                    <div class="text-center py-4 text-text-tertiary text-sm">لا توجد صفقات نشطة</div>
                {:else}
                    <div class="space-y-2">
                        {#each $myP2PTrades.filter((t) => t.status === 'pending' || t.status === 'paid').slice(0, 5) as t}
                            <a href="/p2p/trade/{t.id}" class="flex items-center justify-between text-sm py-1 hover:bg-base-700/30 rounded px-2 -mx-2">
                                <div>
                                    <span class="num-cell">{fmtQty(t.amount, 6)}</span>
                                    <span class="text-text-secondary mr-2">{t.asset_symbol}</span>
                                </div>
                                <span class={p2pStatusPill(t.status)}>{p2pStatusLabel(t.status)}</span>
                            </a>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    {:else if tab === 'orders'}
        <div class="card-default overflow-x-auto">
            {#if $myOrders.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">لا توجد أوامر</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الزوج</th><th>الجهة</th><th>النوع</th>
                            <th class="num-cell">السعر</th>
                            <th class="num-cell">الكمية</th>
                            <th class="num-cell">المنفذ</th>
                            <th>الحالة</th>
                            <th class="num-cell">الوقت</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myOrders as o}
                            <tr>
                                <td class="font-bold">{o.pair.replace('_', '/')}</td>
                                <td><span class={o.side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{o.side === 'buy' ? 'شراء' : 'بيع'}</span></td>
                                <td class="text-text-secondary">{o.order_type === 'limit' ? 'محدد' : 'سوقي'}</td>
                                <td class="num-cell">{o.price ? fmtPrice(o.price) : '—'}</td>
                                <td class="num-cell">{fmtQty(o.quantity)}</td>
                                <td class="num-cell text-text-secondary">{fmtQty(o.filled_quantity)}</td>
                                <td>
                                    {#if o.status === 'open'}<span class="pill-warning">مفتوح</span>
                                    {:else if o.status === 'partially_filled'}<span class="pill-info">منفذ جزئياً</span>
                                    {:else if o.status === 'filled'}<span class="pill-success">منفذ بالكامل</span>
                                    {:else if o.status === 'cancelled'}<span class="pill-muted">ملغى</span>
                                    {:else}<span class="pill-danger">{o.status}</span>{/if}
                                </td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(o.created_at)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {:else if tab === 'trades'}
        <div class="card-default overflow-x-auto">
            {#if $myTrades.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">لا توجد صفقات</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الزوج</th><th>الجهة</th>
                            <th class="num-cell">السعر</th>
                            <th class="num-cell">الكمية</th>
                            <th class="num-cell">القيمة</th>
                            <th class="num-cell">الرسوم</th>
                            <th class="num-cell">الوقت</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myTrades as t}
                            <tr>
                                <td class="font-bold">{t.pair.replace('_', '/')}</td>
                                <td><span class={t.taker_side === 'buy' ? 'text-accent-green' : 'text-accent-red'}>{t.taker_side === 'buy' ? 'شراء' : 'بيع'}</span></td>
                                <td class="num-cell">{fmtPrice(t.price)}</td>
                                <td class="num-cell">{fmtQty(t.quantity)}</td>
                                <td class="num-cell">{fmtEgp((Number(t.price) * Number(t.quantity)).toString())}</td>
                                <td class="num-cell text-text-secondary">{fmtQty(t.taker_fee)}</td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(t.executed_at)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {:else if tab === 'deposits'}
        <div class="card-default overflow-x-auto">
            {#if $myDeposits.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">لا توجد إيداعات</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>المبلغ</th><th>المرجع</th><th>الحالة</th>
                            <th class="num-cell">تاريخ الإنشاء</th>
                            <th class="num-cell">تاريخ الإكمال</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myDeposits as tx}
                            <tr>
                                <td class="num-cell font-bold">{fmtEgp(tx.amount)}</td>
                                <td class="text-text-secondary text-sm">{tx.reference || '—'}</td>
                                <td><span class={txStatusPill(tx.status)}>{txStatusLabel(tx.status)}</span></td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(tx.created_at)}</td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(tx.completed_at)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {:else if tab === 'withdrawals'}
        <div class="card-default overflow-x-auto">
            {#if $myWithdrawals.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">لا توجد سحوبات</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الأصل</th><th class="num-cell">المبلغ</th>
                            <th>الوجهة</th><th>الحالة</th>
                            <th class="num-cell">تاريخ الإنشاء</th>
                            <th class="num-cell">تاريخ الإكمال</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myWithdrawals as tx}
                            <tr>
                                <td class="font-bold">{tx.asset_symbol}</td>
                                <td class="num-cell">{fmtQty(tx.amount, 8)}</td>
                                <td class="text-text-secondary text-xs num-cell max-w-[160px] truncate" title={tx.destination || ''}>{tx.destination || '—'}</td>
                                <td><span class={txStatusPill(tx.status)}>{txStatusLabel(tx.status)}</span></td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(tx.created_at)}</td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(tx.completed_at)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {:else if tab === 'positions'}
        <div class="card-default overflow-x-auto">
            {#if $myPositions.filter((p) => p.status === 'open').length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">لا توجد مراكز مفتوحة</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الزوج</th><th>الجهة</th><th>الرافعة</th>
                            <th class="num-cell">الهامش</th>
                            <th class="num-cell">الحجم</th>
                            <th class="num-cell">سعر الدخول</th>
                            <th class="num-cell">سعر السوق</th>
                            <th class="num-cell">PnL</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myPositions.filter((p) => p.status === 'open') as p}
                            {@const pnl = Number(p.unrealized_pnl)}
                            <tr>
                                <td class="font-bold">{p.pair.replace('_', '/')}</td>
                                <td><span class={p.side === 'long' ? 'text-accent-green' : 'text-accent-red'}>{p.side === 'long' ? 'شراء' : 'بيع'}</span></td>
                                <td class="num-cell">{p.leverage}x</td>
                                <td class="num-cell">{fmtEgp(p.margin)}</td>
                                <td class="num-cell">{fmtQty(p.quantity, 8)}</td>
                                <td class="num-cell">{fmtPrice(p.entry_price)}</td>
                                <td class="num-cell">{fmtPrice(p.mark_price)}</td>
                                <td class="num-cell {pnl >= 0 ? 'text-accent-green' : 'text-accent-red'} font-semibold">
                                    {pnl >= 0 ? '+' : ''}{fmtEgp(pnl.toFixed(2))}
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {:else if tab === 'p2p'}
        <div class="card-default overflow-x-auto">
            {#if $myP2PTrades.length === 0}
                <div class="text-center py-8 text-text-tertiary text-sm">
                    لا توجد صفقات P2P
                    <a href="/p2p" class="block text-accent-blue hover:underline mt-2">تصفح العروض ←</a>
                </div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الأصل</th>
                            <th class="num-cell">الكمية</th>
                            <th class="num-cell">الإجمالي</th>
                            <th>طريقة الدفع</th>
                            <th>الحالة</th>
                            <th class="num-cell">الوقت</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myP2PTrades as t}
                            <tr>
                                <td class="font-bold">{t.asset_symbol}</td>
                                <td class="num-cell">{fmtQty(t.amount, 8)}</td>
                                <td class="num-cell">{fmtEgp(t.total_egp)}</td>
                                <td class="text-text-secondary text-xs">{paymentLabel(t.payment_method)}</td>
                                <td><span class={p2pStatusPill(t.status)}>{p2pStatusLabel(t.status)}</span></td>
                                <td class="num-cell text-text-tertiary text-xs">{fmtDate(t.created_at)}</td>
                                <td>
                                    {#if t.status === 'pending' || t.status === 'paid'}
                                        <a href="/p2p/trade/{t.id}" class="text-xs text-accent-blue hover:underline">فتح ←</a>
                                    {/if}
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            {/if}
        </div>
    {/if}
</div>
