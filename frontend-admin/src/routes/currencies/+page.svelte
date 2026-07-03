<script lang="ts">
    import { onMount } from 'svelte';
    import { ApiError } from '$lib/api';
    import { fmtQty } from '$lib/format';
    import type { Currency, CurrencyType } from '$lib/types';

    let items: Currency[] = [];
    let loading = true;
    let error = '';
    let showForm = false;
    let editing: Currency | null = null;

    // نموذج
    let symbol = '';
    let name = '';
    let type: CurrencyType = 'crypto';
    let precision = 8;
    let withdrawFee = '0';
    let minWithdrawal = '0';
    let network = '';
    let isActive = true;
    let saving = false;

    const API_BASE: string = (import.meta.env.VITE_API_URL as string) || 'http://localhost:8080';

    function authHeaders(): Record<string, string> {
        const token = typeof localStorage !== 'undefined' ? localStorage.getItem('admin_access_token') : null;
        return {
            'Content-Type': 'application/json',
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
        };
    }

    async function apiCall<T>(path: string, init?: RequestInit): Promise<T> {
        const res = await fetch(`${API_BASE}${path}`, {
            ...init,
            headers: { ...authHeaders(), ...(init?.headers || {}) },
        });
        if (!res.ok) {
            const body = await res.json().catch(() => ({}));
            throw new ApiError(res.status, body?.error?.message || res.statusText);
        }
        return res.json();
    }

    async function load() {
        try {
            items = await apiCall<Currency[]>('/api/admin/currencies');
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    function resetForm() {
        symbol = '';
        name = '';
        type = 'crypto';
        precision = 8;
        withdrawFee = '0';
        minWithdrawal = '0';
        network = '';
        isActive = true;
        editing = null;
    }

    function openCreate() {
        resetForm();
        showForm = true;
    }

    function openEdit(c: Currency) {
        editing = c;
        symbol = c.symbol;
        name = c.name;
        type = c.type;
        precision = c.precision;
        withdrawFee = c.withdraw_fee;
        minWithdrawal = c.min_withdrawal;
        network = c.network || '';
        isActive = c.is_active;
        showForm = true;
    }

    async function save() {
        saving = true;
        error = '';
        try {
            if (editing) {
                await apiCall(`/api/admin/currencies/${editing.id}`, {
                    method: 'PUT',
                    body: JSON.stringify({
                        name,
                        precision,
                        withdraw_fee: withdrawFee,
                        min_withdrawal: minWithdrawal,
                        network: network || undefined,
                        is_active: isActive,
                    }),
                });
            } else {
                await apiCall('/api/admin/currencies', {
                    method: 'POST',
                    body: JSON.stringify({
                        symbol,
                        name,
                        type,
                        precision,
                        withdraw_fee: withdrawFee,
                        min_withdrawal: minWithdrawal,
                        network: network || undefined,
                        is_active: isActive,
                    }),
                });
            }
            showForm = false;
            resetForm();
            await load();
        } catch (e: any) {
            error = e.message;
        } finally {
            saving = false;
        }
    }

    async function toggleActive(c: Currency) {
        try {
            await apiCall(`/api/admin/currencies/${c.id}`, {
                method: 'PUT',
                body: JSON.stringify({ is_active: !c.is_active }),
            });
            await load();
        } catch (e: any) {
            alert(e.message);
        }
    }

    async function remove(c: Currency) {
        if (!confirm(`حذف العملة ${c.symbol}؟`)) return;
        try {
            await apiCall(`/api/admin/currencies/${c.id}`, { method: 'DELETE' });
            await load();
        } catch (e: any) {
            alert(e.message);
        }
    }
</script>

<svelte:head><title>إدارة العملات · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">إدارة العملات</h1>
            <p class="text-sm text-text-secondary mt-1">التحكم الكامل في العملات المدعومة ورسوم السحب</p>
        </div>
        <button class="btn-primary text-sm" on:click={openCreate}>+ إضافة عملة</button>
    </div>

    {#if error}<div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>{/if}

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>الرمز</th>
                        <th>الاسم</th>
                        <th>النوع</th>
                        <th class="num-cell">الدقة</th>
                        <th class="num-cell">رسوم السحب</th>
                        <th class="num-cell">أقل سحب</th>
                        <th>الشبكة</th>
                        <th>الحالة</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as c}
                        <tr>
                            <td class="font-bold">{c.symbol}</td>
                            <td>{c.name}</td>
                            <td><span class="pill-muted">{c.type === 'fiat' ? 'نقدي' : 'رقمي'}</span></td>
                            <td class="num-cell">{c.precision}</td>
                            <td class="num-cell">{fmtQty(c.withdraw_fee, 8)}</td>
                            <td class="num-cell">{fmtQty(c.min_withdrawal, 8)}</td>
                            <td class="text-text-secondary text-xs">{c.network || '—'}</td>
                            <td>
                                <button on:click={() => toggleActive(c)}>
                                    {#if c.is_active}
                                        <span class="pill-success">نشط</span>
                                    {:else}
                                        <span class="pill-danger">غير نشط</span>
                                    {/if}
                                </button>
                            </td>
                            <td>
                                <div class="flex gap-2">
                                    <button class="text-xs text-accent-blue hover:underline" on:click={() => openEdit(c)}>تعديل</button>
                                    <button class="text-xs text-accent-red hover:underline" on:click={() => remove(c)}>حذف</button>
                                </div>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

<!-- النموذج -->
{#if showForm}
    <div class="fixed inset-0 z-50 bg-base-900/70 flex items-center justify-center p-4" on:click|self={() => (showForm = false)}>
        <div class="bg-base-800 border border-base-600 rounded-lg w-full max-w-md p-6 max-h-[90vh] overflow-y-auto">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold text-text-primary">{editing ? 'تعديل عملة' : 'إضافة عملة'}</h3>
                <button on:click={() => (showForm = false)} class="text-text-tertiary hover:text-text-primary">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" /></svg>
                </button>
            </div>

            <div class="space-y-3 text-sm">
                <div>
                    <label class="label" for="sym">الرمز</label>
                    <input id="sym" type="text" bind:value={symbol} class="input" disabled={!!editing} placeholder="BTC" dir="ltr" />
                </div>
                <div>
                    <label class="label" for="name">الاسم</label>
                    <input id="name" type="text" bind:value={name} class="input" placeholder="بيتكوين" />
                </div>
                <div>
                    <label class="label" for="type">النوع</label>
                    <select id="type" bind:value={type} class="input" disabled={!!editing}>
                        <option value="fiat">نقدي</option>
                        <option value="crypto">رقمي</option>
                    </select>
                </div>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="label" for="prec">الدقة</label>
                        <input id="prec" type="number" min="0" max="18" bind:value={precision} class="input" />
                    </div>
                    <div>
                        <label class="label" for="net">الشبكة</label>
                        <input id="net" type="text" bind:value={network} class="input" placeholder="ERC-20" dir="ltr" />
                    </div>
                </div>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="label" for="wf">رسوم السحب</label>
                        <input id="wf" type="number" step="any" min="0" bind:value={withdrawFee} class="input" dir="ltr" />
                    </div>
                    <div>
                        <label class="label" for="mw">أقل سحب</label>
                        <input id="mw" type="number" step="any" min="0" bind:value={minWithdrawal} class="input" dir="ltr" />
                    </div>
                </div>
                <label class="flex items-center gap-2 mt-2">
                    <input type="checkbox" bind:checked={isActive} class="accent-accent-blue" />
                    <span class="text-text-secondary text-sm">نشط</span>
                </label>

                {#if error}<div class="text-xs text-accent-red">{error}</div>{/if}

                <div class="flex gap-2 pt-2">
                    <button class="btn-ghost flex-1" on:click={() => (showForm = false)}>إلغاء</button>
                    <button class="btn-primary flex-1" on:click={save} disabled={saving}>
                        {saving ? 'جارٍ الحفظ...' : 'حفظ'}
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}
