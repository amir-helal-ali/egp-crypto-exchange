<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { page } from '$app/stores';
    import { navigate } from '$app/navigation';
    import { user, isAuthenticated, connectMarketWs, disconnectMarketWs, clearSession, circuitOpen } from '$lib/stores';
    import CircuitBanner from '$lib/components/CircuitBanner.svelte';

    let mobileOpen = false;

    onMount(() => {
        connectMarketWs();
    });
    onDestroy(() => {
        disconnectMarketWs();
    });

    async function logout() {
        clearSession();
        await navigate('/login', { replace: true });
    }

    const navItems = [
        { href: '/', label: 'Dashboard' },
        { href: '/trade/BTC_EGP', label: 'Trade' },
        { href: '/wallet', label: 'Wallet' },
        { href: '/history', label: 'History' },
    ];

    $: path = $page.url.pathname;
    $: isActive = (href: string) => {
        if (href === '/') return path === '/';
        if (href.startsWith('/trade')) return path.startsWith('/trade');
        return path.startsWith(href);
    };
</script>

<div class="min-h-screen flex flex-col bg-base-900">
    <!-- Header -->
    <header class="bg-base-800 border-b border-base-600 sticky top-0 z-30 backdrop-blur-md bg-base-800/90">
        <div class="max-w-[1600px] mx-auto px-4 sm:px-6 lg:px-8 flex items-center justify-between h-14">
            <div class="flex items-center gap-3">
                <button class="lg:hidden text-text-secondary" on:click={() => (mobileOpen = !mobileOpen)} aria-label="menu">
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
                    </svg>
                </button>
                <a href="/" class="flex items-center gap-2">
                    <div class="w-8 h-8 rounded-md bg-gradient-to-br from-accent-blue to-accent-cyan flex items-center justify-center font-bold text-white text-sm">EGP</div>
                    <span class="font-semibold text-text-primary">EGP Exchange</span>
                </a>
            </div>

            <!-- Desktop nav -->
            <nav class="hidden lg:flex items-center gap-1">
                {#each navItems as item}
                    <a href={item.href} class="px-3 py-1.5 rounded-md text-sm font-medium {isActive(item.href) ? 'bg-base-700 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-base-700/50'} transition-colors">
                        {item.label}
                    </a>
                {/each}
            </nav>

            <!-- Right cluster -->
            <div class="flex items-center gap-3">
                {#if $circuitOpen}
                    <div class="hidden md:flex items-center gap-1.5 text-xs text-accent-red bg-accent-red/10 px-2 py-1 rounded-full border border-accent-red/30">
                        <span class="w-1.5 h-1.5 rounded-full bg-accent-red animate-pulse"></span>
                        Feed Halted
                    </div>
                {/if}
                {#if $isAuthenticated}
                    <div class="hidden sm:flex items-center gap-2 px-3 py-1.5 rounded-md bg-base-700 border border-base-600">
                        <div class="w-7 h-7 rounded-full bg-accent-blue/20 text-accent-blue flex items-center justify-center text-xs font-semibold">
                            {$user?.email.charAt(0).toUpperCase() || 'U'}
                        </div>
                        <div class="text-xs">
                            <div class="text-text-primary font-medium">{$user?.email}</div>
                            <div class="text-text-tertiary uppercase">{$user?.role}</div>
                        </div>
                    </div>
                    <button class="btn-ghost text-sm" on:click={logout}>Logout</button>
                {:else}
                    <a href="/login" class="btn-primary text-sm">Sign in</a>
                {/if}
            </div>
        </div>

        <!-- Mobile nav -->
        {#if mobileOpen}
            <nav class="lg:hidden border-t border-base-700 px-4 py-3 space-y-1">
                {#each navItems as item}
                    <a href={item.href} on:click={() => (mobileOpen = false)} class="block px-3 py-2 rounded-md text-sm font-medium {isActive(item.href) ? 'bg-base-700 text-text-primary' : 'text-text-secondary hover:bg-base-700/50'}">
                        {item.label}
                    </a>
                {/each}
            </nav>
        {/if}
    </header>

    <!-- Body -->
    <main class="flex-1 max-w-[1600px] w-full mx-auto px-4 sm:px-6 lg:px-8 py-4 sm:py-6">
        {#if $isAuthenticated || $page.url.pathname === '/login' || $page.url.pathname === '/register'}
            <div class="mb-4">
                <CircuitBanner />
            </div>
        {/if}
        <slot />
    </main>

    <footer class="border-t border-base-700 mt-12 py-6">
        <div class="max-w-[1600px] mx-auto px-4 sm:px-6 lg:px-8 text-center text-xs text-text-tertiary">
            EGP Exchange &middot; Closed-loop &middot; Binance price feed &middot; {new Date().getFullYear()}
        </div>
    </footer>
</div>
