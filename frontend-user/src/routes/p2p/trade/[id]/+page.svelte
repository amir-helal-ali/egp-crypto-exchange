<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { page } from '$app/stores';
    import { navigate } from '$app/navigation';
    import { p2p } from '$lib/api';
    import { p2pMessages, p2pTradeUpdates, user, pushNotification } from '$lib/stores';
    import { fmtEgp, fmtQty, fmtDate, fmtRelative } from '$lib/format';
    import type { P2PTrade, P2PMessage } from '$lib/types';

    let trade: P2PTrade | null = null;
    let loading = true;
    let error = '';
    let newMessage = '';
    let sendingMessage = false;
    let actionLoading = false;
    let messages: P2PMessage[] = [];
    let messagesContainer: HTMLElement;

    $: id = $page.params.id;
    $: myId = $user?.id || '';
    $: isBuyer = trade?.buyer_id === myId;
    $: isSeller = trade?.seller_id === myId;

    // الاشتراك في تحديثات الصفقة عبر WebSocket
    const unsubTrade = p2pTradeUpdates.subscribe((t) => {
        if (t && t.id === id) {
            trade = t;
        }
    });

    // الاشتراك في الرسائل الجديدة
    const unsubMsgs = p2pMessages.subscribe((map) => {
        if (map[id]) {
            messages = map[id];
            scrollToBottom();
        }
    });

    function scrollToBottom() {
        if (messagesContainer) {
            setTimeout(() => {
                messagesContainer.scrollTop = messagesContainer.scrollHeight;
            }, 50);
        }
    }

    async function load() {
        try {
            trade = await p2p.getTrade(id);
            messages = await p2p.listMessages(id);
            // تحديث متجر الرسائل
            p2pMessages.update((m) => ({ ...m, [id]: messages }));
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);
    onDestroy(() => {
        unsubTrade();
        unsubMsgs();
    });

    async function sendMessage() {
        if (!newMessage.trim() || !trade) return;
        sendingMessage = true;
        try {
            await p2p.sendMessage(trade.id, newMessage.trim());
            newMessage = '';
        } catch (e: any) {
            pushNotification({ type: 'error', title: 'خطأ', message: e.message });
        } finally {
            sendingMessage = false;
        }
    }

    async function confirmPaid() {
        if (!trade || !confirm('تأكيد أنك قمت بالدفع؟')) return;
        actionLoading = true;
        try {
            await p2p.confirmPaid(trade.id);
            pushNotification({ type: 'success', title: 'تم التأكيد', message: 'تم تأكيد الدفع' });
        } catch (e: any) {
            pushNotification({ type: 'error', title: 'خطأ', message: e.message });
        } finally {
            actionLoading = false;
        }
    }

    async function releaseCrypto() {
        if (!trade || !confirm('تأكيد إطلاق العملات للمشتري؟')) return;
        actionLoading = true;
        try {
            await p2p.releaseCrypto(trade.id);
            pushNotification({ type: 'success', title: 'تم الإطلاق', message: 'تم إطلاق العملات بنجاح' });
        } catch (e: any) {
            pushNotification({ type: 'error', title: 'خطأ', message: e.message });
        } finally {
            actionLoading = false;
        }
    }

    async function cancelTrade() {
        if (!trade || !confirm('هل تريد إلغاء هذه الصفقة؟')) return;
        actionLoading = true;
        try {
            await p2p.cancelTrade(trade.id);
            pushNotification({ type: 'warning', title: 'تم الإلغاء', message: 'تم إلغاء الصفقة' });
        } catch (e: any) {
            pushNotification({ type: 'error', title: 'خطأ', message: e.message });
        } finally {
            actionLoading = false;
        }
    }

    function statusLabel(s: string): string {
        const map: Record<string, string> = {
            pending: 'بانتظار الدفع',
            paid: 'تم الدفع',
            released: 'تم الإطلاق',
            cancelled: 'ملغاة',
            disputed: 'نزاع',
            completed: 'مكتملة',
        };
        return map[s] || s;
    }

    function statusPill(s: string): string {
        const map: Record<string, string> = {
            pending: 'pill-warning',
            paid: 'pill-info',
            released: 'pill-info',
            cancelled: 'pill-muted',
            disputed: 'pill-danger',
            completed: 'pill-success',
        };
        return map[s] || 'pill-muted';
    }

    function paymentLabel(id: string): string {
        const map: Record<string, string> = {
            bank_transfer: 'تحويل بنكي',
            vodafone_cash: 'فودافون كاش',
            instapay: 'إنستا باي',
            fawry: 'فوري',
            etisalat_cash: 'اتصالات كاش',
            orange_cash: 'أورانج كاش',
            we_pay: 'وي باي',
            cash_deposit: 'إيداع نقدي',
        };
        return map[id] || id;
    }
</script>

<svelte:head><title>صفقة P2P · منصة الجنيه</title></svelte:head>

<div class="max-w-4xl mx-auto space-y-4">
    <a href="/p2p" class="text-xs text-accent-blue hover:underline">← العودة للسوق</a>

    {#if loading}
        <div class="text-center py-12 text-text-tertiary">جارٍ التحميل...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else if trade}
        <!-- رأس الصفقة -->
        <div class="card-default">
            <div class="flex items-center justify-between flex-wrap gap-3 mb-4">
                <div>
                    <h1 class="text-xl font-bold text-text-primary">
                        {isBuyer ? 'شراء' : 'بيع'} {trade.asset_symbol}
                    </h1>
                    <p class="text-xs text-text-tertiary mt-1">صفقة #{trade.id.slice(0, 8)} · {fmtRelative(trade.created_at)}</p>
                </div>
                <span class={statusPill(trade.status)}>{statusLabel(trade.status)}</span>
            </div>

            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 text-sm">
                <div>
                    <div class="text-xs text-text-tertiary">المبلغ</div>
                    <div class="num-cell font-bold text-text-primary">{fmtQty(trade.amount, 8)} {trade.asset_symbol}</div>
                </div>
                <div>
                    <div class="text-xs text-text-tertiary">السعر</div>
                    <div class="num-cell text-text-primary">{fmtEgp(trade.price_egp)}</div>
                </div>
                <div>
                    <div class="text-xs text-text-tertiary">الإجمالي</div>
                    <div class="num-cell font-bold text-accent-green">{fmtEgp(trade.total_egp)}</div>
                </div>
                <div>
                    <div class="text-xs text-text-tertiary">طريقة الدفع</div>
                    <div class="text-text-primary">{paymentLabel(trade.payment_method)}</div>
                </div>
            </div>
        </div>

        <!-- إجراءات الصفقة -->
        {#if trade.status === 'pending' || trade.status === 'paid'}
            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">إجراءات الصفقة</h3>
                <div class="space-y-3">
                    {#if trade.status === 'pending' && isBuyer}
                        <div class="bg-accent-yellow/10 border border-accent-yellow/30 rounded-md p-3 text-sm">
                            <div class="font-semibold text-accent-yellow mb-1">بانتظار الدفع</div>
                            <p class="text-text-secondary text-xs">قم بالدفع إلى البائع عبر {paymentLabel(trade.payment_method)}، ثم اضغط "تأكيد الدفع".</p>
                        </div>
                        <button class="w-full btn-success text-sm" on:click={confirmPaid} disabled={actionLoading}>
                            {actionLoading ? 'جارٍ...' : 'تأكيد الدفع'}
                        </button>
                    {:else if trade.status === 'pending' && isSeller}
                        <div class="bg-accent-blue/10 border border-accent-blue/30 rounded-md p-3 text-sm">
                            <div class="font-semibold text-accent-blue mb-1">بانتظار الدفع من المشتري</div>
                            <p class="text-text-secondary text-xs">سيتم إشعارك فور تأكيد المشتري للدفع. لا تطلق العملات حتى تتأكد من استلام المبلغ.</p>
                        </div>
                    {:else if trade.status === 'paid' && isSeller}
                        <div class="bg-accent-green/10 border border-accent-green/30 rounded-md p-3 text-sm">
                            <div class="font-semibold text-accent-green mb-1">أكد المشتري الدفع</div>
                            <p class="text-text-secondary text-xs">تأكد من استلام المبلغ في حسابك، ثم اضغط "إطلاق العملات" لإتمام الصفقة.</p>
                        </div>
                        <button class="w-full btn-success text-sm" on:click={releaseCrypto} disabled={actionLoading}>
                            {actionLoading ? 'جارٍ...' : 'إطلاق العملات'}
                        </button>
                    {:else if trade.status === 'paid' && isBuyer}
                        <div class="bg-accent-blue/10 border border-accent-blue/30 rounded-md p-3 text-sm">
                            <div class="font-semibold text-accent-blue mb-1">تم تأكيد الدفع</div>
                            <p class="text-text-secondary text-xs">بانتظار قيام البائع بإطلاق العملات.</p>
                        </div>
                    {/if}

                    <button class="w-full btn-ghost text-sm text-accent-red hover:bg-accent-red/10" on:click={cancelTrade} disabled={actionLoading}>
                        إلغاء الصفقة
                    </button>
                </div>
            </div>
        {/if}

        <!-- المحادثة -->
        {#if trade.status !== 'completed' && trade.status !== 'cancelled'}
            <div class="card-default">
                <h3 class="text-sm font-semibold text-text-primary uppercase tracking-wider mb-3">محادثة الصفقة</h3>
                <div bind:this={messagesContainer} class="h-64 overflow-y-auto space-y-2 bg-base-900 rounded-md p-3 mb-3">
                    {#if messages.length === 0}
                        <div class="text-center text-text-tertiary text-sm py-8">لا توجد رسائل بعد</div>
                    {:else}
                        {#each messages as m}
                            <div class="flex {m.sender_id === myId ? 'justify-start' : 'justify-end'}">
                                <div class="max-w-[75%] rounded-lg px-3 py-2 text-sm {m.sender_id === myId ? 'bg-accent-blue text-white' : 'bg-base-700 text-text-primary'}">
                                    <div>{m.message}</div>
                                    <div class="text-[10px] opacity-70 mt-0.5">{fmtDate(m.created_at)}</div>
                                </div>
                            </div>
                        {/each}
                    {/if}
                </div>
                <div class="flex gap-2">
                    <input
                        type="text"
                        bind:value={newMessage}
                        on:keydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMessage(); } }}
                        class="input flex-1"
                        placeholder="اكتب رسالتك..."
                        dir="rtl"
                    />
                    <button class="btn-primary text-sm" on:click={sendMessage} disabled={sendingMessage || !newMessage.trim()}>
                        {sendingMessage ? '...' : 'إرسال'}
                    </button>
                </div>
            </div>
        {/if}

        <!-- معلومات الحماية -->
        <div class="card-compact text-xs text-text-tertiary text-center">
            🔒 هذه الصفقة محمية بنظام الضمان (Escrow) — العملات الرقمية محجوزة تلقائياً حتى تأكيد الدفع
        </div>
    {/if}
</div>
