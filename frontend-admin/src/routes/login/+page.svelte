<script lang="ts">
    import { onMount } from 'svelte';
    import { navigate } from '$app/navigation';
    import { auth } from '$lib/api';
    import { setAdminSession, isAdminAuthenticated } from '$lib/stores';

    let email = '';
    let password = '';
    let error = '';
    let loading = false;

    onMount(() => {
        if ($isAdminAuthenticated) navigate('/', { replace: true });
    });

    async function submit(e: Event) {
        e.preventDefault();
        loading = true;
        error = '';
        try {
            const res = await auth.login(email, password);
            if (res.user.role !== 'admin') {
                throw new Error('Access denied — admin role required');
            }
            setAdminSession(res.access_token, res.user);
            if (typeof localStorage !== 'undefined') {
                localStorage.setItem('admin_refresh_token', res.refresh_token);
            }
            await navigate('/', { replace: true });
        } catch (e: any) {
            error = e.message || 'Login failed';
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head><title>Admin Sign in · EGP Exchange</title></svelte:head>

<div class="min-h-screen flex items-center justify-center px-4 bg-base-900">
    <div class="w-full max-w-md">
        <div class="text-center mb-8">
            <div class="w-12 h-12 mx-auto rounded-lg bg-gradient-to-br from-accent-purple to-accent-blue flex items-center justify-center font-bold text-white text-sm">ADM</div>
            <h1 class="text-2xl font-bold text-text-primary mt-4">Admin Panel</h1>
            <p class="text-sm text-text-secondary mt-1">EGP Exchange · Restricted access</p>
        </div>

        <form class="card-default space-y-4" on:submit={submit}>
            <div>
                <label class="label" for="email">Admin Email</label>
                <input id="email" type="email" bind:value={email} class="input" placeholder="admin@egp-exchange.local" required />
            </div>
            <div>
                <label class="label" for="pwd">Password</label>
                <input id="pwd" type="password" bind:value={password} class="input" placeholder="••••••••" required />
            </div>
            {#if error}
                <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
            {/if}
            <button type="submit" class="w-full btn-primary" disabled={loading}>
                {loading ? 'Signing in...' : 'Sign in as Admin'}
            </button>
        </form>

        <div class="mt-6 text-xs text-text-tertiary text-center">
            <p>This panel is restricted to authorized administrators.</p>
            <p class="mt-2">Default: <code class="bg-base-700 px-1.5 py-0.5 rounded">admin@egp-exchange.local</code> / <code class="bg-base-700 px-1.5 py-0.5 rounded">ChangeMe!Admin2024</code></p>
        </div>
    </div>
</div>
