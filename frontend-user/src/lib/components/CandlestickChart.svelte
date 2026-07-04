<script lang="ts">
    import { onMount, onDestroy, tick } from 'svelte';
    import { tickers, recentTrades, circuitOpen } from '$lib/stores';
    import { fmtEgp, fmtPrice, fmtQty } from '$lib/format';
    import {
        createChart, ColorType, CrosshairMode, LineStyle,
        type IChartApi, type ISeriesApi, type UTCTimestamp,
    } from 'lightweight-charts';

    export let pair: string;
    let chartContainer: HTMLDivElement;
    let intervalLabel: HTMLDivElement;
    let chart: IChartApi | null = null;
    let candleSeries: ISeriesApi<'Candlestick'> | null = null;
    let volumeSeries: ISeriesApi<'Histogram'> | null = null;
    let currentInterval: '1m' | '5m' | '15m' | '1h' | '4h' | '1d' = '15m';
    let intervalMs = 15 * 60 * 1000;

    let lastPrice = '';
    let prevPrice = '';
    let direction: 'up' | 'down' | 'flat' = 'flat';
    let priceChange24h = '';
    let candles: Map<number, { time: number; open: number; high: number; low: number; close: number; volume: number }> = new Map();

    // اشتراك في تحديثات السعر اللحظية
    const unsubTicker = tickers.subscribe((map) => {
        const t = map[pair];
        if (!t) return;
        prevPrice = lastPrice;
        lastPrice = t.derived_egp_price;
        if (prevPrice) {
            const diff = Number(lastPrice) - Number(prevPrice);
            if (diff > 0) direction = 'up';
            else if (diff < 0) direction = 'down';
        }
        updateLastCandle();
    });

    function updateLastCandle() {
        if (!lastPrice || !candleSeries) return;
        const now = Math.floor(Date.now() / intervalMs) * intervalMs;
        const price = Number(lastPrice);
        let candle = candles.get(now);
        if (!candle) {
            candle = { time: now / 1000 as UTCTimestamp, open: price, high: price, low: price, close: price, volume: 0 };
            candles.set(now, candle);
        } else {
            candle.close = price;
            candle.high = Math.max(candle.high, price);
            candle.low = Math.min(candle.low, price);
        }
        candleSeries.update({
            time: candle.time as UTCTimestamp,
            open: candle.open, high: candle.high, low: candle.low, close: candle.close,
        });
    }

    function setInterval(iv: typeof currentInterval) {
        currentInterval = iv;
        const map: Record<typeof currentInterval, number> = {
            '1m': 60 * 1000,
            '5m': 5 * 60 * 1000,
            '15m': 15 * 60 * 1000,
            '1h': 60 * 60 * 1000,
            '4h': 4 * 60 * 60 * 1000,
            '1d': 24 * 60 * 60 * 1000,
        };
        intervalMs = map[iv];
        candles.clear();
        if (candleSeries) {
            candleSeries.setData([]);
        }
        // أعد بناء الشموع من الصفقات الأخيرة
        rebuildCandlesFromTrades();
    }

    function rebuildCandlesFromTrades() {
        const trades = (recentTrades as any).subscribe ? null : null;
        // نعتمد على تدفق الصفقات اللحظي
    }

    // إضافة الصفقات الجديدة إلى الشموع
    const unsubTrades = recentTrades.subscribe((map) => {
        const trades = map[pair];
        if (!trades || !candleSeries) return;
        for (const t of trades.slice(0, 200).reverse()) {
            const ts = new Date(t.executed_at).getTime();
            const bucket = Math.floor(ts / intervalMs) * intervalMs;
            const price = Number(t.price);
            const qty = Number(t.quantity);
            let candle = candles.get(bucket);
            if (!candle) {
                candle = { time: bucket / 1000 as UTCTimestamp, open: price, high: price, low: price, close: price, volume: 0 };
                candles.set(bucket, candle);
            } else {
                candle.close = price;
                candle.high = Math.max(candle.high, price);
                candle.low = Math.min(candle.low, price);
                candle.volume += qty;
            }
        }
        // إعادة ترتيب الشموع زمنياً
        const sorted = Array.from(candles.values()).sort((a, b) => a.time - b.time);
        candleSeries.setData(sorted.map((c) => ({
            time: c.time as UTCTimestamp,
            open: c.open, high: c.high, low: c.low, close: c.close,
        })));
        if (volumeSeries) {
            volumeSeries.setData(sorted.map((c) => ({
                time: c.time as UTCTimestamp,
                value: c.volume,
                color: c.close >= c.open ? 'rgba(0, 214, 143, 0.4)' : 'rgba(255, 82, 82, 0.4)',
            })));
        }
    });

    function initChart() {
        if (!chartContainer) return;
        chart = createChart(chartContainer, {
            layout: {
                background: { type: ColorType.Solid, color: '#0f1623' },
                textColor: '#8b94a8',
                fontFamily: 'JetBrains Mono, monospace',
                fontSize: 11,
            },
            grid: {
                vertLines: { color: 'rgba(36, 48, 71, 0.5)', style: LineStyle.Solid },
                horzLines: { color: 'rgba(36, 48, 71, 0.5)', style: LineStyle.Solid },
            },
            crosshair: {
                mode: CrosshairMode.Normal,
                vertLine: { color: '#3b82f6', width: 1, style: LineStyle.Dashed, labelBackgroundColor: '#1d2738' },
                horzLine: { color: '#3b82f6', width: 1, style: LineStyle.Dashed, labelBackgroundColor: '#1d2738' },
            },
            rightPriceScale: {
                borderColor: '#1d2738',
                scaleMargins: { top: 0.05, bottom: 0.25 },
            },
            timeScale: {
                borderColor: '#1d2738',
                timeVisible: true,
                secondsVisible: false,
                rightOffset: 5,
            },
            handleScroll: true,
            handleScale: true,
        });

        candleSeries = chart.addCandlestickSeries({
            upColor: '#00d68f',
            downColor: '#ff5252',
            borderUpColor: '#00d68f',
            borderDownColor: '#ff5252',
            wickUpColor: '#00d68f',
            wickDownColor: '#ff5252',
            priceLineColor: '#3b82f6',
            priceLineStyle: LineStyle.Dotted,
        });

        volumeSeries = chart.addHistogramSeries({
            priceFormat: { type: 'volume' },
            priceScaleId: 'volume',
        });
        chart.priceScale('volume').applyOptions({
            scaleMargins: { top: 0.8, bottom: 0 },
        });

        // ضبط حجم الرسم
        const resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                if (chart) {
                    chart.applyOptions({
                        width: entry.contentRect.width,
                        height: entry.contentRect.height,
                    });
                }
            }
        });
        resizeObserver.observe(chartContainer);
    }

    onMount(async () => {
        await tick();
        initChart();
    });

    onDestroy(() => {
        unsubTicker();
        unsubTrades();
        if (chart) {
            chart.remove();
            chart = null;
        }
    });

    const intervals: Array<typeof currentInterval> = ['1m', '5m', '15m', '1h', '4h', '1d'];
