<script lang="ts">
    import { page } from '$app/state'
    import Heading from '$lib/components/base/Heading.svelte'
    import ListEditor from '$lib/features/settings/ListEditor.svelte'
    import SettingsRow from '$lib/features/settings/SettingsRow.svelte'
    import SettingsSection from '$lib/features/settings/SettingsSection.svelte'
    import { pluginState } from '$lib/state/plugins.svelte'
    import type { PluginSetting, SettingValue } from '$lib/types/plugin'
    import 'mdui/components/circular-progress.js'
    import 'mdui/components/switch.js'
    import 'mdui/components/text-field.js'
    import { onMount } from 'svelte'

    const pluginId = $derived(page.url.searchParams.get('id') ?? '')

    let settings = $state<PluginSetting[]>([])
    let isLoading = $state(true)

    // Json 字段校验错误，按 setting.key 索引；非 null 表示当前输入无法解析
    let jsonErrors = $state<Record<string, string | null>>({})

    const sidebarEntry = $derived(
        pluginState.sidebarExtensions.find((e) => e.pluginId === pluginId),
    )

    async function loadPluginData() {
        if (!pluginId) return
        isLoading = true
        settings = await pluginState.getPluginSettings(pluginId)
        isLoading = false
    }

    onMount(async () => {
        await pluginState.loadSidebarExtensions()
        await loadPluginData()
    })

    function getBoolValue(val: SettingValue): boolean {
        if (val.type === 'Bool') return val.value
        return false
    }

    function getStringValue(val: SettingValue): string {
        if (val.type === 'Text') return val.value
        if (val.type === 'Integer') return String(val.value)
        if (val.type === 'Float') return String(val.value)
        if (val.type === 'Json') return JSON.stringify(val.value, null, 2)
        if (val.type === 'List') return val.value.join('\n')
        return ''
    }

    async function handleSettingBoolChange(key: string, checked: boolean) {
        if (!pluginId) return
        await pluginState.updatePluginSetting(pluginId, key, {
            type: 'Bool',
            value: checked,
        })
        await loadPluginData()
    }

    // 按原 type 构造 SettingValue 写回，避免类型被改写为 Text
    async function handleSettingChange(
        key: string,
        type: SettingValue['type'],
        rawValue: string,
    ) {
        if (!pluginId) return
        let nextValue: SettingValue | null = null
        switch (type) {
            case 'Integer': {
                const num = parseInt(rawValue, 10)
                if (Number.isNaN(num)) return
                nextValue = { type: 'Integer', value: num }
                break
            }
            case 'Float': {
                const num = parseFloat(rawValue)
                if (Number.isNaN(num)) return
                nextValue = { type: 'Float', value: num }
                break
            }
            case 'Text':
                nextValue = { type: 'Text', value: rawValue }
                break
            case 'Json': {
                try {
                    const parsed = JSON.parse(rawValue)
                    nextValue = { type: 'Json', value: parsed }
                    jsonErrors[key] = null
                } catch (err) {
                    jsonErrors[key] =
                        err instanceof Error ? err.message : String(err)
                    return
                }
                break
            }
            case 'List':
                nextValue = { type: 'List', value: rawValue.split('\n') }
                break
            case 'Bool':
                return
        }
        if (!nextValue) return
        await pluginState.updatePluginSetting(pluginId, key, nextValue)
        await loadPluginData()
    }

    async function handleListChange(key: string, newArr: string[]) {
        if (!pluginId) return
        await pluginState.updatePluginSetting(pluginId, key, {
            type: 'List',
            value: newArr,
        })
        await loadPluginData()
    }

    // Json 输入时清除上一次的校验错误提示
    function handleJsonInput(key: string) {
        if (jsonErrors[key]) jsonErrors[key] = null
    }

    async function togglePlugin() {
        if (!pluginId) return
        if (sidebarEntry?.state === 'Enabled') {
            await pluginState.disablePlugin(pluginId)
        } else {
            await pluginState.enablePlugin(pluginId)
        }
        await pluginState.loadSidebarExtensions()
        await loadPluginData()
    }
</script>

<svelte:head>
    <title>插件 - {sidebarEntry?.title ?? pluginId}</title>
</svelte:head>

