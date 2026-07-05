<script lang="ts">
    import { onMount } from 'svelte';

    export let data: Array<{ label: string; value: number; color: string }> = [];
    export let size = 160;
    export let thickness = 24;
    export let centerLabel = '';
    export let centerValue = '';

    let canvas: HTMLCanvasElement;
    let total = 0;

    $: total = data.reduce((s, d) => s + d.value, 0);

    function render() {
        if (!canvas || total === 0) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;
        const w = canvas.width = size * 2;
        const h = canvas.height = size * 2;
        const cx = w / 2;
        const cy = h / 2;
        const radius = (size - thickness) * 1; // scale for 2x

        ctx.clearRect(0, 0, w, h);

        // خلفية دائرية
        ctx.beginPath();
        ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(36, 48, 71, 0.5)';
        ctx.lineWidth = thickness * 2;
        ctx.stroke();

        let startAngle = -Math.PI / 2;
        for (const d of data) {
            const angle = (d.value / total) * Math.PI * 2;
            ctx.beginPath();
            ctx.arc(cx, cy, radius, startAngle, startAngle + angle);
            ctx.strokeStyle = d.color;
            ctx.lineWidth = thickness * 2;
            ctx.stroke();
            startAngle += angle;
        }
    }

    onMount(() => render());
    $: { data; total; setTimeout(() => render(), 0); }
</script>

<div class="relative inline-block" style="width: {size}px; height: {size}px;">
    <canvas bind:this={canvas} class="w-full h-full"></canvas>
    <div class="absolute inset-0 flex flex-col items-center justify-center text-center">
        {#if centerValue}
            <div class="text-lg font-mono font-bold text-text-primary">{centerValue}</div>
        {/if}
        {#if centerLabel}
            <div class="text-[10px] text-text-tertiary">{centerLabel}</div>
        {/if}
    </div>
</div>
