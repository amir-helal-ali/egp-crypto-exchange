<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { page } from '$app/stores';
    import { navigate } from '$app/navigation';
    import { adminUser, isAdminAuthenticated, clearAdminSession } from '$lib/stores';

    let sidebarOpen = false;

    const navItems = [
        { href: '/', label: 'Overview', icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
        { href: '/deposits', label: 'Deposits', icon: 'M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z' },
        { href: '/withdrawals', label: 'Withdrawals', icon: 'M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z' },
        { href: '/users', label: 'Users', icon: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z' },
        { href: '/liquidity', label: 'Liquidity', icon: 'M3 6l3 1m0 0l-3 9a5.002 5.002 0 006.001 0M6 7l3 9M6 7l6-2m6 2l3-1m-3 1l-3 9a5.002 5.002 0 006.001 0M18 7l3 9m-3-9l-6-2m0-2v2m0 16V5m0 16H9m3 0h3' },
        { href: '/orders', label: 'Orders', icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2' },
        { href: '/trades', label: 'Trades', icon: 'M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4' },
        { href: '/audit', label: 'Audit Log', icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z' },
    ];

    onMount(() => {
        if (!$isAdminAuthenticated && $page.url.pathname !== '/login') {
            navigate('/login', { replace: true });
        }
    });

    async function logout() {
        clearAdminSession();
        await navigate('/login', { replace: true });
    }

    $: path = $page.url.pathname;
    $: isActive = (href: string) => {
        if (href === '/') return path === '/';
        return path.startsWith(href);
    };
</script>

<div class="min-h-screen flex bg-base-900">
    <!-- Sidebar -->
    <aside class="fixed lg:static inset-y-0 left-0 z-40 w-60 bg-base-800 border-r border-base-600 flex flex-col transform {sidebarOpen ? 'translate-x-0' : '-translate-x-full'} lg:translate-x-0 transition-transform">
        <div class="px-4 py-4 border-b border-base-700 flex items-center gap-2">
            <div class="w-9 h-9 rounded-md bg-gradient-to-br from-accent-purple to-accent-blue flex items-center justify-center font-bold text-white text-xs">ADM</div>
            <div>
                <div class="font-semibold text-text-primary text-sm">EGP Exchange</div>
                <div class="text-xs text-text-tertiary">Admin Panel</div>
            </div>
        </div>

        <nav class="flex-1 overflow-y-auto py-2">
            {#each navItems as item}
                <a href={item.href} on:click={() => (sidebarOpen = false)}
                   class="flex items-center gap-3 px-4 py-2.5 text-sm font-medium {isActive(item.href) ? 'bg-base-700 text-text-primary border-l-2 border-accent-blue' : 'text-text-secondary hover:bg-base-700/50 hover:text-text-primary border-l-2 border-transparent'} transition-colors">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
                        <path stroke-linecap="round" stroke-linejoin="round" d={item.icon} />
                    </svg>
                    {item.label}
                </a>
            {/each}
        </nav>

        <div class="border-t border-base-700 p-3">
            {#if $adminUser}
                <div class="px-2 py-2 mb-2">
                    <div class="text-sm text-text-primary font-medium truncate">{$adminUser.email}</div>
                    <div class="text-xs text-text-tertiary uppercase">{$adminUser.role}</div>
                </div>
            {/if}
            <button class="w-full btn-ghost text-sm" on:click={logout}>Logout</button>
        </div>
    </aside>

    {#if sidebarOpen}
        <div class="fixed inset-0 bg-base-900/70 z-30 lg:hidden" on:click={() => (sidebarOpen = false)}></div>
    {/if}

    <!-- Main -->
    <div class="flex-1 flex flex-col min-w-0">
        <header class="lg:hidden bg-base-800 border-b border-base-600 px-4 h-14 flex items-center justify-between">
            <button on:click={() => (sidebarOpen = !sidebarOpen)} aria-label="menu">
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
                </svg>
            </button>
            <span class="font-semibold text-text-primary">Admin</span>
            <div class="w-6"></div>
        </header>

        <main class="flex-1 overflow-x-auto px-4 sm:px-6 lg:px-8 py-6">
            <slot />
        </main>
    </div>
</div>
