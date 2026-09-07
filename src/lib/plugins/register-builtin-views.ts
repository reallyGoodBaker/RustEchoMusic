import type { Component } from 'svelte'
import { registerNativeView } from './component-registry'
import { pluginState } from '$lib/state/plugins.svelte'

type ViewLoader = () => Promise<{ default: Component }>

// 内置原生视图组件目录：token -> 动态导入器。
// 这是应用自身已知的"内置面板"清单；具体要注册哪些 token，
// 由贡献点数据（diagnostics().contributions 中的 ui.nativeView）驱动。
const BUILTIN_NATIVE_VIEWS: Record<string, ViewLoader> = {
    'eq-panel': () => import('$lib/features/EqPanel.svelte'),
    'lyrics-panel': () => import('$lib/features/LyricsPanel.svelte'),
}

export function registerBuiltinViews(): void {
    const tokens = pluginState.contributions
        .filter(c => c.point === 'ui.nativeView')
        .map(c => (c.payload as { token?: string }).token)
        .filter((t): t is string => !!t)

    // 贡献点尚未加载（或加载失败）时退回注册全部内置视图，避免 EQ/歌词面板消失。
    const effective = tokens.length > 0 ? tokens : Object.keys(BUILTIN_NATIVE_VIEWS)

    for (const token of effective) {
        const loader = BUILTIN_NATIVE_VIEWS[token]
        if (loader) registerNativeView(token, loader)
    }
}
