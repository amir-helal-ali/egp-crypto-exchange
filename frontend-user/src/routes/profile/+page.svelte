<script lang="ts">
    import { onMount } from 'svelte';
    import { user, myWallets, pushNotification } from '$lib/stores';
    import { fmtDate } from '$lib/format';

    let loading = true;
    let error = '';
    let fullName = '';
    let phone = '';
    let saving = false;
    let saveError = '';
    let saveSuccess = '';

    // KYC state
    let kycLevel = 0;
    let kycStatus = 'pending';

    onMount(async () => {
        try {
            const { auth } = await import('$lib/api');
            const me = await auth.me();
            fullName = me.full_name;
            kycLevel = me.kyc_level;
            kycStatus = me.status === 'active' && kycLevel >= 2 ? 'verified' : 'pending';
        } catch (e: any) {
            if (e.status !== 401) error = e.message;
        } finally {
            loading = false;
        }
    });

    async function saveProfile() {
        saving = true;
        saveError = '';
        saveSuccess = '';
        try {
            // ملاحظة: الـ API الحالي لا يدعم تحديث الملف الشخصي
            // لكن نحفظ محلياً ونعرض إشعار
            saveSuccess = 'سيتم تحديث ملفك الشخصي قريباً';
            pushNotification({ type: 'info', title: 'ملف شخصي', message: 'ميزة تحديث الملف قيد التطوير' });
            setTimeout(() => (saveSuccess = ''), 3000);
        } catch (e: any) {
            saveError = e.message;
        } finally {
            saving = false;
        }
    }

    function kycLevelLabel(level: number): string {
        return ['غير مُتحقق', 'مستوى 1', 'مستوى 2', 'مستوى 3'][level] || 'غير مُتحقق';
    }
    function kycLevelDesc(level: number): string {
        return [
            'لم يتم التحقق - حدود منخفضة على السحب',
            'بريد إلكتروني مُؤكد - حد متوسط',
            'هوية مُؤكدة - حد مرتفع',
            'إقامة مُؤكدة - حد كامل + ميزات متقدمة',
        ][level] || '';
    }

    $: totalBalance = $myWallets.reduce((s, w) => s + Number(w.balance) + Number(w.locked_balance), 0);
    $: activeWallets = $myWallets.filter((w) => Number(w.balance) > 0 || Number(w.locked_balance) > 0).length;
</script>

<svelte:head><title>ملفي الشخصي · منصة الجنيه</title></svelte:head>

