<script lang="ts">
    import { onMount } from 'svelte';
    import { users } from '$lib/api';
    import { fmtDate, fmtRelative } from '$lib/format';
    import type { UserFull, UserStatus } from '$lib/types';

    let items: UserFull[] = [];
    let total = 0;
    let loading = true;
    let error = '';
    let filter: 'all' | UserStatus = 'all';
    let offset = 0;
    const limit = 50;

    let selected: UserFull | null = null;
    let newStatus: UserStatus = 'active';
    let newKyc: number | null = null;
    let saving = false;
    let saveError = '';
    let saveSuccess = '';

    // تعديل الرصيد
    let showWallet = false;
    let userWallets: any[] = [];
    let walletLoading = false;
    let adjustAsset = 'EGP';
    let adjustDelta = '';
    let adjustReason = '';
    let adjustSaving = false;
    let adjustError = '';
    let adjustSuccess = '';

    const API_BASE: string = (import.meta.env.VITE_API_URL as string) || 'http://localhost:8080';

    function authHeaders(): Record<string, string> {
        const token = typeof localStorage !== 'undefined' ? localStorage.getItem('admin_access_token') : null;
        return {
            'Content-Type': 'application/json',
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
        };
    }

    async function loadWallets(userId: string) {
        walletLoading = true;
        try {
            const res = await fetch(`${API_BASE}/api/admin/users/${userId}/wallets`, { headers: authHeaders() });
            userWallets = await res.json();
        } catch (e: any) {
            // ignore
        } finally {
            walletLoading = false;
        }
    }

    async function adjustWallet() {
        if (!selected || !adjustDelta || !adjustReason) {
            adjustError = 'املأ كل الحقول';
            return;
        }
        adjustSaving = true;
        adjustError = '';
        try {
            const res = await fetch(`${API_BASE}/api/admin/wallets/adjust`, {
                method: 'POST',
                headers: authHeaders(),
                body: JSON.stringify({
                    user_id: selected.id,
                    asset: adjustAsset,
                    delta: adjustDelta,
                    reason: adjustReason,
                }),
            });
            if (!res.ok) {
                const body = await res.json().catch(() => ({}));
                throw new Error(body?.error?.message || 'فشل التعديل');
            }
            adjustSuccess = 'تم تعديل الرصيد بنجاح';
            await loadWallets(selected.id);
            adjustDelta = '';
            adjustReason = '';
            setTimeout(() => (adjustSuccess = ''), 2000);
        } catch (e: any) {
            adjustError = e.message;
        } finally {
            adjustSaving = false;
        }
    }

    async function load() {
        try {
            const res = await users.list({
                offset,
                limit,
                status: filter === 'all' ? undefined : filter,
            });
            items = res.users || [];
            total = res.total;
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    $: pages = Math.max(1, Math.ceil(total / limit));

    function openEdit(u: UserFull) {
        selected = u;
        newStatus = u.status;
        newKyc = u.kyc_level;
        saveError = '';
        saveSuccess = '';
        showWallet = false;
    }

    function openWallets() {
        if (!selected) return;
        showWallet = true;
        loadWallets(selected.id);
    }

    async function save() {
        if (!selected) return;
        saving = true;
        saveError = '';
        try {
            await users.updateStatus(selected.id, newStatus, newKyc ?? undefined);
            saveSuccess = 'تم تحديث المستخدم';
            await load();
            setTimeout(() => { selected = null; saveSuccess = ''; }, 1200);
        } catch (e: any) {
            saveError = e.message;
        } finally {
            saving = false;
        }
    }
</script>

<svelte:head><title>المستخدمون · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">إدارة المستخدمين</h1>
        <p class="text-sm text-text-secondary mt-1">عرض وإدارة كل حسابات المستخدمين</p>
    </div>

    <div class="flex items-center gap-2 text-xs flex-wrap">
        {#each [['all', 'الكل'], ['active', 'نشط'], ['suspended', 'موقوف'], ['banned', 'محظور'], ['pending_kyc', 'بانتظار التحقق']] as [v, label]}
            <button
                class="px-3 py-1.5 rounded-md font-medium {filter === v ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:bg-base-700/50'}"
                on:click={() => { filter = v as any; offset = 0; load(); }}>
                {label}
            </button>
        {/each}
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>البريد</th>
                        <th>الاسم</th>
                        <th>الدور</th>
                        <th>الحالة</th>
                        <th>التحقق</th>
                        <th class="num-cell">منذ</th>
                        <th class="num-cell">آخر دخول</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as u}
                        <tr class="cursor-pointer" on:click={() => openEdit(u)}>
                            <td class="font-medium">{u.email}</td>
                            <td class="text-text-secondary">{u.full_name}</td>
                            <td>{#if u.role === 'admin'}<span class="pill-info">مدير</span>{:else}<span class="pill-muted">مستخدم</span>{/if}</td>
                            <td>
                                {#if u.status === 'active'}<span class="pill-success">نشط</span>
                                {:else if u.status === 'suspended'}<span class="pill-warning">موقوف</span>
                                {:else if u.status === 'banned'}<span class="pill-danger">محظور</span>
                                {:else}<span class="pill-muted">{u.status}</span>{/if}
                            </td>
                            <td class="num-cell">{u.kyc_level}</td>
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(u.created_at)}>{fmtRelative(u.created_at)}</td>
                            <td class="num-cell text-text-tertiary text-xs">{u.last_login_at ? fmtRelative(u.last_login_at) : '—'}</td>
                            <td><button class="text-xs text-accent-blue hover:underline">تعديل ←</button></td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>

        {#if pages > 1}
            <div class="flex items-center justify-between">
                <div class="text-xs text-text-tertiary">{total} إجمالي · صفحة {offset / limit + 1} من {pages}</div>
                <div class="flex gap-2">
                    <button class="btn-ghost text-xs" disabled={offset === 0} on:click={() => { offset -= limit; load(); }}>→ السابق</button>
                    <button class="btn-ghost text-xs" disabled={offset + limit >= total} on:click={() => { offset += limit; load(); }}>التالي ←</button>
                </div>
            </div>
        {/if}
    {/if}
</div>

{#if selected}
    <div class="fixed inset-0 z-50 bg-base-900/70 flex items-center justify-center p-4" on:click|self={() => (selected = null)}>
        <div class="bg-base-800 border border-base-600 rounded-lg w-full max-w-md p-6 max-h-[90vh] overflow-y-auto">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold text-text-primary">تعديل المستخدم</h3>
                <button on:click={() => (selected = null)} class="text-text-tertiary hover:text-text-primary">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>
                </button>
            </div>

            <div class="space-y-3 text-sm">
                <div class="text-text-secondary">{selected.email}</div>

                <div>
                    <label class="label" for="status">الحالة</label>
                    <select id="status" bind:value={newStatus} class="input">
                        <option value="active">نشط</option>
                        <option value="suspended">موقوف</option>
                        <option value="banned">محظور</option>
                        <option value="pending_kyc">بانتظار التحقق</option>
                    </select>
                </div>

                <div>
                    <label class="label" for="kyc">مستوى التحقق</label>
                    <input id="kyc" type="number" min="0" max="3" bind:value={newKyc} class="input" dir="ltr" />
                </div>

                {#if saveError}<div class="text-xs text-accent-red">{saveError}</div>{/if}
                {#if saveSuccess}<div class="text-xs text-accent-green">{saveSuccess}</div>{/if}

                <button class="btn-ghost w-full text-sm" on:click={openWallets}>
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M21 12a2.25 2.25 0 00-2.25-2.25H15a3 3 0 11-6 0H5.25A2.25 2.25 0 003 12m18 0v6a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 18v-6m18 0V9M3 12V9" /></svg>
                    عرض المحافظ وتعديل الأرصدة
                </button>

                <div class="flex gap-2 pt-2">
                    <button class="btn-ghost flex-1" on:click={() => (selected = null)}>إلغاء</button>
                    <button class="btn-primary flex-1" on:click={save} disabled={saving}>
                        {saving ? 'جارٍ الحفظ...' : 'حفظ'}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}

<!-- نافذة المحافظ وتعديل الأرصدة -->
{#if selected && showWallet}
    <div class="fixed inset-0 z-[60] bg-base-900/80 flex items-center justify-center p-4" on:click|self={() => (showWallet = false)}>
        <div class="bg-base-800 border border-base-600 rounded-lg w-full max-w-lg p-6 max-h-[90vh] overflow-y-auto">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold text-text-primary">محافظ المستخدم</h3>
                <button on:click={() => (showWallet = false)} class="text-text-tertiary hover:text-text-primary">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>
                </button>
            </div>

            <div class="text-xs text-text-secondary mb-3">{selected.email}</div>

            <!-- قائمة المحافظ -->
            {#if walletLoading}
                <div class="text-center py-4 text-text-tertiary text-sm">جارٍ التحميل...</div>
            {:else}
                <div class="card-compact mb-4">
                    <table class="table-pro">
                        <thead><tr><th>الأصل</th><th class="num-cell">المتاح</th><th class="num-cell">المحجوز</th></tr></thead>
                        <tbody>
                            {#each userWallets as w}
                                <tr>
                                    <td class="font-bold">{w.asset_symbol}</td>
                                    <td class="num-cell">{w.balance}</td>
                                    <td class="num-cell text-text-secondary">{w.locked_balance}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}

            <!-- تعديل رصيد -->
            <div class="border-t border-base-700 pt-4 space-y-3">
                <h4 class="text-sm font-semibold text-text-primary">تعديل رصيد</h4>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="label" for="adj-asset">الأصل</label>
                        <select id="adj-asset" bind:value={adjustAsset} class="input">
                            {#each userWallets as w}
                                <option value={w.asset_symbol}>{w.asset_symbol}</option>
                            {/each}
                        </select>
                    </div>
                    <div>
                        <label class="label" for="adj-delta">المبلغ (موجب=إضافة، سالب=خصم)</label>
                        <input id="adj-delta" type="number" step="any" bind:value={adjustDelta} class="input" placeholder="0.00" dir="ltr" />
                    </div>
                </div>
                <div>
                    <label class="label" for="adj-reason">السبب</label>
                    <input id="adj-reason" type="text" bind:value={adjustReason} class="input" placeholder="مثال: تعويض عن خطأ في النظام" />
                </div>
                {#if adjustError}<div class="text-xs text-accent-red">{adjustError}</div>{/if}
                {#if adjustSuccess}<div class="text-xs text-accent-green">{adjustSuccess}</div>{/if}
                <button class="btn-primary w-full text-sm" on:click={adjustWallet} disabled={adjustSaving}>
                    {adjustSaving ? 'جارٍ التعديل...' : 'تعديل الرصيد'}
                </button>
                <div class="text-xs text-text-tertiary text-center">
                    ⚠️ سيتم تسجيل العملية في سجل التدقيق
                </div>
            </div>
        </div>
    </div>
{/if}
