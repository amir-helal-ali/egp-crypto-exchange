<script lang="ts">
    import { onMount } from 'svelte';
    import { wallet, txStatusPill } from '$lib/api';
    import { myWallets, myDeposits, myWithdrawals } from '$lib/stores';
    import { fmtEgp, fmtQty, fmtDate } from '$lib/format';

    let loading = true;
    let error = '';

    // نموذج الإيداع
    let depAmount = '';
    let depReference = '';
    let depSubmitting = false;
    let depError = '';
    let depSuccess = '';

    // نموذج السحب
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
            myWallets.set(ws);
            myDeposits.set(ds);
            myWithdrawals.set(wds);
        } catch (e: any) {
            if (e.status !== 401) error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(loadAll);

    async function submitDeposit() {
        depSubmitting = true;
        depError = '';
        depSuccess = '';
        try {
            const tx = await wallet.requestDeposit(depReference, depAmount);
            depSuccess = `تم إرسال طلب الإيداع (رقم: ${tx.id.slice(0, 8)}). ستظهر حالته في الطابور لحظياً هنا.`;
            depAmount = '';
            depReference = '';
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
            wdSuccess = `تم إرسال طلب السحب (رقم: ${tx.id.slice(0, 8)}). سيتم حجز الأموال حتى موافقة الإدارة.`;
            wdAmount = '';
            wdDestination = '';
        } catch (e: any) {
            wdError = e.message;
        } finally {
            wdSubmitting = false;
        }
    }

    $: allTx = [...$myDeposits, ...$myWithdrawals].sort((a, b) =>
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
    );
</script>

<svelte:head><title>المحفظة · منصة الجنيه</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">المحفظة</h1>
        <p class="text-sm text-text-secondary mt-1">إدارة أرصدتك وعمليات الإيداع والسحب</p>
    </div>

    {#if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {/if}

    <!-- الأرصدة -->
    <section>
        <h2 class="text-sm font-semibold text-text-secondary uppercase tracking-wider mb-3">الأرصدة</h2>
        <div class="card-default overflow-x-auto">
            {#if loading}
                <div class="text-center py-8 text-text-tertiary">جارٍ التحميل...</div>
            {:else if $myWallets.length === 0}
                <div class="text-center py-8 text-text-tertiary">لا توجد محافظ</div>
            {:else}
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الأصل</th>
                            <th>النوع</th>
                            <th class="num-cell">المتاح</th>
                            <th class="num-cell">المحجوز</th>
                            <th class="num-cell">الإجمالي</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each $myWallets as w}
                            <tr>
                                <td class="font-bold">{w.asset_symbol}</td>
                                <td><span class="pill-muted">{w.wallet_type === 'fiat' ? 'نقدي' : 'رقمي'}</span></td>
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

    <!-- التبويبات -->
    <section>
        <div class="border-b border-base-700 mb-4">
            <nav class="flex gap-4">
                <button class="py-2 text-sm font-medium border-b-2 {activeTab === 'deposit' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (activeTab = 'deposit')}>إيداع جنيه</button>
                <button class="py-2 text-sm font-medium border-b-2 {activeTab === 'withdraw' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (activeTab = 'withdraw')}>سحب عملة رقمية</button>
                <button class="py-2 text-sm font-medium border-b-2 {activeTab === 'history' ? 'border-accent-blue text-text-primary' : 'border-transparent text-text-secondary hover:text-text-primary'}" on:click={() => (activeTab = 'history')}>سجل المعاملات</button>
            </nav>
        </div>

        {#if activeTab === 'deposit'}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="card-default">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">إيداع جنيه جديد</h3>
                    <p class="text-xs text-text-secondary mb-4 leading-relaxed">
                        حوّل الجنيه إلى حسابنا البنكي، ثم أرسل طلب إيداع مع مرجع التحويل. سيتحقق المدير ويُعتمد رصيدك. هذه عملية يدوية - تظهر الطلبات في طابور الإدارة وتتحدث لحظياً بالأسفل.
                    </p>
                    <div class="space-y-3">
                        <div>
                            <label class="label" for="dep-amount">المبلغ (جنيه)</label>
                            <input id="dep-amount" type="number" step="0.01" min="100" bind:value={depAmount} class="input" placeholder="100.00" />
                        </div>
                        <div>
                            <label class="label" for="dep-ref">مرجع التحويل / رقم الإيصال</label>
                            <input id="dep-ref" type="text" bind:value={depReference} class="input" placeholder="TRX20240115001" dir="ltr" />
                        </div>
                        {#if depError}<div class="text-xs text-accent-red">{depError}</div>{/if}
                        {#if depSuccess}<div class="text-xs text-accent-green">{depSuccess}</div>{/if}
                        <button class="w-full btn-success" on:click={submitDeposit} disabled={depSubmitting}>
                            {depSubmitting ? 'جارٍ الإرسال...' : 'إرسال طلب الإيداع'}
                        </button>
                    </div>
                </div>

                <div class="card-default">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">تعليمات التحويل البنكي</h3>
                    <div class="text-xs space-y-2 text-text-secondary">
                        <div class="flex justify-between"><span class="text-text-tertiary">البنك:</span> <span>بنك مصر</span></div>
                        <div class="flex justify-between"><span class="text-text-tertiary">اسم الحساب:</span> <span>منصة الجنيه للعملات الرقمية</span></div>
                        <div class="flex justify-between"><span class="text-text-tertiary">رقم الحساب:</span> <span class="num-cell">1000-2000-3000-4000</span></div>
                        <div class="flex justify-between"><span class="text-text-tertiary">الآيبان:</span> <span class="num-cell">EG00 1000 2000 3000 4000 5000</span></div>
                        <div class="flex justify-between"><span class="text-text-tertiary">السويفت:</span> <span class="num-cell">BMISEGCX</span></div>
                        <div class="flex justify-between"><span class="text-text-tertiary">المرجع:</span> <span>استخدم بريدك أو رقم الإيداع</span></div>
                    </div>
                    <div class="mt-4 text-xs text-accent-yellow bg-accent-yellow/10 border border-accent-yellow/30 rounded px-3 py-2">
                        <strong>هام:</strong> استخدم دائماً مرجعاً للتحويل، وإلا لن نتمكن من مطابقتها.
                    </div>
                </div>
            </div>
        {:else if activeTab === 'withdraw'}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="card-default">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">سحب عملة رقمية جديد</h3>
                    <p class="text-xs text-text-secondary mb-4 leading-relaxed">
                        تتم معالجة سحب العملات الرقمية يدوياً بواسطة فريق الإدارة بعد المراجعة الأمنية. سيتم حجز أموالك فور الإرسال وتُطلق في حال الرفض.
                    </p>
                    <div class="space-y-3">
                        <div>
                            <label class="label" for="wd-asset">العملة</label>
                            <select id="wd-asset" bind:value={wdAsset} class="input">
                                <option value="BTC">BTC</option>
                                <option value="ETH">ETH</option>
                                <option value="USDT">USDT</option>
                            </select>
                        </div>
                        <div>
                            <label class="label" for="wd-amount">المبلغ</label>
                            <input id="wd-amount" type="number" step="any" min="0" bind:value={wdAmount} class="input" placeholder="0.00" />
                        </div>
                        <div>
                            <label class="label" for="wd-dest">عنوان الوجهة</label>
                            <input id="wd-dest" type="text" bind:value={wdDestination} class="input" placeholder="bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh" dir="ltr" />
                        </div>
                        {#if wdError}<div class="text-xs text-accent-red">{wdError}</div>{/if}
                        {#if wdSuccess}<div class="text-xs text-accent-green">{wdSuccess}</div>{/if}
                        <button class="w-full btn-danger" on:click={submitWithdrawal} disabled={wdSubmitting}>
                            {wdSubmitting ? 'جارٍ الإرسال...' : 'إرسال طلب السحب'}
                        </button>
                    </div>
                </div>

                <div class="card-default">
                    <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">ملاحظات أمنية</h3>
                    <ul class="text-xs text-text-secondary space-y-2 list-disc list-inside leading-relaxed">
                        <li>تتم مراجعة السحوبات خلال 24 ساعة في أيام العمل.</li>
                        <li>تحقق دائماً من عنوان الوجهة - معاملات البلوكتشين لا رجعة فيها.</li>
                        <li>السحوبات الكبيرة قد تتطلب مستوى تحقق 2.</li>
                        <li>يتم خصم رسوم الشبكة من المبلغ المسحوب.</li>
                        <li>قد يتم وضع علامة على النشاط المشبوه للتحقق الإضافي.</li>
                    </ul>
                </div>
            </div>
        {:else}
            <div class="card-default overflow-x-auto">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">سجل المعاملات</h3>
                {#if allTx.length === 0}
                    <div class="text-center py-8 text-text-tertiary text-sm">لا توجد معاملات بعد</div>
                {:else}
                    <table class="table-pro">
                        <thead>
                            <tr>
                                <th>النوع</th>
                                <th>الأصل</th>
                                <th class="num-cell">المبلغ</th>
                                <th class="num-cell">الرسوم</th>
                                <th>الحالة</th>
                                <th class="num-cell">المرجع / الوجهة</th>
                                <th class="num-cell">تاريخ الإنشاء</th>
                                <th class="num-cell">تاريخ الإكمال</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each allTx as tx}
                                <tr>
                                    <td><span class={tx.tx_type === 'deposit' ? 'pill-success' : 'pill-info'}>{tx.tx_type === 'deposit' ? 'إيداع' : 'سحب'}</span></td>
                                    <td class="font-bold">{tx.asset_symbol}</td>
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
