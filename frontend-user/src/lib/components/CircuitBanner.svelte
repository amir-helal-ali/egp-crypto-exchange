<script lang="ts">
    import { circuitOpen } from '$lib/stores';

    let open = false;
    $: open = $circuitOpen;
</script>

{#if open}
    <div class="bg-accent-red/15 border border-accent-red/40 text-accent-red px-4 py-2.5 rounded-md flex items-center gap-3 text-sm animate-slide-in">
        <svg class="w-5 h-5 shrink-0 rtl-flip" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 9v2m0 4h.01M5.07 19h13.86c1.54 0 2.5-1.67 1.73-3L13.73 4a2 2 0 00-3.46 0L3.34 16c-.77 1.33.19 3 1.73 3z" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <div class="flex-1">
            <span class="font-semibold">التداول متوقف.</span>
            <span class="text-accent-red/80">انقطع تدفق أسعار بينانس. تم إيقاف محرك المطابقة لحماية المستخدمين. سيتم رفض الأوامر الجديدة حتى يستعيد التدفق عمله.</span>
        </div>
    </div>
{/if}
