<script lang="ts">
    import { onMount } from 'svelte';

    export let data: Array<{ label: string; value: number; color?: string }> = [];
    export let height = 200;
    export let showLabels = true;

    let canvas: HTMLCanvasElement;
    let maxValue = 0;

    $: maxValue = Math.max(...data.map((d) => d.value), 1);

    function render() {
        if (!canvas || data.length === 0) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;
        const dpr = window.devicePixelRatio || 1;
        const w = canvas.offsetWidth;
        const h = height;
        canvas.width = w * dpr;
        canvas.height = h * dpr;
        canvas.style.height = h + 'px';
        ctx.scale(dpr, dpr);
        ctx.clearRect(0, 0, w, h);

        const barWidth = (w - 20) / data.length;
        const barInnerWidth = barWidth * 0.6;
        const offset = (barWidth - barInnerWidth) / 2;

        // خط أفقي
        ctx.strokeStyle = 'rgba(36, 48, 71, 0.5)';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(10, h - 30);
        ctx.lineTo(w - 10, h - 30);
        ctx.stroke();

        data.forEach((d, i) => {
            const x = 10 + i * barWidth + offset;
            const barHeight = (d.value / maxValue) * (h - 50);
            const y = h - 30 - barHeight;
            const color = d.color || (i % 2 === 0 ? '#3b82f6' : '#00d68f');

            // ظل
            ctx.fillStyle = color + '20';
            ctx.fillRect(x, y, barInnerWidth, barHeight);

            // الشريط
            ctx.fillStyle = color;
            ctx.fillRect(x, y, barInnerWidth, barHeight);

            if (showLabels) {
                ctx.fillStyle = '#5a647a';
                ctx.font = '10px JetBrains Mono';
                ctx.textAlign = 'center';
                ctx.fillText(d.label, x + barInnerWidth / 2, h - 15);
            }
        });
    }

    onMount(() => {
        render();
        const ro = new ResizeObserver(() => render());
        if (canvas) ro.observe(canvas);
    });
    $: { data; setTimeout(() => render(), 0); }
</script>

<div class="w-full">
    <canvas bind:this={canvas} class="w-full"></canvas>
</div>
