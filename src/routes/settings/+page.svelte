<script lang="ts">
    import Button from '$lib/components/base/Button.svelte'
    import Heading from '$lib/components/base/Heading.svelte'
    import Select from '$lib/components/base/Select.svelte'
    import SettingsListItem from '$lib/features/settings/SettingsListItem.svelte'
    import SettingsRow from '$lib/features/settings/SettingsRow.svelte'
    import SettingsSection from '$lib/features/settings/SettingsSection.svelte'
    import { settings } from '$lib/state/settings.svelte'
    import type { ThemeMode, PluginLogLevel } from '$lib/types'
    import { scanDirectory, removeMusicDirectory } from '$lib/utils/library'

    import { setTheme } from '@tauri-apps/api/app'
    import 'mdui/components/circular-progress.js'
    import 'mdui/components/switch.js'
    import { setTheme as setMduiTheme } from 'mdui/functions/setTheme.js'

    const themeOptions: { label: string; value: ThemeMode }[] = [
        { label: '跟随系统', value: 'auto' },
        { label: '浅色模式', value: 'light' },
        { label: '深色模式', value: 'dark' },
    ]

    const logLevelOptions: { label: string; value: PluginLogLevel }[] = [
        { label: '关闭', value: 'off' },
        { label: '错误', value: 'error' },
        { label: '警告', value: 'warn' },
        { label: '信息', value: 'info' },
        { label: '调试', value: 'debug' },
    ]

    function handleSwitchChange(
        event: Event,
        key: 'scanOnStartup' | 'reduceMotion' | 'pluginScanOnStartup' | 'pluginDevMode',
    ) {
        const target = event.currentTarget as HTMLElement & {
            checked: boolean
        }

        settings.update({
            [key]: target.checked,
        })
    }

    export async function setThemeMode(theme: ThemeMode) {
        settings.update({
            theme,
        })

        await applyTheme(theme)
    }

    export async function applyTheme(theme: ThemeMode) {
        setMduiTheme(theme)

        await setTheme(theme === 'auto' ? null : theme)
    }
</script>

<svelte:head>
    <title>设置</title>
</svelte:head>

