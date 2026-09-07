<script lang="ts">
    import Heading from '$lib/components/base/Heading.svelte'
    import { page } from '$app/state'
    import { resolve } from '$app/paths'
    import { pluginState } from '$lib/state/plugins.svelte'
    import { hostBridge } from '$lib/plugins/host-bridge'
    import { invoke } from '@tauri-apps/api/core'

    interface PluginViewResolution {
        pluginId: string
        viewId: string
        title: string
        entryUrl: string
    }

    const slug = $derived(page.params.slug ?? '')

    const manifest = $derived(
        pluginState.manifests.find(m => m.route === `/plugins/view/${slug}`),
    )

    const resolved = $derived(pluginState.getViewForSlug(slug))

    const hasUIPermission = $derived(
        resolved?.manifest.permissions.includes('pluginUI') ?? false,
    )

    let iframeEl: HTMLIFrameElement | undefined = $state()
    let entryUrl: string | null = $state(null)

    // 解析插件 view 入口地址（仅在具备 UI 权限时）
    $effect(() => {
        if (!resolved || !hasUIPermission) return
        const { manifest, view } = resolved
        let active = true

        async function load() {
            try {
                const result = await invoke<PluginViewResolution | null>(
                    'get_plugin_view',
                    { pluginId: manifest.id, viewId: view.id },
                )
                if (active && result) entryUrl = result.entryUrl
            } catch (err) {
                console.error('[plugin-view] 解析 view 入口失败:', err)
            }
        }
        load()

        return () => {
            active = false
            entryUrl = null
        }
    })

    // iframe 挂载且 entryUrl 就绪后绑定宿主桥，卸载时自动解绑
    $effect(() => {
        if (!iframeEl || !resolved || !entryUrl) return
        const { manifest } = resolved
        return hostBridge.attach(iframeEl, manifest.id)
    })
</script>

<svelte:head>
    <title>{manifest?.displayName ?? 'Plugin'} — Plugins</title>
</svelte:head>

<section class="flex h-full min-h-0 flex-col gap-6 overflow-auto pb-10">
    {#if !manifest}
        <div class="flex flex-1 flex-col items-center justify-center gap-4">
            <p class="text-sm text-[rgb(var(--mdui-color-on-surface-variant))]">
                未找到插件 "{slug}"
            </p>
            <a
                href={resolve('/plugins')}
                class="text-sm text-[rgb(var(--mdui-color-primary))] underline"
            >
                返回插件列表
            </a>
        </div>
    {:else}
        <div class="flex items-center gap-4">
            <a
                href={resolve('/plugins')}
                class="flex h-10 w-10 items-center justify-center rounded-full hover:bg-[rgb(var(--mdui-color-surface-container))]"
                title="plugins"
            >
                <mdui-icon name="arrow_back"></mdui-icon>
            </a>
            <Heading
                eyebrow="Plugin"
                title={manifest.displayName}
            />
        </div>

        {#if !resolved}
            <!-- 兜底：插件未声明 view，保留元信息卡片 -->
            <div class="grid gap-4">
                <div class="rounded-2xl bg-[rgb(var(--mdui-color-surface-container))] p-5">
                    <div class="grid gap-2 text-sm">
                        <div class="flex justify-between">
                            <span class="text-[rgb(var(--mdui-color-on-surface-variant))]">ID</span>
                            <span class="font-mono">{manifest.id}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-[rgb(var(--mdui-color-on-surface-variant))]">来源</span>
                            <span>{manifest.source}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-[rgb(var(--mdui-color-on-surface-variant))]">版本</span>
                            <span>{manifest.version}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-[rgb(var(--mdui-color-on-surface-variant))]">路由</span>
                            <span class="font-mono">{manifest.route}</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-[rgb(var(--mdui-color-on-surface-variant))]">作者</span>
                            <span>{manifest.author}</span>
                        </div>
                        <div class="text-[rgb(var(--mdui-color-on-surface-variant))]">
                            {manifest.description}
                        </div>
                    </div>
                </div>
            </div>
        {:else if !hasUIPermission}
            <div class="flex flex-1 flex-col items-center justify-center gap-4">
                <p class="text-sm text-[rgb(var(--mdui-color-on-surface-variant))]">
                    插件未声明 UI 权限
                </p>
            </div>
        {:else}
            <div class="min-h-0 flex-1">
                <iframe
                    src={entryUrl ?? 'about:blank'}
                    sandbox="allow-scripts"
                    title={resolved.view.title}
                    bind:this={iframeEl}
                    class="h-full w-full border-0"
                ></iframe>
            </div>
        {/if}
    {/if}
</section>
