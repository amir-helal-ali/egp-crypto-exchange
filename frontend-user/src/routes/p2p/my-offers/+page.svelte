<script lang="ts">
    import { onMount } from 'svelte';
    import { p2p } from '$lib/api';
    import { myP2POffers, pushNotification } from '$lib/stores';
    import { fmtEgp, fmtRelative } from '$lib/format';
    import type { P2POffer, P2POfferStatus } from '$lib/types';

    let loading = true;
    let error = '';

    async function load() {
        try {
            const offers = await p2p.listMyOffers();
            myP2POffers.set(offers);
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);

    async function updateStatus(offer: P2POffer, newStatus: P2POfferStatus) {
        try {
            await p2p.updateOfferStatus(offer.id, newStatus);
            pushNotification({
                type: 'success',
                title: 'تم التحديث',
                message: `حالة العرض: ${newStatus === 'active' ? 'نشط' : newStatus === 'paused' ? 'متوقف مؤقتاً' : 'مغلق'}`,
            });
        } catch (e: any) {
            pushNotification({ type: 'error', title: 'خطأ', message: e.message });
        }
    }

    function statusLabel(s: string): string {
        return s === 'active' ? 'نشط' : s === 'paused' ? 'متوقف مؤقتاً' : 'مغلق';
    }
    function statusPill(s: string): string {
        return s === 'active' ? 'pill-success' : s === 'paused' ? 'pill-warning' : 'pill-muted';
    }
    function paymentLabel(id: string): string {
        const map: Record<string, string> = {
            bank_transfer: 'بنك', vodafone_cash: 'فودافون', instapay: 'إنستا باي',
            fawry: 'فوري', etisalat_cash: 'اتصالات', orange_cash: 'أورانج',
            we_pay: 'وي باي', cash_deposit: 'نقدي',
        };
        return map[id] || id;
    }
</script>

<svelte:head><title>عروضي · منصة الجنيه</title></svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between flex-wrap gap-2">
        <div>
            <h1 class="text-2xl font-bold text-text-primary">عروضي</h1>
            <p class="text-sm text-text-secondary mt-1">إدارة عروض التداول بين الأفراد</p>
        </div>
        <a href="/p2p/create" class="btn-primary text-sm">+ إنشاء عرض جديد</a>
    </div>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if $myP2POffers.length === 0}
        <div class="card-default text-center py-12">
            <div class="text-text-tertiary text-sm mb-4">لم تنشئ أي عروض بعد</div>
            <a href="/p2p/create" class="btn-primary text-sm">+ أنشئ أول عرض</a>
        </div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>النوع</th>
                        <th>الأصل</th>
                        <th class="num-cell">هامش السعر</th>
                        <th class="num-cell">الحد الأدنى</th>
                        <th class="num-cell">الحد الأقصى</th>
                        <th>طرق الدفع</th>
                        <th class="num-cell">الصفقات</th>
                        <th>الحالة</th>
                        <th class="num-cell">منذ</th>
                        <th>إجراءات</th>
                    </tr>
                </thead>
                <tbody>
                    {#each $myP2POffers as offer}
                        <tr>
                            <td>
                                <span class={offer.side === 'buy' ? 'pill-success' : 'pill-info'}>
                                    {offer.side === 'buy' ? 'شراء' : 'بيع'}
                                </span>
                            </td>
                            <td class="font-bold">{offer.asset_symbol}</td>
                            <td class="num-cell">{offer.price_margin_pct > 0 ? '+' : ''}{offer.price_margin_pct}%</td>
                            <td class="num-cell">{fmtEgp(offer.min_amount_egp)}</td>
                            <td class="num-cell">{fmtEgp(offer.max_amount_egp)}</td>
                            <td class="text-xs text-text-secondary">
                                {offer.payment_methods.map((p) => paymentLabel(p)).join('، ')}
                            </td>
                            <td class="num-cell">{offer.total_trades}</td>
                            <td><span class={statusPill(offer.status)}>{statusLabel(offer.status)}</span></td>
                            <td class="num-cell text-text-tertiary text-xs">{fmtRelative(offer.created_at)}</td>
                            <td>
                                <div class="flex gap-1">
                                    {#if offer.status === 'active'}
                                        <button class="text-xs text-accent-yellow hover:underline" on:click={() => updateStatus(offer, 'paused')}>إيقاف</button>
                                    {:else if offer.status === 'paused'}
                                        <button class="text-xs text-accent-green hover:underline" on:click={() => updateStatus(offer, 'active')}>تفعيل</button>
                                    {/if}
                                    {#if offer.status !== 'closed'}
                                        <button class="text-xs text-accent-red hover:underline mr-2" on:click={() => updateStatus(offer, 'closed')}>إغلاق</button>
                                    {/if}
                                </div>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