<section class="flex h-full min-h-0 flex-col gap-6 overflow-auto pb-10">
    <Heading eyebrow="Settings" title="设置" />

    <div
        class="flex flex-1 items-center justify-center"
        class:hidden={!settings.isLoading}
    >
        <mdui-circular-progress></mdui-circular-progress>
    </div>

    <div class="grid gap-6" class:hidden={settings.isLoading}>
        <SettingsSection title="外观">
            <SettingsRow title="主题" description="设置应用的明暗模式">
                <div class="w-48">
                    <Select
                        value={settings.data?.theme}
                        options={themeOptions}
                        onchange={value => setThemeMode(value as ThemeMode)}
                    />
                </div>
            </SettingsRow>

            <SettingsRow title="减少动画" description="降低界面动画和过渡效果">
                <mdui-switch
                    checked={settings.data?.reduceMotion}
                    onchange={(event: Event) =>
                        handleSwitchChange(event, 'reduceMotion')}
                ></mdui-switch>
            </SettingsRow>
        </SettingsSection>

        <SettingsSection title="媒体库">
            <SettingsRow
                title="启动时扫描"
                description="应用启动时自动刷新媒体库"
            >
                <mdui-switch
                    checked={settings.data?.scanOnStartup}
                    onchange={(event: Event) =>
                        handleSwitchChange(event, 'scanOnStartup')}
                ></mdui-switch>
            </SettingsRow>
            <div class="grid gap-3">
                <div>
                    <div class="text-sm font-medium">媒体库目录</div>
                    <div
                        class="text-xs text-[rgb(var(--mdui-color-on-surface-variant))]"
                    >
                        管理应用扫描音乐文件的本地文件夹
                    </div>
                </div>

                <div class="flex">
                    <Button variant="tonal" onclick={scanDirectory}>
                        添加本地文件夹
                    </Button>
                </div>

                {#if settings.data?.libraryDirs?.length > 0}
                    <div class="grid gap-2 mt-2">
                        {#each settings.data.libraryDirs as dir (dir)}
                            <SettingsListItem title={dir}>
                                <Button
                                    variant="text"
                                    onclick={() =>
                                        removeMusicDirectory(dir)}
                                >
                                    <span class="text-red-500">移除</span>
                                </Button>
                            </SettingsListItem>
                        {/each}
                    </div>
                {:else}
                    <div
                        class="rounded-2xl border border-dashed border-[rgb(var(--mdui-color-outline-variant))] p-4 text-sm text-[rgb(var(--mdui-color-on-surface-variant))]"
                    >
                        暂未添加媒体库目录，应用将无法扫描到任何歌曲。
                    </div>
                {/if}
            </div>
        </SettingsSection>

        <SettingsSection title="插件">
            <SettingsRow
                title="启动时扫描插件"
                description="应用启动时自动发现并加载插件"
            >
                <mdui-switch
                    checked={settings.data?.pluginScanOnStartup}
                    onchange={(event: Event) =>
                        handleSwitchChange(event, 'pluginScanOnStartup')}
                ></mdui-switch>
            </SettingsRow>

            <SettingsRow
                title="开发者模式"
                description="启用插件开发调试功能"
            >
                <mdui-switch
                    checked={settings.data?.pluginDevMode}
                    onchange={(event: Event) =>
                        handleSwitchChange(event, 'pluginDevMode')}
                ></mdui-switch>
            </SettingsRow>

            <SettingsRow
                title="插件日志级别"
                description="设置插件系统的日志输出级别"
            >
                <div class="w-48">
                    <Select
                        value={settings.data?.pluginLogLevel}
                        options={logLevelOptions}
                        onchange={value =>
                            settings.update({
                                pluginLogLevel: value as PluginLogLevel,
                            })}
                    />
                </div>
            </SettingsRow>

            <div class="grid gap-3">
                <div>
                    <div class="text-sm font-medium">插件目录</div>
                    <div
                        class="text-xs text-[rgb(var(--mdui-color-on-surface-variant))]"
                    >
                        管理应用扫描插件的本地文件夹
                    </div>
                </div>

                <div class="flex">
                    <Button
                        variant="tonal"
                        onclick={async () => {
                            const { open } = await import(
                                '@tauri-apps/plugin-dialog'
                            )
                            const selected = await open({
                                directory: true,
                                multiple: false,
                            })
                            if (
                                selected &&
                                typeof selected === 'string' &&
                                !settings.data?.pluginDirs?.includes(selected)
                            ) {
                                settings.update({
                                    pluginDirs: [
                                        ...(settings.data?.pluginDirs ?? []),
                                        selected,
                                    ],
                                })
                            }
                        }}
                    >
                        添加插件目录
                    </Button>
                </div>

                {#if settings.data?.pluginDirs?.length > 0}
                    <div class="grid gap-2 mt-2">
                        {#each settings.data.pluginDirs as dir (dir)}
                            <SettingsListItem title={dir}>
                                <Button
                                    variant="text"
                                    onclick={() =>
                                        settings.update({
                                            pluginDirs:
                                                settings.data.pluginDirs.filter(
                                                    (d) => d !== dir,
                                                ),
                                        })}
                                >
                                    <span class="text-red-500">移除</span>
                                </Button>
                            </SettingsListItem>
                        {/each}
                    </div>
                {:else}
                    <div
                        class="rounded-2xl border border-dashed border-[rgb(var(--mdui-color-outline-variant))] p-4 text-sm text-[rgb(var(--mdui-color-on-surface-variant))]"
                    >
                        暂未添加插件目录。
                    </div>
                {/if}
            </div>
        </SettingsSection>

        {#if settings.error}
            <div class="text-sm text-red-500">
                {settings.error}
            </div>
        {/if}
    </div>
</section>
