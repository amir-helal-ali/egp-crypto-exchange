<script lang="ts">
    import { onMount } from 'svelte';
    import { overview } from '$lib/api';
    import { fmtEgp, fmtNum, fmtQty } from '$lib/format';
    import type { Overview } from '$lib/types';

    let data: Overview | null = null;
    let loading = true;
    let error = '';

    async function load() {
        try {
            data = await overview.get();
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);
</script>

<svelte:head><title>نظرة عامة · الإدارة</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">نظرة عامة</h1>
        <p class="text-sm text-text-secondary mt-1">الحالة اللحظية والمؤشرات الأساسية</p>
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if data}
        <!-- بطاقات المؤشرات -->
        <section class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">المستخدمون</div>
                <div class="text-2xl font-mono font-bold text-text-primary mt-1">{fmtNum(data.users, 0)}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">الأوامر المفتوحة</div>
                <div class="text-2xl font-mono font-bold text-accent-blue mt-1">{fmtNum(data.orders.open, 0)}</div>
                <div class="text-xs text-text-tertiary">من {data.orders.total}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">إجمالي الصفقات</div>
                <div class="text-2xl font-mono font-bold text-accent-green mt-1">{fmtNum(data.trades, 0)}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">إيداعات معلقة</div>
                <div class="text-2xl font-mono font-bold text-accent-yellow mt-1">{fmtNum(data.pending.deposits, 0)}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">سحوبات معلقة</div>
                <div class="text-2xl font-mono font-bold text-accent-yellow mt-1">{fmtNum(data.pending.withdrawals, 0)}</div>
            </div>
            <div class="card-compact">
                <div class="text-xs text-text-tertiary uppercase">قاطع الدائرة</div>
                {#if data.circuit_breaker_open}
                    <div class="text-2xl font-bold text-accent-red mt-1">مفتوح</div>
                    <div class="text-xs text-accent-red">التداول متوقف</div>
                {:else}
                    <div class="text-2xl font-bold text-accent-green mt-1">مغلق</div>
                    <div class="text-xs text-accent-green">التدفق سليم</div>
                {/if}
            </div>
        </section>

        <!-- جدول السيولة -->
        <section>
            <h2 class="text-sm font-semibold text-text-secondary uppercase tracking-wider mb-3">سيولة النظام</h2>
            <div class="card-default overflow-x-auto">
                <table class="table-pro">
                    <thead>
                        <tr>
                            <th>الأصل</th>
                            <th class="num-cell">إجمالي الرصيد</th>
                            <th class="num-cell">المحجوز</th>
                            <th class="num-cell">المتاح</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each data.liquidity as l}
                            <tr>
                                <td class="font-bold">{l.asset}</td>
                                <td class="num-cell">{l.asset === 'EGP' ? fmtEgp(l.balance) : fmtQty(l.balance, 8)}</td>
                                <td class="num-cell text-text-secondary">{l.asset === 'EGP' ? fmtEgp(l.locked) : fmtQty(l.locked, 8)}</td>
                                <td class="num-cell text-accent-green">{l.asset === 'EGP' ? fmtEgp((Number(l.balance) - Number(l.locked)).toString()) : fmtQty((Number(l.balance) - Number(l.locked)).toString(), 8)}</td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </section>

        <!-- روابط سريعة -->
        <section class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <a href="/deposits" class="card-default hover:border-accent-yellow transition-colors block">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold text-text-primary">معالجة الإيداعات</div>
                        <div class="text-xs text-text-tertiary mt-1">تحقق من التحويلات البنكية</div>
                    </div>
                    <div class="text-2xl font-bold text-accent-yellow">{data.pending.deposits}</div>
                </div>
            </a>
            <a href="/withdrawals" class="card-default hover:border-accent-red transition-colors block">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold text-text-primary">معالجة السحوبات</div>
                        <div class="text-xs text-text-tertiary mt-1">أطلق العملات الرقمية</div>
                    </div>
                    <div class="text-2xl font-bold text-accent-red">{data.pending.withdrawals}</div>
                </div>
            </a>
            <a href="/currencies" class="card-default hover:border-accent-blue transition-colors block">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold text-text-primary">إدارة العملات</div>
                        <div class="text-xs text-text-tertiary mt-1">التحكم في العملات والرسوم</div>
                    </div>
                    <svg class="w-6 h-6 text-accent-blue" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8" /></svg>
                </div>
            </a>
            <a href="/settings" class="card-default hover:border-accent-purple transition-colors block">
                <div class="flex items-center justify-between">
                    <div>
                        <div class="text-sm font-semibold text-text-primary">الإعدادات</div>
                        <div class="text-xs text-text-tertiary mt-1">سعر الصرف والرسوم</div>
                    </div>
                    <svg class="w-6 h-6 text-accent-purple" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" /></svg>
                </div>
            </a>
        </section>
    {/if}
</div>
