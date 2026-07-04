<script lang="ts">
    import { notifications, dismissNotification } from '$lib/stores';
    import { fly } from 'svelte/transition';

    function iconFor(type: string): string {
        switch (type) {
            case 'success': return 'M5 13l4 4L19 7';
            case 'error': return 'M6 18L18 6M6 6l12 12';
            case 'warning': return 'M12 9v2m0 4h.01M5.07 19h13.86c1.54 0 2.5-1.67 1.73-3L13.73 4a2 2 0 00-3.46 0L3.34 16c-.77 1.33.19 3 1.73 3z';
            case 'info': return 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z';
            default: return '';
        }
    }
    function colorFor(type: string): string {
        switch (type) {
            case 'success': return 'text-accent-green border-accent-green/30 bg-accent-green/10';
            case 'error': return 'text-accent-red border-accent-red/30 bg-accent-red/10';
            case 'warning': return 'text-accent-yellow border-accent-yellow/30 bg-accent-yellow/10';
            case 'info': return 'text-accent-blue border-accent-blue/30 bg-accent-blue/10';
            default: return 'text-text-secondary border-base-600 bg-base-700';
        }
    }
</script>

<div class="fixed bottom-4 left-4 z-50 space-y-2 max-w-sm">
    {#each $notifications as n (n.id)}
        <div
            in:fly={{ y: 20, duration: 200 }}
            out:fly={{ y: 20, duration: 150 }}
            class="card-default {colorFor(n.type)} flex items-start gap-3 cursor-pointer"
            on:click={() => dismissNotification(n.id)}
        >
            <svg class="w-5 h-5 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d={iconFor(n.type)} />
            </svg>
            <div class="flex-1 min-w-0">
                <div class="font-semibold text-sm">{n.title}</div>
                <div class="text-xs text-text-secondary mt-0.5">{n.message}</div>
            </div>
            <button class="text-text-tertiary hover:text-text-primary" on:click|stopPropagation={() => dismissNotification(n.id)}>
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>
        </div>
    {/each}
</div>
