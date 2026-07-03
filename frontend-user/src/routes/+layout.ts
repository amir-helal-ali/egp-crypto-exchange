<script lang="ts">
    import { redirect } from '@sveltejs/kit';
    import type { LayoutLoad } from './$types';
    import { browser } from '$app/environment';
    import { user, accessToken, setSession } from '$lib/stores';

    export const load: LayoutLoad = async () => {
        if (browser) {
            const storedToken = localStorage.getItem('access_token');
            const storedRefresh = localStorage.getItem('refresh_token');
            const storedUser = localStorage.getItem('user');
            if (storedToken && storedUser) {
                try {
                    const u = JSON.parse(storedUser);
                    setSession(storedToken, u);
                } catch {
                    /* ignore */
                }
            }
        }
        return {};
    };
