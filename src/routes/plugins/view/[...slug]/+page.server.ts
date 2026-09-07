// 插件 view 路由按运行时 pluginState.manifests 解析，不能预渲染。
// 全局 +layout.ts 设置了 prerender=true，此处覆盖为 false，
// 使 /plugins/view/<任意 slug> 走 SPA fallback 而非仅限 lyrics。
export const prerender = false