<section class="flex h-full min-h-0 flex-col gap-6 overflow-auto pb-10">
    <div class="flex items-center gap-4">
        <a
            href="/plugins"
            class="flex h-10 w-10 items-center justify-center rounded-full hover:bg-[rgb(var(--mdui-color-surface-container))]"
            title="plugins"
        >
            <mdui-icon name="arrow_back"></mdui-icon>
        </a>
        <Heading
            eyebrow="Plugin"
            title={sidebarEntry?.title ?? pluginId ?? '插件'}
        />
    </div>

    <div
        class="flex flex-1 items-center justify-center"
        class:hidden={!isLoading}
    >
        <mdui-circular-progress></mdui-circular-progress>
    </div>

    <div class="grid gap-6" class:hidden={isLoading}>
        <SettingsSection title="基本信息">
            <SettingsRow title="插件 ID" description="唯一标识符">
                <span
                    class="font-mono text-sm text-[rgb(var(--mdui-color-on-surface-variant))]"
                >
                    {pluginId}
                </span>
            </SettingsRow>

            <SettingsRow title="状态" description="启用或禁用此插件">
                <mdui-switch
                    checked={sidebarEntry?.state === 'Enabled'}
                    onchange={() => togglePlugin()}
                ></mdui-switch>
            </SettingsRow>
        </SettingsSection>

        {#if settings.length > 0}
            <SettingsSection title="插件设置">
                {#each settings as setting (setting.key)}
                    <SettingsRow title={setting.title} description={setting.key}>
                        {#if setting.value.type === 'Bool'}
                            <mdui-switch
                                checked={getBoolValue(setting.value)}
                                onchange={(e: Event) => {
                                    const target =
                                        e.currentTarget as HTMLInputElement
                                    handleSettingBoolChange(
                                        setting.key,
                                        target.checked,
                                    )
                                }}
                            ></mdui-switch>
                        {:else if setting.value.type === 'Integer'}
                            <mdui-text-field
                                type="number"
                                step="1"
                                inputmode="numeric"
                                value={getStringValue(setting.value)}
                                onkeydown={(e: KeyboardEvent) => {
                                    if (e.key === 'Enter') {
                                        const target =
                                            e.currentTarget as HTMLInputElement
                                        handleSettingChange(
                                            setting.key,
                                            'Integer',
                                            target.value,
                                        )
                                    }
                                }}
                                role="textbox"
                                tabindex="0"
                                style="width: 200px;"
                            ></mdui-text-field>
                        {:else if setting.value.type === 'Float'}
                            <mdui-text-field
                                type="number"
                                step="0.1"
                                inputmode="decimal"
                                value={getStringValue(setting.value)}
                                onkeydown={(e: KeyboardEvent) => {
                                    if (e.key === 'Enter') {
                                        const target =
                                            e.currentTarget as HTMLInputElement
                                        handleSettingChange(
                                            setting.key,
                                            'Float',
                                            target.value,
                                        )
                                    }
                                }}
                                role="textbox"
                                tabindex="0"
                                style="width: 200px;"
                            ></mdui-text-field>
                        {:else if setting.value.type === 'Text'}
                            <mdui-text-field
                                value={getStringValue(setting.value)}
                                onkeydown={(e: KeyboardEvent) => {
                                    if (e.key === 'Enter') {
                                        const target =
                                            e.currentTarget as HTMLInputElement
                                        handleSettingChange(
                                            setting.key,
                                            'Text',
                                            target.value,
                                        )
                                    }
                                }}
                                role="textbox"
                                tabindex="0"
                                style="width: 200px;"
                            ></mdui-text-field>
                        {:else if setting.value.type === 'Json'}
                            <div class="flex flex-col gap-1" style="width: 320px;">
                                <mdui-text-field
                                    rows="6"
                                    value={getStringValue(setting.value)}
                                    error={jsonErrors[setting.key] ?? ''}
                                    oninput={() => handleJsonInput(setting.key)}
                                    onblur={(e: Event) => {
                                        const target =
                                            e.currentTarget as HTMLInputElement
                                        handleSettingChange(
                                            setting.key,
                                            'Json',
                                            target.value,
                                        )
                                    }}
                                    role="textbox"
                                    tabindex="0"
                                ></mdui-text-field>
                                {#if jsonErrors[setting.key]}
                                    <div
                                        class="text-xs text-[rgb(var(--mdui-color-error))]"
                                    >
                                        JSON 解析失败：{jsonErrors[setting.key]}
                                    </div>
                                {/if}
                            </div>
                        {:else if setting.value.type === 'List'}
                            <ListEditor
                                value={setting.value.value}
                                onchange={(arr: string[]) =>
                                    handleListChange(setting.key, arr)}
                            />
                        {/if}
                    </SettingsRow>
                {/each}
            </SettingsSection>
        {/if}
    </div>
</section>
