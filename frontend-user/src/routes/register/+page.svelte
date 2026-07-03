<script lang="ts">
    import { navigate } from '$app/navigation';
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
            error = 'Password must be at least 8 characters';
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
            await navigate('/', { replace: true });
        } catch (e: any) {
            error = e.message || 'Registration failed';
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head><title>Register · EGP Exchange</title></svelte:head>

<div class="min-h-[80vh] flex items-center justify-center px-4">
    <div class="w-full max-w-md">
        <div class="text-center mb-8">
            <div class="w-12 h-12 mx-auto rounded-lg bg-gradient-to-br from-accent-blue to-accent-cyan flex items-center justify-center font-bold text-white text-lg">EGP</div>
            <h1 class="text-2xl font-bold text-text-primary mt-4">Create your account</h1>
            <p class="text-sm text-text-secondary mt-1">Join the EGP-based crypto exchange</p>
        </div>

        <form class="card-default space-y-4" on:submit={submit}>
            <div>
                <label class="label" for="name">Full name</label>
                <input id="name" type="text" bind:value={fullName} class="input" placeholder="Ahmed Ali" required />
            </div>
            <div>
                <label class="label" for="email">Email</label>
                <input id="email" type="email" bind:value={email} class="input" placeholder="you@example.com" required />
            </div>
            <div>
                <label class="label" for="phone">Phone (optional)</label>
                <input id="phone" type="tel" bind:value={phone} class="input" placeholder="+20 10 1234 5678" />
            </div>
            <div>
                <label class="label" for="pwd">Password</label>
                <input id="pwd" type="password" bind:value={password} class="input" placeholder="At least 8 characters" required minlength="8" />
            </div>
            {#if error}
                <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
            {/if}
            <button type="submit" class="w-full btn-primary" disabled={loading}>
                {loading ? 'Creating account...' : 'Create account'}
            </button>
            <div class="text-center text-sm text-text-secondary">
                Already have an account? <a href="/login" class="text-accent-blue hover:underline">Sign in</a>
            </div>
        </form>
    </div>
</div>
