<script lang="ts">
    import { onMount } from "svelte"
    import { invoke } from "@tauri-apps/api/core"
    import type { Track } from "$lib/types"
    import type { MenuExtension } from "$lib/types/plugin"
    import { pluginState } from "$lib/state/plugins.svelte"

    interface Props {
        open: boolean
        track: Track | null
        index: number

        x: number
        y: number

        onclose: () => void
        onplay?: (track: Track, index: number) => void | Promise<void>
        oninsertnext?: (track: Track, index: number) => void | Promise<void>
        onshowfolder?: (track: Track, index: number) => void | Promise<void>
        onremovefromlist?: (track: Track, index: number) => void | Promise<void>
        ondelete?: (track: Track, index: number) => void | Promise<void>
    }

    let {
        open,
        track,
        index,
        x,
        y,
        onclose,
        onplay,
        oninsertnext,
        onshowfolder,
        onremovefromlist,
        ondelete,
    }: Props = $props()

    // Plugin-contributed menu extensions for this context menu location.
    let menuExtensions = $state<MenuExtension[]>([])

    // Load plugin menu extensions on mount. Manifests are loaded app-wide in
    // the root layout, so the `$derived` below resolves titles reactively as
    // they arrive. Failures are swallowed so the native menu still works.
    onMount(() => {
        void pluginState
            .getMenuExtensions("TrackContextMenu")
            .then((exts) => {
                menuExtensions = exts
            })
            .catch(() => {
                menuExtensions = []
            })
    })

    type PluginMenuItem = {
        key: string
        title: string
        command: string
        showDivider: boolean
    }

    // Resolve each extension's display title from registered manifests. A
    // divider precedes the first plugin item (separating it from the native
    // items) and any item whose `group` differs from the previous one. Empty
    // when no plugin items exist, so no trailing divider is rendered.
    const pluginMenuItems = $derived.by<PluginMenuItem[]>(() => {
        const enabled = menuExtensions.filter((e) => e.state === "Enabled")
        return enabled.map((ext, i) => {
            const manifest = pluginState.manifests.find(
                (m) => m.id === ext.pluginId,
            )
            const menu = manifest?.contributes.menus.find(
                (m) => m.command === ext.command,
            )
            const title = menu?.title ?? ext.command
            const prev = enabled[i - 1]
            const showDivider = i === 0 || prev.group !== ext.group
            return { key: ext.id, title, command: ext.command, showDivider }
        })
    })

    async function execute(
        action?: (
            track: Track,
            index: number,
        ) => void | Promise<void>,
    ) {
        const clickedTrack = track
        const clickedIndex = index

        onclose()

        if (!clickedTrack || clickedIndex < 0) {
            return
        }

        await action?.(
            clickedTrack,
            clickedIndex,
        )
    }

    function handleKey(
        event: KeyboardEvent,
        action?: (track: Track, index: number) => void | Promise<void>,
    ) {
        if (event.key === "Enter" || event.key === " ") {
            void execute(action)
        }
    }

    // Run a plugin-contributed command against the current track. `args` uses
    // the default (externally-tagged) serde representation of the `CommandArgs`
    // enum, so `CommandArgs::TrackId(id)` is sent as `{ TrackId: id }`.
    async function executePluginCommand(commandId: string) {
        const clickedTrack = track
        onclose()
        if (!clickedTrack) return
        try {
            await invoke("execute_plugin_command", {
                commandId,
                args: { TrackId: clickedTrack.id },
            })
        } catch (err) {
            console.error("Failed to execute plugin command:", commandId, err)
        }
    }

    function handlePluginKey(event: KeyboardEvent, commandId: string) {
        if (event.key === "Enter" || event.key === " ") {
            void executePluginCommand(commandId)
        }
    }
</script>

{#if open && track}
    <div
        class="fixed z-50 min-w-44 overflow-hidden rounded-xl bg-[rgb(var(--mdui-color-surface-container-high))] py-1 text-sm text-[rgb(var(--mdui-color-on-surface))] shadow-xl"
        style:left={`${x}px`}
        style:top={`${y}px`}
        role="menu"
        tabindex="-1"
        oncontextmenu={(e) => e.preventDefault()}
    >
        <div
            class="menu-item"
            role="menuitem"
            tabindex="0"
            onclick={() => void execute(onplay)}
            onkeydown={(e) => handleKey(e, onplay)}
        >
            播放
        </div>

        <div
            class="menu-item"
            role="menuitem"
            tabindex="0"
            onclick={() => void execute(oninsertnext)}
            onkeydown={(e) => handleKey(e, oninsertnext)}
        >
            下一首播放
        </div>

        <div
            class="menu-item"
            role="menuitem"
            tabindex="0"
            onclick={() => void execute(onshowfolder)}
            onkeydown={(e) => handleKey(e, onshowfolder)}
        >
            打开文件所在目录
        </div>

        <div
            class="menu-item"
            role="menuitem"
            tabindex="0"
            onclick={() => void execute(onremovefromlist)}
            onkeydown={(e) => handleKey(e, onremovefromlist)}
        >
            从当前列表移除
        </div>

        <div
            class="menu-item danger"
            role="menuitem"
            tabindex="0"
            onclick={() => void execute(ondelete)}
            onkeydown={(e) => handleKey(e, ondelete)}
        >
            删除文件
        </div>

        {#each pluginMenuItems as item (item.key)}
            {#if item.showDivider}
                <div class="menu-divider"></div>
            {/if}
            <div
                class="menu-item"
                role="menuitem"
                tabindex="0"
                onclick={() => void executePluginCommand(item.command)}
                onkeydown={(e) => handlePluginKey(e, item.command)}
            >
                {item.title}
            </div>
        {/each}
    </div>
{/if}

<style>
    .menu-item {
        padding: 8px 16px;
        cursor: pointer;
    }

    .menu-item:hover {
        background: rgb(var(--mdui-color-surface-container-highest));
    }

    .danger {
        color: rgb(var(--mdui-color-error));
    }

    .menu-divider {
        height: 1px;
        margin: 4px 0;
        background: rgb(var(--mdui-color-outline-variant));
    }
</style>
