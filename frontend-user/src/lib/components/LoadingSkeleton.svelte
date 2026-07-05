<script lang="ts">
    export let lines = 5;
    export let avatar = false;
    export let columns = 1;
</script>

<div class="animate-pulse space-y-3">
    {#if avatar}
        <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-full bg-base-700"></div>
            <div class="flex-1 space-y-2">
                <div class="h-3 bg-base-700 rounded w-1/4"></div>
                <div class="h-2 bg-base-700 rounded w-1/3"></div>
            </div>
        </div>
    {/if}
    {#if columns > 1}
        <div class="grid gap-3" style="grid-template-columns: repeat({columns}, 1fr);">
            {#each Array(columns) as _, i}
                <div class="space-y-2">
                    {#each Array(3) as _, j}
                        <div class="h-3 bg-base-700 rounded" style="width: {60 + Math.random() * 30}%"></div>
                    {/each}
                </div>
            {/each}
        </div>
    {:else}
        {#each Array(lines) as _, i}
            <div class="h-3 bg-base-700 rounded" style="width: {50 + Math.random() * 45}%"></div>
        {/each}
    {/if}
</div>
