import { browser } from '$app/environment';
import { user, accessToken, setSession } from '$lib/stores';

export const load = async () => {
    if (browser) {
        const storedToken = localStorage.getItem('access_token');
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
