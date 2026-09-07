<script lang="ts">
    import { onMount } from 'svelte'
    import type { Component } from 'svelte'
    import { resolveNativeView } from '$lib/plugins/component-registry'
    import 'mdui/components/circular-progress.js'

    let { token, pluginId }: { token: string; pluginId?: string } = $props()

    let Comp = $state<Component | null>(null)
    let loadError = $state<string | null>(null)

    onMount(async () => {
        const mod = await resolveNativeView(token)
        if (mod) {
            Comp = mod.default
        } else {
            loadError = `未知组件 token: ${token}`
        }
    })
</script>

{#if Comp}
    <Comp />
{:else if loadError}
    <div
        class="flex h-full items-center justify-center p-4 text-center text-sm text-red-500"
    >
        {loadError}
    </div>
{:else}
    <div class="flex h-full items-center justify-center">
        <mdui-circular-progress></mdui-circular-progress>
    </div>
{/if}
