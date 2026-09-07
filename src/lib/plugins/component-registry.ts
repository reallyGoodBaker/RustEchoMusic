import type { Component } from 'svelte'

type ComponentLoader = () => Promise<{ default: Component }>

const registry = new Map<string, ComponentLoader>()

export function registerNativeView(token: string, loader: ComponentLoader): void {
    if (registry.has(token)) {
        console.warn(`[component-registry] token "${token}" already registered, overriding`)
    }
    registry.set(token, loader)
}

export function resolveNativeView(token: string): Promise<{ default: Component } | null> {
    const loader = registry.get(token)
    if (!loader) return Promise.resolve(null)
    return loader()
}
