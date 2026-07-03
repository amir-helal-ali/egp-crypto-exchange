import Decimal from 'decimal.js';

Decimal.set({ precision: 28, rounding: Decimal.ROUND_HALF_UP });

export const fmtNum = (v: string | number | null | undefined, dp = 2): string => {
    if (v === null || v === undefined || v === '') return '—';
    try {
        return new Decimal(v).toFormat(dp);
    } catch {
        return String(v);
    }
};

export const fmtPrice = (v: string | number | null | undefined): string => {
    if (v === null || v === undefined || v === '') return '—';
    try {
        const d = new Decimal(v);
        // 2 dp for EGP, 8 dp for crypto
        if (d.lt(1)) return d.toFormat(8);
        return d.toFormat(2);
    } catch {
        return String(v);
    }
};

export const fmtQty = (v: string | number | null | undefined, dp = 6): string => {
    if (v === null || v === undefined || v === '') return '—';
    try {
        return new Decimal(v).toFormat(dp);
    } catch {
        return String(v);
    }
};

export const fmtEgp = (v: string | number | null | undefined): string => {
    if (v === null || v === undefined || v === '') return '—';
    try {
        return `EGP ${new Decimal(v).toFormat(2)}`;
    } catch {
        return String(v);
    }
};

export const fmtAsset = (v: string | number | null | undefined, symbol: string, dp = 6): string => {
    if (v === null || v === undefined || v === '') return '—';
    try {
        return `${new Decimal(v).toFormat(dp)} ${symbol}`;
    } catch {
        return String(v);
    }
};

export const fmtDate = (iso: string | null | undefined): string => {
    if (!iso) return '—';
    try {
        const d = new Date(iso);
        return d.toLocaleString('en-GB', {
            year: 'numeric', month: 'short', day: '2-digit',
            hour: '2-digit', minute: '2-digit', second: '2-digit',
            hour12: false,
        });
    } catch {
        return String(iso);
    }
};

export const fmtTime = (iso: string | null | undefined): string => {
    if (!iso) return '—';
    try {
        const d = new Date(iso);
        return d.toLocaleTimeString('en-GB', {
            hour: '2-digit', minute: '2-digit', second: '2-digit',
            hour12: false,
        });
    } catch {
        return String(iso);
    }
};

export const addDecimal = (a: string, b: string): string => {
    try { return new Decimal(a).plus(b).toString(); } catch { return '0'; }
};
export const subDecimal = (a: string, b: string): string => {
    try { return new Decimal(a).minus(b).toString(); } catch { return '0'; }
};
export const mulDecimal = (a: string, b: string): string => {
    try { return new Decimal(a).times(b).toString(); } catch { return '0'; }
};

export const divDecimal = (a: string, b: string): string => {
    try {
        if (Number(b) === 0) return '0';
        return new Decimal(a).div(b).toString();
    } catch { return '0'; }
};

export const pairToAssets = (pair: string): { base: string; quote: string } => {
    const [base, quote] = pair.split('_');
    return { base: base || '', quote: quote || '' };
};
