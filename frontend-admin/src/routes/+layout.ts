<script lang="ts">
    import { browser } from '$app/environment';
    import { adminUser, adminToken, setAdminSession } from '$lib/stores';

    export const load = async () => {
        if (browser) {
            const storedToken = localStorage.getItem('admin_access_token');
            const storedUser = localStorage.getItem('admin_user');
            if (storedToken && storedUser) {
                try {
                    const u = JSON.parse(storedUser);
                    setAdminSession(storedToken, u);
                } catch { /* ignore */ }
            }
        }
        return {};
    };
