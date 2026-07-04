import Decimal from 'decimal.js';
Decimal.set({ precision: 28, rounding: Decimal.ROUND_HALF_UP });

export const fmtNum = (v: string | number | null | undefined, dp = 2): string => {
    if (v === null || v === undefined || v === '') return '—';
    try { return new Decimal(v).toFormat(dp); } catch { return String(v); }
};
export const fmtEgp = (v: string | number | null | undefined): string => {
    if (v === null || v === undefined || v === '') return '—';
    try { return `EGP ${new Decimal(v).toFormat(2)}`; } catch { return String(v); }
};
export const fmtAsset = (v: string | number | null | undefined, symbol: string, dp = 6): string => {
    if (v === null || v === undefined || v === '') return '—';
    try { return `${new Decimal(v).toFormat(dp)} ${symbol}`; } catch { return String(v); }
};
export const fmtQty = (v: string | number | null | undefined, dp = 6): string => {
    if (v === null || v === undefined || v === '') return '—';
    try { return new Decimal(v).toFormat(dp); } catch { return String(v); }
};
export const fmtDate = (iso: string | null | undefined): string => {
    if (!iso) return '—';
    try {
        return new Date(iso).toLocaleString('en-GB', {
            year: 'numeric', month: 'short', day: '2-digit',
            hour: '2-digit', minute: '2-digit', second: '2-digit',
            hour12: false,
        });
    } catch { return String(iso); }
};
export const fmtRelative = (iso: string | null | undefined): string => {
    if (!iso) return '—';
    const d = new Date(iso).getTime();
    const now = Date.now();
    const sec = Math.floor((now - d) / 1000);
    if (sec < 60) return `${sec}s ago`;
    if (sec < 3600) return `${Math.floor(sec / 60)}m ago`;
    if (sec < 86400) return `${Math.floor(sec / 3600)}h ago`;
    return `${Math.floor(sec / 86400)}d ago`;
};
