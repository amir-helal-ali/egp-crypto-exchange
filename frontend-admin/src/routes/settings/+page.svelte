<script lang="ts">
    import { onMount } from 'svelte';
    import { ApiError } from '$lib/api';
    import { fmtNum } from '$lib/format';

    let egpUsdRate = '';
    let minEgpDeposit = '';
    let minEgpWithdrawal = '';
    let loading = true;
    let saving = false;
    let error = '';
    let success = '';

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
            const res = await fetch(`${API_BASE}/api/admin/settings`, { headers: authHeaders() });
            const data = await res.json();
            // القيم تأتي كـ JSONB - قد تكون strings أو numbers
            egpUsdRate = String(data.egp_usd_rate || '48.5').replace(/"/g, '');
            minEgpDeposit = String(data.min_egp_deposit || '100').replace(/"/g, '');
            minEgpWithdrawal = String(data.min_egp_withdrawal || '200').replace(/"/g, '');
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    async function save() {
        saving = true;
        error = '';
        success = '';
        try {
            const res = await fetch(`${API_BASE}/api/admin/settings`, {
                method: 'PUT',
                headers: authHeaders(),
                body: JSON.stringify({
                    egp_usd_rate: egpUsdRate,
                    min_egp_deposit: minEgpDeposit,
                    min_egp_withdrawal: minEgpWithdrawal,
                }),
            });
            if (!res.ok) {
                const body = await res.json().catch(() => ({}));
                throw new Error(body?.error?.message || 'فشل الحفظ');
            }
            success = 'تم حفظ الإعدادات بنجاح';
            setTimeout(() => (success = ''), 3000);
        } catch (e: any) {
            error = e.message;
        } finally {
            saving = false;
        }
    }
</script>

<svelte:head><title>الإعدادات · الإدارة</title></svelte:head>

<div class="space-y-6 max-w-2xl">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">الإعدادات</h1>
        <p class="text-sm text-text-secondary mt-1">التحكم في إعدادات النظام الأساسية</p>
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else}
        <div class="card-default space-y-5">
            <div>
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">إعدادات سعر الصرف</h3>
                <div>
                    <label class="label" for="rate">سعر الجنيه مقابل الدولار (EGP/USD)</label>
                    <input id="rate" type="number" step="0.01" min="0" bind:value={egpUsdRate} class="input" dir="ltr" />
                    <p class="text-xs text-text-tertiary mt-1">يُستخدم لاشتقاق أسعار العملات الرقمية مقابل الجنيه من أسعار Binance USDT</p>
                </div>
            </div>

            <div class="border-t border-base-700 pt-5">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">حدود الإيداع والسحب</h3>
                <div class="grid grid-cols-2 gap-3">
                    <div>
                        <label class="label" for="md">أقل إيداع (جنيه)</label>
                        <input id="md" type="number" step="0.01" min="0" bind:value={minEgpDeposit} class="input" dir="ltr" />
                    </div>
                    <div>
                        <label class="label" for="mw">أقل سحب (جنيه)</label>
                        <input id="mw" type="number" step="0.01" min="0" bind:value={minEgpWithdrawal} class="input" dir="ltr" />
                    </div>
                </div>
            </div>

            {#if error}
                <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
            {/if}
            {#if success}
                <div class="text-xs text-accent-green bg-accent-green/10 border border-accent-green/30 rounded px-3 py-2">{success}</div>
            {/if}

            <button class="btn-primary text-sm" on:click={save} disabled={saving}>
                {saving ? 'جارٍ الحفظ...' : 'حفظ الإعدادات'}
            </button>
        </div>

        <!-- معلومات إضافية -->
        <div class="card-default">
            <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">معلومات النظام</h3>
            <div class="space-y-2 text-xs text-text-secondary">
                <div class="flex justify-between">
                    <span>مصدر أسعار العملات الرقمية:</span>
                    <span class="text-text-primary">Binance WebSocket</span>
                </div>
                <div class="flex justify-between">
                    <span>آلية تحديد السعر:</span>
                    <span class="text-text-primary num-cell">binance_price × EGP/USD</span>
                </div>
                <div class="flex justify-between">
                    <span>نظام قاطع الدائرة:</span>
                    <span class="text-text-primary">يفتح عند انقطاع التدفق</span>
                </div>
                <div class="flex justify-between">
                    <span>نظام الضمان (P2P):</span>
                    <span class="text-text-primary">حجز تلقائي حتى تأكيد الدفع</span>
                </div>
            </div>
        </div>
    {/if}
</div>
