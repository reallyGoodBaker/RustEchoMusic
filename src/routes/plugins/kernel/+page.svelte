<script lang="ts">
    import Heading from '$lib/components/base/Heading.svelte'
    import { pluginState } from '$lib/state/plugins.svelte'
    import 'mdui/components/circular-progress.js'
    import { onMount } from 'svelte'

    onMount(() => {
        void pluginState.loadKernel()
    })

    let byPoint = $derived(
        pluginState.contributions.reduce<Record<string, typeof pluginState.contributions>>(
            (acc, c) => {
                ;(acc[c.point] ??= []).push(c)
                return acc
            },
            {},
        ),
    )

    let points = $derived(Object.keys(byPoint).sort())
</script>

<svelte:head>
    <title>插件内核</title>
</svelte:head>

<section class="flex h-full min-h-0 flex-col gap-6 overflow-auto pb-10">
    <Heading eyebrow="Plugins · Kernel" title="插件内核与贡献点" />

    <div class="flex flex-1 items-center justify-center" class:hidden={!pluginState.kernelLoading}>
        <mdui-circular-progress></mdui-circular-progress>
    </div>

    {#if pluginState.error}
        <div class="text-sm text-red-500">{pluginState.error}</div>
    {/if}

    {#if pluginState.kernel}
        <div class="rounded-2xl border border-[rgb(var(--mdui-color-outline-variant))] p-4 text-xs text-[rgb(var(--mdui-color-on-surface-variant))]">
            宿主版本 {pluginState.kernel.hostVersion}
        </div>

        <div>
            <h3 class="mb-3 text-sm font-semibold">已安装插件（{pluginState.kernel.plugins.length}）</h3>
            <div class="grid gap-3">
                {#each pluginState.kernel.plugins as plugin (plugin.id)}
                    <div class="flex items-center gap-4 rounded-2xl bg-[rgb(var(--mdui-color-surface-container))] p-4">
                        <div class="flex flex-1 flex-col gap-1">
                            <div class="flex items-center gap-2 text-sm font-medium">
                                {plugin.id}
                                <span class="rounded-full bg-[rgb(var(--mdui-color-secondary-container))] px-2 py-0.5 text-xs text-[rgb(var(--mdui-color-on-secondary-container))]">{plugin.tier}</span>
                                {#if plugin.active}
                                    <span class="rounded-full bg-green-100 px-2 py-0.5 text-xs text-green-700">active</span>
                                {/if}
                            </div>
                            <div class="text-xs text-[rgb(var(--mdui-color-on-surface-variant))]">
                                v{plugin.version} · {plugin.source} · 状态 {plugin.state} · 健康 {plugin.health}
                            </div>
                            {#if plugin.lastError}
                                <div class="text-xs text-red-500">错误：{plugin.lastError}</div>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        </div>

        <div>
            <h3 class="mb-3 text-sm font-semibold">贡献点（{pluginState.contributions.length}）</h3>
            {#if points.length === 0}
                <div class="rounded-2xl border border-dashed border-[rgb(var(--mdui-color-outline-variant))] p-6 text-center text-sm text-[rgb(var(--mdui-color-on-surface-variant))]">
                    暂无贡献点。
                </div>
            {:else}
                <div class="grid gap-4">
                    {#each points as point (point)}
                        <div class="rounded-2xl bg-[rgb(var(--mdui-color-surface-container-low))] p-4">
                            <div class="mb-2 text-xs font-mono text-[rgb(var(--mdui-color-primary))]">{point}</div>
                            <div class="grid gap-2">
                                {#each byPoint[point] as contribution (contribution.plugin + contribution.key)}
                                    <div class="rounded-xl bg-[rgb(var(--mdui-color-surface))] p-3">
                                        <div class="mb-1 flex items-center gap-2 text-xs">
                                            <span class="font-mono">{contribution.plugin}</span>
                                            <span class="text-[rgb(var(--mdui-color-on-surface-variant))]">{contribution.key}</span>
                                            <span class="rounded-full bg-[rgb(var(--mdui-color-tertiary-container))] px-2 py-0.5 text-[rgb(var(--mdui-color-on-tertiary-container))]">{contribution.payload.kind ?? 'extension'}</span>
                                        </div>
                                        <pre class="overflow-auto rounded-lg bg-[rgb(var(--mdui-color-surface-container-highest))] p-2 text-[11px] text-[rgb(var(--mdui-color-on-surface-variant))]">{JSON.stringify(contribution.payload, null, 2)}</pre>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>

        <div>
            <h3 class="mb-3 text-sm font-semibold">发现问题（{pluginState.kernel.issues.length}）</h3>
            {#if pluginState.kernel.issues.length === 0}
                <div class="text-sm text-[rgb(var(--mdui-color-on-surface-variant))]">无。</div>
            {:else}
                <ul class="list-disc space-y-1 pl-5 text-xs text-[rgb(var(--mdui-color-on-surface-variant))]">
                    {#each pluginState.kernel.issues as issue (issue)}
                        <li>{issue}</li>
                    {/each}
                </ul>
            {/if}
        </div>
    {/if}
</section>
