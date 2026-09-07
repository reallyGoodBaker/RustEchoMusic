export type PluginSource = 'builtin' | 'packaged' | 'user'

export type PluginPermission =
    | 'playerRead'
    | 'playerControl'
    | 'queueRead'
    | 'queueWrite'
    | 'libraryRead'
    | 'libraryWrite'
    | 'settingsRead'
    | 'settingsWrite'
    | 'pluginUI'

export type SettingValue =
    | { type: 'Bool'; value: boolean }
    | { type: 'Integer'; value: number }
    | { type: 'Float'; value: number }
    | { type: 'Text'; value: string }
    | { type: 'List'; value: string[] }
    | { type: 'Json'; value: unknown }

export interface PluginSetting {
    key: string
    title: string
    value: SettingValue
    defaultValue: SettingValue
}

export interface SidebarExtension {
    id: string
    pluginId: string
    title: string
    icon: string
    route: string
    state: 'Enabled' | 'Disabled'
}

export interface NativeViewExtension {
    id: string
    pluginId: string
    title: string
    token: string
    icon: string | null
    state: 'Enabled' | 'Disabled'
}

export interface MenuExtension {
    id: string
    pluginId: string
    command: string
    location: string
    group: string | null
    state: 'Enabled' | 'Disabled'
}

export interface ViewContribution {
    id: string
    title: string
    entry: string
    icon: string | null
}

export interface NativeViewContribution {
    id: string
    title: string
    token: string
    icon: string | null
}

/** Fully resolved, normalized manifest — the only plugin-identity shape
 *  the frontend ever consumes. Routes are already normalized to
 *  `/plugins/view/<slug>`; no optional fields. */
export interface ResolvedPluginManifest {
    id: string
    source: PluginSource
    route: string
    name: string
    displayName: string
    version: string
    author: string
    description: string
    entry: string
    minAppVersion: string
    permissions: string[]
    activationEvents: string[]
    contributes: {
        commands: { id: string; title: string; category: string | null }[]
        menus: {
            command: string
            title: string
            location: string
            group: string | null
        }[]
        sidebars: { id: string; title: string; icon: string }[]
        views: ViewContribution[]
        nativeViews: NativeViewContribution[]
    }
    settings: {
        key: string
        title: string
        defaultValue: SettingValue
    }[]
}

export interface PluginInfo {
    id: string
    displayName: string
    icon: string
    route: string
    state: 'Enabled' | 'Disabled'
}

export interface KernelPlugin {
    id: string
    version: string
    source: string
    tier: string
    userDisableable: boolean
    state: string
    health: string
    active: boolean
    lastError: string | null
}

export interface KernelSnapshot {
    hostVersion: string
    plugins: KernelPlugin[]
    contributions: { point: string; plugin: string; key: string }[]
    issues: string[]
}

export interface RichContribution {
    point: string
    plugin: string
    key: string
    payload: Record<string, any>
}
