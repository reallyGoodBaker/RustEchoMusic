/**
 * Plugin Host Bridge
 *
 * 宿主端 postMessage 桥：负责将沙箱 iframe 内的插件 SDK 请求路由到对应的
 * Tauri 命令/前端状态，并把宿主事件广播给所有已挂载的插件 iframe。
 *
 * 协议信封与 `static/plugin-sdk/plugin-host-sdk.js` 完全对齐：
 *   { source: 'rem-plugin-host' | 'rem-plugin', type: string, id?: string, payload?: unknown }
 */

import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { pluginState } from '$lib/state/plugins.svelte'
import { player } from '$lib/state/player.svelte'
import type { GlobalAppEvent, PluginSetting } from '$lib/types'

const HOST_SOURCE = 'rem-plugin-host'
const PLUGIN_SOURCE = 'rem-plugin'

export type PluginEventKind =
    | 'trackChanged'
    | 'playbackStateChanged'
    | 'queueChanged'
    | 'settingsChanged'

type StateKind = 'playback' | 'track' | 'queue'

// 与 Rust 侧 `EventPattern` 对齐的剪枝描述：all / 精确 kind / 前缀匹配。
type BridgeEventPattern =
    | { all: true }
    | { kind: string }
    | { prefix: string }

function patternMatches(pattern: BridgeEventPattern, kind: string): boolean {
    if ('all' in pattern) return true
    if ('kind' in pattern) return pattern.kind === kind
    return kind.startsWith(pattern.prefix)
}

// 未显式订阅的插件默认收全部事件；一旦显式订阅，只收匹配的事件。
function attachmentMatches(att: Attachment, kind: string): boolean {
    if (!att.subscribed) return true
    return att.subscriptions.some(p => patternMatches(p, kind))
}

interface PluginEnvelope {
    source: 'rem-plugin' | 'rem-plugin-host'
    type: string
    id?: string
    payload?: unknown
}

interface Attachment {
    pluginId: string
    messageListener: (e: MessageEvent) => void
    loadListener: () => void
    // 事件订阅剪枝：仅向订阅了对应事件的 iframe 广播。
    // `subscribed` 为 false 时视为"收全部"（兼容未显式订阅的旧插件）。
    subscribed: boolean
    subscriptions: BridgeEventPattern[]
}

class PluginHostBridge {
    #attachments = new Map<HTMLIFrameElement, Attachment>()
    #eventUnlisten: UnlistenFn | null = null
    #started = false

    constructor() {
        this.#startEventSubscription()
    }

    /**
     * 模块加载时启动一次 Tauri 事件订阅，将后端事件桥接为
     * 插件侧的 `event` 广播。单例，全程存活。
     */
    #startEventSubscription(): void {
        if (this.#started) return
        this.#started = true

        void listen<GlobalAppEvent>('global-app-event', event => {
            const { type, payload } = event.payload
            switch (type) {
                case 'TrackStarted':
                    this.broadcastEvent('trackChanged', payload)
                    break
                case 'PlaybackStateChanged':
                    this.broadcastEvent('playbackStateChanged', payload)
                    break
                case 'QueueChanged':
                    this.broadcastEvent('queueChanged', payload)
                    break
                case 'SettingsChanged':
                    this.broadcastEvent('settingsChanged', payload)
                    break
                // PlaybackProgress / VolumeChanged / LyricsLoaded 不广播给插件
            }
        }).then(unlisten => {
            this.#eventUnlisten = unlisten
        }).catch(err => {
            console.error('[host-bridge] 订阅 global-app-event 失败:', err)
        })
    }

