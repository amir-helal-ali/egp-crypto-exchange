<script lang="ts">
    import { onMount } from 'svelte';
    import { navigate } from '$app/navigation';
    import { auth } from '$lib/api';
    import { setSession, isAuthenticated } from '$lib/stores';

    let email = '';
    let password = '';
    let error = '';
    let loading = false;

    onMount(() => {
        if ($isAuthenticated) navigate('/', { replace: true });
    });

    async function submit(e: Event) {
        e.preventDefault();
        if (!email || !password) {
            error = 'Please enter email and password';
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
            await navigate('/', { replace: true });
        } catch (e: any) {
            error = e.message || 'Login failed';
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head><title>Sign in · EGP Exchange</title></svelte:head>

<div class="min-h-[80vh] flex items-center justify-center px-4">
    <div class="w-full max-w-md">
        <div class="text-center mb-8">
            <div class="w-12 h-12 mx-auto rounded-lg bg-gradient-to-br from-accent-blue to-accent-cyan flex items-center justify-center font-bold text-white text-lg">EGP</div>
            <h1 class="text-2xl font-bold text-text-primary mt-4">Welcome back</h1>
            <p class="text-sm text-text-secondary mt-1">Sign in to your EGP Exchange account</p>
        </div>

        <form class="card-default space-y-4" on:submit={submit}>
            <div>
                <label class="label" for="email">Email</label>
                <input id="email" type="email" bind:value={email} class="input" placeholder="you@example.com" autocomplete="email" required />
            </div>
            <div>
                <label class="label" for="pwd">Password</label>
                <input id="pwd" type="password" bind:value={password} class="input" placeholder="••••••••" autocomplete="current-password" required />
            </div>
            {#if error}
                <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
            {/if}
            <button type="submit" class="w-full btn-primary" disabled={loading}>
                {loading ? 'Signing in...' : 'Sign in'}
            </button>
            <div class="text-center text-sm text-text-secondary">
                Don't have an account? <a href="/register" class="text-accent-blue hover:underline">Register</a>
            </div>
        </form>

        <div class="mt-6 text-xs text-text-tertiary text-center">
            Default admin: <code class="bg-base-700 px-1.5 py-0.5 rounded">admin@egp-exchange.local</code> / <code class="bg-base-700 px-1.5 py-0.5 rounded">ChangeMe!Admin2024</code>
        </div>
    </div>
</div>