</script>

<div class="card-default">
    <!-- الترويسة مع السعر والإطارات الزمنية -->
    <div class="flex items-center justify-between mb-3 flex-wrap gap-2">
        <div class="flex items-center gap-3">
            <div>
                <h3 class="text-base font-bold text-text-primary">{pair.replace('_', '/')}</h3>
                <div class="text-xs text-text-tertiary">السعر المباشر · مستمد من بينانس</div>
            </div>
            <div class="text-right">
                <div class="text-2xl font-mono font-bold leading-none {direction === 'up' ? 'text-accent-green' : direction === 'down' ? 'text-accent-red' : 'text-text-primary'}">
                    {fmtEgp(lastPrice || '0')}
                </div>
                {#if direction === 'up'}
                    <div class="text-xs text-accent-green flex items-center gap-1 mt-1">
                        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20"><path d="M5 15l5-5 5 5H5z" /></svg>
                        صاعد
                    </div>
                {:else if direction === 'down'}
                    <div class="text-xs text-accent-red flex items-center gap-1 mt-1">
                        <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20"><path d="M5 5l5 5 5-5H5z" /></svg>
                        هابط
                    </div>
                {/if}
            </div>
        </div>

        <!-- الإطارات الزمنية -->
        <div class="flex items-center gap-1 bg-base-900 border border-base-600 rounded-md p-0.5" bind:this={intervalLabel}>
            {#each intervals as iv}
                <button
                    class="px-2.5 py-1 text-xs font-medium rounded transition-colors {currentInterval === iv ? 'bg-base-600 text-text-primary' : 'text-text-tertiary hover:text-text-secondary'}"
                    on:click={() => setInterval(iv)}>
                    {iv}
                </button>
            {/each}
        </div>
    </div>

    <!-- الرسم البياني -->
    <div class="h-[420px] relative">
        <div bind:this={chartContainer} class="w-full h-full"></div>
        {#if $circuitOpen}
            <div class="absolute inset-0 bg-base-900/70 flex items-center justify-center text-accent-red text-sm font-semibold backdrop-blur-sm">
                <div class="text-center">
                    <svg class="w-8 h-8 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01M5.07 19h13.86c1.54 0 2.5-1.67 1.73-3L13.73 4a2 2 0 00-3.46 0L3.34 16c-.77 1.33.19 3 1.73 3z" />
                    </svg>
                    تدفق الأسعار متوقف
                </div>
            </div>
        {/if}
    </div>
</div>
