<script lang="ts">
    import { goto } from '$app/navigation';
    import { auth } from '$lib/api';
    import { setSession } from '$lib/stores';

    let email = '';
    let password = '';
    let fullName = '';
    let phone = '';
    let error = '';
    let loading = false;

    async function submit(e: Event) {
        e.preventDefault();
        if (password.length < 8) {
            error = 'كلمة المرور يجب أن تكون 8 أحرف على الأقل';
            return;
        }
        loading = true;
        error = '';
        try {
            const res = await auth.register({ email, password, full_name: fullName, phone: phone || undefined });
            setSession(res.access_token, res.user);
            if (typeof localStorage !== 'undefined') {
                localStorage.setItem('refresh_token', res.refresh_token);
            }
            await goto('/', { replace: true });
        } catch (e: any) {
            error = e.message || 'فشل التسجيل';
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head><title>إنشاء حساب · منصة الجنيه</title></svelte:head>

<div class="min-h-[80vh] flex items-center justify-center px-4">
    <div class="w-full max-w-md">
        <div class="text-center mb-8">
            <div class="w-14 h-14 mx-auto rounded-xl bg-gradient-to-br from-accent-blue to-accent-cyan flex items-center justify-center font-bold text-white text-base">EGP</div>
            <h1 class="text-2xl font-bold text-text-primary mt-4">إنشاء حساب جديد</h1>
            <p class="text-sm text-text-secondary mt-1">انضم إلى منصة الجنيه للعملات الرقمية</p>
        </div>

        <form class="card-default space-y-4" on:submit={submit}>
            <div>
                <label class="label" for="name">الاسم الكامل</label>
                <input id="name" type="text" bind:value={fullName} class="input" placeholder="أحمد علي" required />
            </div>
            <div>
                <label class="label" for="email">البريد الإلكتروني</label>
                <input id="email" type="email" bind:value={email} class="input" placeholder="you@example.com" required dir="ltr" />
            </div>
            <div>
                <label class="label" for="phone">الهاتف (اختياري)</label>
                <input id="phone" type="tel" bind:value={phone} class="input" placeholder="+20 10 1234 5678" dir="ltr" />
            </div>
            <div>
                <label class="label" for="pwd">كلمة المرور</label>
                <input id="pwd" type="password" bind:value={password} class="input" placeholder="8 أحرف على الأقل" required minlength="8" dir="ltr" />
            </div>
            {#if error}
                <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
            {/if}
            <button type="submit" class="w-full btn-primary" disabled={loading}>
                {loading ? 'جارٍ إنشاء الحساب...' : 'إنشاء الحساب'}
            </button>
            <div class="text-center text-sm text-text-secondary">
                لديك حساب بالفعل؟ <a href="/login" class="text-accent-blue hover:underline">تسجيل الدخول</a>
            </div>
        </form>
    </div>
</div>
