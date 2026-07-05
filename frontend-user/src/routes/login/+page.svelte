<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { auth } from '$lib/api';
    import { setSession, isAuthenticated } from '$lib/stores';

    let email = '';
    let password = '';
    let error = '';
    let loading = false;

    onMount(() => {
        if ($isAuthenticated) goto('/', { replace: true });
    });

    async function submit(e: Event) {
        e.preventDefault();
        if (!email || !password) {
            error = 'الرجاء إدخال البريد وكلمة المرور';
            return;
        }
        loading = true;
        error = '';
        try {
            const res = await auth.login({ email, password });
            setSession(res.access_token, res.user);
            if (typeof localStorage !== 'undefined') {
                localStorage.setItem('refresh_token', res.refresh_token);
            }
            await goto('/', { replace: true });
        } catch (e: any) {
            error = e.message || 'فشل تسجيل الدخول';
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head><title>تسجيل الدخول · منصة الجنيه</title></svelte:head>

<div class="min-h-[80vh] flex items-center justify-center px-4">
    <div class="w-full max-w-md">
        <div class="text-center mb-8">
            <div class="w-14 h-14 mx-auto rounded-xl bg-gradient-to-br from-accent-blue to-accent-cyan flex items-center justify-center font-bold text-white text-base">EGP</div>
            <h1 class="text-2xl font-bold text-text-primary mt-4">مرحباً بعودتك</h1>
            <p class="text-sm text-text-secondary mt-1">سجّل الدخول إلى حسابك في منصة الجنيه</p>
        </div>

        <form class="card-default space-y-4" on:submit={submit}>
            <div>
                <label class="label" for="email">البريد الإلكتروني</label>
                <input id="email" type="email" bind:value={email} class="input" placeholder="you@example.com" autocomplete="email" required dir="ltr" />
            </div>
            <div>
                <label class="label" for="pwd">كلمة المرور</label>
                <input id="pwd" type="password" bind:value={password} class="input" placeholder="••••••••" autocomplete="current-password" required dir="ltr" />
            </div>
            {#if error}
                <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
            {/if}
            <button type="submit" class="w-full btn-primary" disabled={loading}>
                {loading ? 'جارٍ تسجيل الدخول...' : 'تسجيل الدخول'}
            </button>
            <div class="text-center text-sm text-text-secondary">
                ليس لديك حساب؟ <a href="/register" class="text-accent-blue hover:underline">إنشاء حساب</a>
            </div>
        </form>

        <div class="mt-6 text-xs text-text-tertiary text-center">
            <p>المدير الافتراضي: <code class="bg-base-700 px-1.5 py-0.5 rounded">admin@egp-exchange.local</code> / <code class="bg-base-700 px-1.5 py-0.5 rounded">ChangeMe!Admin2024</code></p>
        </div>
    </div>
</div>
