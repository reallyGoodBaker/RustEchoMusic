<script lang="ts">
    import Heading from '$lib/components/base/Heading.svelte'
    import { pluginState } from '$lib/state/plugins.svelte'
    import 'mdui/components/circular-progress.js'
    import { onMount } from 'svelte'
    import { goto } from '$app/navigation'
    import { base } from '$app/paths'

    onMount(() => {
        void pluginState.loadManifests()
        void pluginState.loadSidebarExtensions()
    })

    function handlePluginClick(route: string) {
        if (route) void goto(`${base}${route}`)
    }
</script>

<svelte:head>
    <title>插件</title>
</svelte:head>

<section class="flex h-full min-h-0 flex-col gap-6 overflow-auto pb-10">
    <Heading eyebrow="Plugins" title="插件" />

    <a
        href="{base}/plugins/kernel"
        class="self-start rounded-full bg-[rgb(var(--mdui-color-secondary-container))] px-3 py-1 text-xs text-[rgb(var(--mdui-color-on-secondary-container))] transition-colors hover:bg-[rgb(var(--mdui-color-secondary-container-high))]"
    >
        查看插件内核 / 贡献点 →
    </a>

    <div
        class="flex flex-1 items-center justify-center"
        class:hidden={!pluginState.isLoading}
    >
        <mdui-circular-progress></mdui-circular-progress>
    </div>

    <div class="grid gap-4" class:hidden={pluginState.isLoading}>
        {#if pluginState.manifests.length === 0}
            <div
                class="rounded-2xl border border-dashed border-[rgb(var(--mdui-color-outline-variant))] p-8 text-center text-sm text-[rgb(var(--mdui-color-on-surface-variant))]"
            >
                暂无已注册的插件。
            </div>
        {:else}
            {#each pluginState.manifests as manifest (manifest.id)}
                {@const ext = pluginState.sidebarExtensions.find(e => e.pluginId === manifest.id)}
                {@const state = ext?.state ?? 'Enabled'}
                <button
                    type="button"
                    class="flex items-center gap-4 rounded-2xl bg-[rgb(var(--mdui-color-surface-container))] p-5 text-left transition-colors hover:bg-[rgb(var(--mdui-color-surface-container-high))]"
                    onclick={() => handlePluginClick(manifest.route)}
                    onkeydown={(e: KeyboardEvent) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                            handlePluginClick(manifest.route)
                        }
                    }}
                >
                    <div
                        class="flex h-12 w-12 items-center justify-center rounded-xl bg-[rgb(var(--mdui-color-primary-container))] text-[rgb(var(--mdui-color-on-primary-container))]"
                    >
                        <mdui-icon name="extension--rounded"></mdui-icon>
                    </div>
                    <div class="flex flex-1 flex-col gap-1">
                        <div class="text-sm font-medium">
                            {manifest.displayName}
                        </div>
                        <div
                            class="text-xs text-[rgb(var(--mdui-color-on-surface-variant))]"
                        >
                            {manifest.id}
                        </div>
                    </div>
                    <div
                        class="rounded-full px-2 py-0.5 text-xs font-medium"
                        class:bg-green-100={state === 'Enabled'}
                        class:text-green-700={state === 'Enabled'}
                        class:bg-gray-100={state === 'Disabled'}
                        class:text-gray-500={state === 'Disabled'}
                    >
                        {state === 'Enabled' ? '已启用' : '已禁用'}
                    </div>
                </button>
            {/each}
        {/if}

        {#if pluginState.error}
            <div class="text-sm text-red-500">
                {pluginState.error}
            </div>
        {/if}
    </div>
</section>
