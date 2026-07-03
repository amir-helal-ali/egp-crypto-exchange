// مركز الترجمة - i18n store
// Translation center using Svelte stores

import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import { ar } from './ar';

export type Lang = 'ar' | 'en';

// العربية هي اللغة الأساسية - Arabic is the primary language
export const lang = writable<Lang>('ar');

if (browser) {
    const stored = localStorage.getItem('lang') as Lang | null;
    if (stored === 'ar' || stored === 'en') {
        lang.set(stored);
    } else {
        lang.set('ar'); // الافتراضي عربي
    }
    lang.subscribe((v) => {
        localStorage.setItem('lang', v);
        document.documentElement.lang = v;
        document.documentElement.dir = v === 'ar' ? 'rtl' : 'ltr';
    });
}

/**
 * ترجمة المفتاح المحدد
 * Translate a key
 */
export function t(key: keyof typeof ar, params?: Record<string, string | number>): string {
    let value: string = ar[key] ?? String(key);
    if (params) {
        for (const [k, v] of Object.entries(params)) {
            value = value.replace(new RegExp(`\\{${k}\\}`, 'g'), String(v));
        }
    }
    return value;
}

/**
 * متجر ترجمة متجاوب
 * Reactive translation store
 */
export const _t = derived(lang, ($lang) => {
    return (key: keyof typeof ar, params?: Record<string, string | number>) => {
        let value: string = ar[key] ?? String(key);
        if (params) {
            for (const [k, v] of Object.entries(params)) {
                value = value.replace(new RegExp(`\\{${k}\\}`, 'g'), String(v));
            }
        }
        return value;
    };
});

export function setLang(l: Lang) {
    lang.set(l);
}

export function getLang(): Lang {
    return get(lang);
}

export function isRTL(): boolean {
    return get(lang) === 'ar';
}