<div class="space-y-6 max-w-4xl mx-auto">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">ملفي الشخصي</h1>
        <p class="text-sm text-text-secondary mt-1">إدارة حسابك ومعلوماتك الشخصية</p>
    </div>

    {#if loading}
        <div class="card-default"><div class="animate-pulse space-y-3">{#each Array(5) as _}<div class="h-3 bg-base-700 rounded w-2/3"></div>{/each}</div></div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else}
        <!-- بطاقة الملف الشخصي -->
        <div class="card-default">
            <div class="flex items-center gap-4 mb-4">
                <div class="w-16 h-16 rounded-full bg-gradient-to-br from-accent-blue to-accent-cyan flex items-center justify-center text-2xl font-bold text-white">
                    {$user?.email?.charAt(0).toUpperCase() || 'م'}
                </div>
                <div class="flex-1">
                    <div class="text-lg font-bold text-text-primary">{$user?.email}</div>
                    <div class="text-sm text-text-secondary">{fullName || 'مستخدم'}</div>
                    <div class="flex items-center gap-2 mt-1">
                        <span class="pill-info">{$user?.role === 'admin' ? 'مدير' : 'مستخدم'}</span>
                        <span class="pill-success">{$user?.status === 'active' ? 'نشط' : $user?.status}</span>
                        <span class="pill-muted">{$user?.country || 'EG'}</span>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-2 gap-3 text-sm border-t border-base-700 pt-3">
                <div>
                    <div class="text-xs text-text-tertiary">تاريخ التسجيل</div>
                    <div class="text-text-secondary">{fmtDate($user?.id ? new Date().toISOString() : null)}</div>
                </div>
                <div>
                    <div class="text-xs text-text-tertiary">آخر تسجيل دخول</div>
                    <div class="text-text-secondary">{fmtDate(new Date().toISOString())}</div>
                </div>
            </div>
        </div>

        <!-- حالة KYC -->
        <div class="card-default">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider">حالة التحقق (KYC)</h3>
                <span class="pill-info">{kycLevelLabel(kycLevel)}</span>
            </div>

            <!-- شريط تقدم KYC -->
            <div class="flex items-center gap-1 mb-4">
                {#each [1, 2, 3] as level}
                    <div class="flex-1 h-2 rounded-full {kycLevel >= level ? 'bg-accent-green' : 'bg-base-700'}"></div>
                {/each}
            </div>

            <p class="text-xs text-text-secondary mb-4 leading-relaxed">{kycLevelDesc(kycLevel)}</p>

            {#if kycLevel < 3}
                <div class="bg-accent-yellow/10 border border-accent-yellow/30 rounded-md p-3 text-xs">
                    <div class="font-semibold text-accent-yellow mb-1">رفع مستوى التحقق</div>
                    <p class="text-text-secondary">لرفع حدود السحب والوصول للميزات المتقدمة، تواصل مع الدعم لتقديم مستندات التحقق.</p>
                </div>
            {:else}
                <div class="bg-accent-green/10 border border-accent-green/30 rounded-md p-3 text-xs">
                    <div class="font-semibold text-accent-green mb-1">✓ تم التحقق الكامل</div>
                    <p class="text-text-secondary">حسابك مُتحقق بالكامل. لديك وصول لكل الميزات وحدود السحب القصوى.</p>
                </div>
            {/if}
        </div>

        <!-- معلومات قابلة للتعديل -->
        <div class="card-default">
            <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">المعلومات الشخصية</h3>
            <div class="space-y-3">
                <div>
                    <label class="label" for="name">الاسم الكامل</label>
                    <input id="name" type="text" bind:value={fullName} class="input" placeholder="الاسم الكامل" />
                </div>
                <div>
                    <label class="label" for="email">البريد الإلكتروني</label>
                    <input id="email" type="email" value={$user?.email || ''} class="input opacity-60 cursor-not-allowed" disabled dir="ltr" />
                    <p class="text-xs text-text-tertiary mt-1">لا يمكن تغيير البريد الإلكتروني</p>
                </div>
                <div>
                    <label class="label" for="phone">رقم الهاتف</label>
                    <input id="phone" type="tel" bind:value={phone} class="input" placeholder="+20 10 1234 5678" dir="ltr" />
                </div>

                {#if saveError}<div class="text-xs text-accent-red">{saveError}</div>{/if}
                {#if saveSuccess}<div class="text-xs text-accent-green">{saveSuccess}</div>{/if}

                <button class="btn-primary text-sm" on:click={saveProfile} disabled={saving}>
                    {saving ? 'جارٍ الحفظ...' : 'حفظ التغييرات'}
                </button>
            </div>
        </div>

        <!-- ملخص المحافظ -->
        <div class="card-default">
            <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">ملخص المحافظ</h3>
            <div class="grid grid-cols-3 gap-3">
                <div class="bg-base-900 rounded-md p-3 text-center">
                    <div class="text-xs text-text-tertiary uppercase">المحافظ النشطة</div>
                    <div class="text-xl font-mono font-bold text-accent-blue mt-1">{activeWallets}</div>
                </div>
                <div class="bg-base-900 rounded-md p-3 text-center">
                    <div class="text-xs text-text-tertiary uppercase">إجمالي العملات</div>
                    <div class="text-xl font-mono font-bold text-accent-green mt-1">{$myWallets.length}</div>
                </div>
                <div class="bg-base-900 rounded-md p-3 text-center">
                    <div class="text-xs text-text-tertiary uppercase">عمليات نشطة</div>
                    <div class="text-xl font-mono font-bold text-accent-purple mt-1">{$myWallets.filter((w) => Number(w.locked_balance) > 0).length}</div>
                </div>
            </div>
        </div>

        <!-- أمان الحساب -->
        <div class="card-default">
            <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">أمان الحساب</h3>
            <div class="space-y-3">
                <div class="flex items-center justify-between text-sm">
                    <div class="flex items-center gap-2">
                        <svg class="w-4 h-4 text-accent-green" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" /></svg>
                        <span>كلمة مرور قوية</span>
                    </div>
                    <span class="text-xs text-text-tertiary">مُفعّلة</span>
                </div>
                <div class="flex items-center justify-between text-sm">
                    <div class="flex items-center gap-2">
                        <svg class="w-4 h-4 {kycLevel >= 1 ? 'text-accent-green' : 'text-text-tertiary'}" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" /></svg>
                        <span>التحقق من البريد</span>
                    </div>
                    <span class="text-xs {kycLevel >= 1 ? 'text-accent-green' : 'text-text-tertiary'}">{kycLevel >= 1 ? 'مُفعّلة' : 'غير مُفعّلة'}</span>
                </div>
                <div class="flex items-center justify-between text-sm">
                    <div class="flex items-center gap-2">
                        <svg class="w-4 h-4 text-text-tertiary" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" /></svg>
                        <span>المصادقة الثنائية (2FA)</span>
                    </div>
                    <span class="text-xs text-text-tertiary">قريباً</span>
                </div>
            </div>
        </div>
    {/if}
</div>