    /**
     * 绑定 iframe，开始监听其 postMessage。返回清理函数。
     * 在 iframe `load` 时主动发送 `ready` 握手。
     */
    attach(iframe: HTMLIFrameElement, pluginId: string): () => void {
        // 已绑定过则先解绑，避免重复监听
        if (this.#attachments.has(iframe)) {
            this.detach(iframe)
        }

        const messageListener = (e: MessageEvent) => {
            // 来源校验：只接受本 iframe 内发出的消息（防越权）
            if (e.source !== iframe.contentWindow) return
            const data = e.data as PluginEnvelope | undefined
            // 信封校验：必须是插件信封
            if (!data || data.source !== PLUGIN_SOURCE) return
            void this.#handleMessage(iframe, pluginId, data)
        }

        const loadListener = () => {
            const capabilities = this.#getCapabilities(pluginId)
            this.sendReady(iframe, pluginId, capabilities)
        }

        window.addEventListener('message', messageListener)
        iframe.addEventListener('load', loadListener)

        this.#attachments.set(iframe, {
            pluginId,
            messageListener,
            loadListener,
            subscribed: false,
            subscriptions: [],
        })

        // 兜底：若 iframe 已加载完成（load 事件不会再触发），
        // 立即发一次 ready。SDK 对重复 ready 幂等处理。
        if (iframe.contentWindow) {
            loadListener()
        }

        return () => this.detach(iframe)
    }

    /** 解绑 iframe，移除所有监听 */
    detach(iframe: HTMLIFrameElement): void {
        const att = this.#attachments.get(iframe)
        if (!att) return
        window.removeEventListener('message', att.messageListener)
        iframe.removeEventListener('load', att.loadListener)
        this.#attachments.delete(iframe)
    }

    /** 向指定 iframe 发送 ready 握手消息 */
    sendReady(
        iframe: HTMLIFrameElement,
        pluginId: string,
        capabilities: string[],
    ): void {
        const win = iframe.contentWindow
        if (!win) return
        const envelope: PluginEnvelope = {
            source: HOST_SOURCE,
            type: 'ready',
            payload: { pluginId, capabilities },
        }
        win.postMessage(envelope, '*')
    }

    /** 向所有已 attach 且订阅了该事件的 iframe 广播事件 */
    broadcastEvent(kind: PluginEventKind, data: unknown): void {
        if (this.#attachments.size === 0) return
        const envelope: PluginEnvelope = {
            source: HOST_SOURCE,
            type: 'event',
            payload: { kind, data },
        }
        for (const [iframe, att] of this.#attachments) {
            if (!attachmentMatches(att, kind)) continue
            const win = iframe.contentWindow
            if (!win) continue
            win.postMessage(envelope, '*')
        }
    }

    /** 从 manifest 中读取插件 permissions 作为 capabilities */
    #getCapabilities(pluginId: string): string[] {
        const manifest = pluginState.manifests.find(m => m.id === pluginId)
        return manifest?.permissions ?? []
    }

    /** 路由插件请求到对应 Tauri 命令或前端状态，并回写响应 */
    async #handleMessage(
        iframe: HTMLIFrameElement,
        pluginId: string,
        env: PluginEnvelope,
    ): Promise<void> {
        const { type, id, payload } = env

        switch (type) {
            case 'command': {
                try {
                    const p = (payload ?? {}) as { command: string; args?: unknown }
                    const result = await invoke('execute_plugin_command', {
                        commandId: p.command,
                        args: p.args ?? null,
                    })
                    this.#sendResult(iframe, 'command:result', id, {
                        ok: true,
                        data: result,
                    })
                } catch (e) {
                    this.#sendResult(iframe, 'command:result', id, {
                        ok: false,
                        error: String(e),
                    })
                }
                break
            }

            case 'settings:get': {
                try {
                    const p = (payload ?? {}) as { key: string }
                    const settings = await invoke<PluginSetting[]>(
                        'get_plugin_settings',
                        { pluginId },
                    )
                    const found = settings.find(s => s.key === p.key)
                    if (!found) {
                        this.#sendResult(iframe, 'settings:get:result', id, {
                            ok: false,
                            error: 'key not found',
                        })
                    } else {
                        // 裸值：直接返回 SettingValue（tagged union）
                        this.#sendResult(
                            iframe,
                            'settings:get:result',
                            id,
                            found.value,
                        )
                    }
                } catch (e) {
                    this.#sendResult(iframe, 'settings:get:result', id, {
                        ok: false,
                        error: String(e),
                    })
                }
                break
            }

            case 'settings:set': {
                try {
                    const p = (payload ?? {}) as { key: string; value: unknown }
                    await invoke('update_plugin_setting', {
                        pluginId,
                        key: p.key,
                        value: p.value,
                    })
                    this.#sendResult(iframe, 'settings:set:result', id, {
                        ok: true,
                    })
                } catch (e) {
                    this.#sendResult(iframe, 'settings:set:result', id, {
                        ok: false,
                        error: String(e),
                    })
                }
                break
            }

            case 'state:get': {
                try {
                    const p = (payload ?? {}) as { kind: StateKind }
                    const snapshot = this.#getStateSnapshot(p.kind)
                    this.#sendResult(iframe, 'state:get:result', id, snapshot)
                } catch (e) {
                    this.#sendResult(iframe, 'state:get:result', id, {
                        ok: false,
                        error: String(e),
                    })
                }
                break
            }

            case 'subscribe': {
                const p = (payload ?? {}) as {
                    kind?: string
                    prefix?: string
                    all?: boolean
                }
                const att = this.#attachments.get(iframe)
                if (att) {
                    att.subscribed = true
                    if (p.all) att.subscriptions.push({ all: true })
                    else if (p.kind) att.subscriptions.push({ kind: p.kind })
                    else if (p.prefix) att.subscriptions.push({ prefix: p.prefix })
                }
                break
            }

            case 'unsubscribe': {
                const p = (payload ?? {}) as { kind?: string; prefix?: string }
                const att = this.#attachments.get(iframe)
                if (att) {
                    if (p.kind) {
                        att.subscriptions = att.subscriptions.filter(
                            s => !('kind' in s) || s.kind !== p.kind,
                        )
                    } else if (p.prefix) {
                        att.subscriptions = att.subscriptions.filter(
                            s => !('prefix' in s) || s.prefix !== p.prefix,
                        )
                    } else {
                        att.subscriptions = []
                    }
                }
                break
            }

            default:
                console.warn(
                    `[host-bridge] 未识别的插件消息类型: ${type}（plugin=${pluginId}）`,
                )
                break
        }
    }

    /** 从 player 状态读取快照 */
    #getStateSnapshot(kind: StateKind): unknown {
        switch (kind) {
            case 'playback':
                return {
                    playing: player.playing,
                    currentTime: player.currentTime,
                }
            case 'track':
                return { currentTrack: player.currentTrack }
            case 'queue':
                return { queue: player.queue }
        }
    }

    /** 向 iframe 回写响应消息 */
    #sendResult(
        iframe: HTMLIFrameElement,
        type: string,
        id: string | undefined,
        payload: unknown,
    ): void {
        const win = iframe.contentWindow
        if (!win) return
        const envelope: PluginEnvelope = {
            source: HOST_SOURCE,
            type,
            payload,
        }
        if (id !== undefined) envelope.id = id
        win.postMessage(envelope, '*')
    }
}

export const hostBridge = new PluginHostBridge()
