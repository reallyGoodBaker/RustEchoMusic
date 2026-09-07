import { invoke } from '@tauri-apps/api/core'
import type {
    KernelSnapshot,
    MenuExtension,
    NativeViewExtension,
    PluginSetting,
    ResolvedPluginManifest,
    RichContribution,
    SettingValue,
    SidebarExtension,
    ViewContribution,
} from '$lib/types/plugin'

class PluginState {
    manifests = $state<ResolvedPluginManifest[]>([])
    sidebarExtensions = $state<SidebarExtension[]>([])
    nativeViewExtensions = $state<NativeViewExtension[]>([])
    /** 当前激活的原生面板（pluginId），null 表示无面板显示 */
    activeNativePanel = $state<string | null>(null)
    isLoading = $state(false)
    error = $state<string | null>(null)

    kernel = $state<KernelSnapshot | null>(null)
    contributions = $state<RichContribution[]>([])
    kernelLoading = $state(false)

    /**
     * 按 slug 解析插件 view。默认渲染 manifest 的第一个 view
     * （多 view 场景不在本期范围）。
     */
    getViewForSlug(slug: string): { manifest: ResolvedPluginManifest; view: ViewContribution } | null {
        const manifest = this.manifests.find(m => m.route === `/plugins/view/${slug}`)
        if (!manifest) return null
        const view = manifest.contributes.views[0]
        if (!view) return null
        return { manifest, view }
    }

    async loadManifests() {
        try {
            this.manifests = await invoke<ResolvedPluginManifest[]>(
                'get_plugin_manifests',
            )
        } catch (err) {
            console.error(err)
            this.error = String(err)
        }
    }

    async loadSidebarExtensions() {
        this.isLoading = true
        this.error = null
        try {
            this.sidebarExtensions = await invoke<SidebarExtension[]>(
                'get_all_sidebar_extensions',
            )
        } catch (err) {
            console.error(err)
            this.error = String(err)
        } finally {
            this.isLoading = false
        }
    }

    async loadNativeViewExtensions() {
        this.isLoading = true
        this.error = null
        try {
            this.nativeViewExtensions = await invoke<NativeViewExtension[]>(
                'get_native_view_extensions',
            )
        } catch (err) {
            console.error(err)
            this.error = String(err)
        } finally {
            this.isLoading = false
        }
    }

    async getMenuExtensions(location: string): Promise<MenuExtension[]> {
        try {
            return await invoke<MenuExtension[]>('get_menu_extensions', {
                location,
            })
        } catch (err) {
            console.error(err)
            return []
        }
    }

    async getPluginSettings(pluginId: string): Promise<PluginSetting[]> {
        try {
            return await invoke<PluginSetting[]>('get_plugin_settings', {
                pluginId,
            })
        } catch (err) {
            console.error(err)
            return []
        }
    }

    async updatePluginSetting(
        pluginId: string,
        key: string,
        value: SettingValue,
    ) {
        try {
            await invoke('update_plugin_setting', { pluginId, key, value })
        } catch (err) {
            console.error(err)
            this.error = String(err)
        }
    }

    async enablePlugin(pluginId: string) {
        try {
            await invoke('enable_plugin_command', { pluginId })
        } catch (err) {
            console.error(err)
            this.error = String(err)
        }
    }

    async disablePlugin(pluginId: string) {
        try {
            await invoke('disable_plugin_command', { pluginId })
        } catch (err) {
            console.error(err)
            this.error = String(err)
        }
    }

    async loadKernel() {
        this.kernelLoading = true
        this.error = null
        try {
            const [snapshot, contributions] = await Promise.all([
                invoke<KernelSnapshot>('plugin_kernel_snapshot'),
                invoke<RichContribution[]>('plugin_contributions_full'),
            ])
            this.kernel = snapshot
            this.contributions = contributions
        } catch (err) {
            console.error(err)
            this.error = String(err)
        } finally {
            this.kernelLoading = false
        }
    }
}

export const pluginState = new PluginState()
