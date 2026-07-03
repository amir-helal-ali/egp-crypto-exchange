import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import type { UserPublic } from './types';

const emptyUser: UserPublic | null = null;
export const adminUser = writable<UserPublic | null>(emptyUser);
export const adminToken = writable<string>('');
export const isAdminAuthenticated = derived(adminUser, ($u) => $u !== null && $u.role === 'admin');

if (browser) {
    const storedToken = localStorage.getItem('admin_access_token');
    const storedUser = localStorage.getItem('admin_user');
    if (storedToken) adminToken.set(storedToken);
    if (storedUser) {
        try { adminUser.set(JSON.parse(storedUser)); } catch { /* ignore */ }
    }

    adminToken.subscribe((v) => {
        if (v) localStorage.setItem('admin_access_token', v);
        else localStorage.removeItem('admin_access_token');
    });
    adminUser.subscribe((v) => {
        if (v) localStorage.setItem('admin_user', JSON.stringify(v));
        else localStorage.removeItem('admin_user');
    });
}

export function setAdminSession(token: string, u: UserPublic) {
    adminToken.set(token);
    adminUser.set(u);
}

export function clearAdminSession() {
    adminToken.set('');
    adminUser.set(null);
    if (browser) {
        localStorage.removeItem('admin_access_token');
        localStorage.removeItem('admin_refresh_token');
        localStorage.removeItem('admin_user');
    }
}
