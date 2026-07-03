<script lang="ts">
    import { onMount } from 'svelte';
    import { audit } from '$lib/api';
    import { fmtDate, fmtRelative } from '$lib/format';

    let items: any[] = [];
    let loading = true;
    let error = '';

    async function load() {
        try {
            const res = await audit.list(200, 0);
            items = res.items || [];
        } catch (e: any) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    onMount(load);
</script>

<svelte:head><title>Audit Log · Admin</title></svelte:head>

<div class="space-y-6">
    <div>
        <h1 class="text-2xl font-bold text-text-primary">Admin Audit Log</h1>
        <p class="text-sm text-text-secondary mt-1">Immutable record of all admin actions</p>
    </div>

    {#if loading}
        <div class="text-center py-8 text-text-tertiary">Loading...</div>
    {:else if error}
        <div class="text-xs text-accent-red bg-accent-red/10 border border-accent-red/30 rounded px-3 py-2">{error}</div>
    {:else}
        <div class="card-default overflow-x-auto">
            <table class="table-pro">
                <thead>
                    <tr>
                        <th>#</th>
                        <th>Admin</th>
                        <th>Action</th>
                        <th>Target</th>
                        <th>Details</th>
                        <th class="text-right">Time</th>
                    </tr>
                </thead>
                <tbody>
                    {#each items as a}
                        <tr>
                            <td class="num-cell text-text-tertiary text-xs">{a.id}</td>
                            <td class="num-cell text-text-tertiary text-xs">{a.admin_id.slice(0, 8)}…</td>
                            <td class="font-medium">{a.action}</td>
                            <td class="text-text-secondary">{a.target_type || '—'} {a.target_id ? `(${a.target_id.slice(0, 8)}…)` : ''}</td>
                            <td class="text-xs text-text-tertiary num-cell max-w-[400px] truncate" title={JSON.stringify(a.details)}>{JSON.stringify(a.details)}</td>
                            <td class="num-cell text-text-tertiary text-xs" title={fmtDate(a.created_at)}>{fmtRelative(a.created_at)}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
