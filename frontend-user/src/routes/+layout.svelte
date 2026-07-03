<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { page } from '$app/stores';
    import { navigate } from '$app/navigation';
    import { user, isAuthenticated, connectMarketWs, disconnectMarketWs, clearSession, circuitOpen } from '$lib/stores';
    import { _t, lang, setLang } from '$lib/i18n';
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
        { href: '/', label: 'لوحة التحكم', icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
        { href: '/trade/BTC_EGP', label: 'التداول', icon: 'M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z' },
        { href: '/futures/BTC_EGP', label: 'العقود الآجلة', icon: 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6' },
        { href: '/p2p', label: 'تداول فردي', icon: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z' },
        { href: '/wallet', label: 'المحفظة', icon: 'M21 12a2.25 2.25 0 00-2.25-2.25H15a3 3 0 11-6 0H5.25A2.25 2.25 0 003 12m18 0v6a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 18v-6m18 0V9M3 12V9m18 0a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 9m18 0V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v3' },
        { href: '/history', label: 'السجل', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
    ];

    $: path = $page.url.pathname;
    $: isActive = (href: string) => {
        if (href === '/') return path === '/';
        if (href.startsWith('/trade')) return path.startsWith('/trade');
        if (href.startsWith('/futures')) return path.startsWith('/futures');
        return path.startsWith(href);
    };

    function toggleLang() {
        setLang($lang === 'ar' ? 'en' : 'ar');
    }
</script>

<div class="min-h-screen flex flex-col bg-base-900">
    <!-- الترويسة -->
    <header class="bg-base-800 border-b border-base-600 sticky top-0 z-30 glass">
        <div class="max-w-[1600px] mx-auto px-4 sm:px-6 lg:px-8 flex items-center justify-between h-14">
            <div class="flex items-center gap-3">
                <button class="lg:hidden text-text-secondary" on:click={() => (mobileOpen = !mobileOpen)} aria-label="القائمة">
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
                    </svg>
                </button>
                <a href="/" class="flex items-center gap-2">
                    <div class="w-9 h-9 rounded-lg bg-gradient-to-br from-accent-blue to-accent-cyan flex items-center justify-center font-bold text-white text-xs">EGP</div>
                    <div class="hidden sm:block">
                        <div class="font-bold text-text-primary text-sm leading-tight">منصة الجنيه</div>
                        <div class="text-[10px] text-text-tertiary leading-tight">للعملات الرقمية</div>
                    </div>
                </a>
            </div>

            <!-- روابط سطح المكتب -->
            <nav class="hidden lg:flex items-center gap-1">
                {#each navItems as item}
                    <a href={item.href} class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium {isActive(item.href) ? 'bg-base-700 text-text-primary' : 'text-text-secondary hover:text-text-primary hover:bg-base-700/50'} transition-colors">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.8">
                            <path stroke-linecap="round" stroke-linejoin="round" d={item.icon} />
                        </svg>
                        <span>{item.label}</span>
                    </a>
                {/each}
            </nav>

            <!-- العناصر اليمنى -->
            <div class="flex items-center gap-3">
                {#if $circuitOpen}
                    <div class="hidden md:flex items-center gap-1.5 text-xs text-accent-red bg-accent-red/10 px-2 py-1 rounded-full border border-accent-red/30">
                        <span class="w-1.5 h-1.5 rounded-full bg-accent-red animate-pulse"></span>
                        تدفق الأسعار متوقف
                    </div>
                {/if}
                <button on:click={toggleLang} class="text-xs text-text-secondary hover:text-text-primary px-2 py-1 rounded border border-base-600 hover:border-base-500 transition-colors">
                    {$lang === 'ar' ? 'EN' : 'ع'}
                </button>
                {#if $isAuthenticated}
                    <div class="hidden sm:flex items-center gap-2 px-3 py-1.5 rounded-md bg-base-700 border border-base-600">
                        <div class="w-7 h-7 rounded-full bg-accent-blue/20 text-accent-blue flex items-center justify-center text-xs font-semibold">
                            {$user?.email.charAt(0).toUpperCase() || 'م'}
                        </div>
                        <div class="text-xs">
                            <div class="text-text-primary font-medium max-w-[140px] truncate">{$user?.email}</div>
                            <div class="text-text-tertiary uppercase">{$user?.role === 'admin' ? 'مدير' : 'مستخدم'}</div>
                        </div>
                    </div>
                    <button class="btn-ghost text-sm" on:click={logout}>خروج</button>
                {:else}
                    <a href="/login" class="btn-primary text-sm">دخول</a>
                {/if}
            </div>
        </div>

        <!-- قائمة الجوال -->
        {#if mobileOpen}
            <nav class="lg:hidden border-t border-base-700 px-4 py-3 space-y-1 bg-base-800">
                {#each navItems as item}
                    <a href={item.href} on:click={() => (mobileOpen = false)} class="flex items-center gap-2 px-3 py-2 rounded-md text-sm font-medium {isActive(item.href) ? 'bg-base-700 text-text-primary' : 'text-text-secondary hover:bg-base-700/50'}">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.8">
                            <path stroke-linecap="round" stroke-linejoin="round" d={item.icon} />
                        </svg>
                        {item.label}
                    </a>
                {/each}
            </nav>
        {/if}
    </header>

    <!-- المحتوى الرئيسي -->
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
            منصة الجنيه · حلقة مغلقة · تدفق أسعار بينانس · {new Date().getFullYear()}
        </div>
    </footer>
</div>
