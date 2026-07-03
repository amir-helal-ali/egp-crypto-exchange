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
        <div class="bg-base-800 border border-base-600 rounded-lg w-full max-w-md p-6">
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
