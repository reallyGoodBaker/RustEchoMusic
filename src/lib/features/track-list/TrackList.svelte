<script lang="ts">
    import { player } from "$lib/state/player.svelte"
    import type { Track } from "$lib/types"
    import { createVirtualizer, type PartialKeys, type VirtualizerOptions } from "@tanstack/svelte-virtual"
    import "mdui/components/list.js"
    import { get } from "svelte/store"
    import TrackContextMenu from "./TrackContextMenu.svelte"
    import TrackListRow from "./TrackListRow.svelte"

    type TrackColumn = "index" | "title" | "album" | "duration";

    interface Props {
        tracks: Track[];
        columns?: TrackColumn[];
        selectable?: boolean;
        selectedIds?: number[];
        currentTrackId?: number;
        hideHeader?: boolean;
        ontoggletrack?: (track: Track, checked: boolean) => void;
        ondblclicktrack?: (track: Track, index: number) => void;
        onremovetrack?: (track: Track, index: number) => void;
        onplaytrack?: (track: Track, index: number) => void | Promise<void>;
        oninsertnexttrack?: (
            track: Track,
            index: number,
        ) => void | Promise<void>;
        onshowtrackfolder?: (
            track: Track,
            index: number,
        ) => void | Promise<void>;
        onremovetrackfromlist?: (
            track: Track,
            index: number,
        ) => void | Promise<void>;
        ondeletetrackfile?: (
            track: Track,
            index: number,
        ) => void | Promise<void>;
    }

    const columnWidths: Record<TrackColumn, string> = {
        index: "24px",
        title: "1fr",
        album: "1fr",
        duration: "60px",
    };

    let {
        tracks,
        columns = ["index", "title", "album", "duration"],
        selectable = false,
        selectedIds = [],
        hideHeader = false,
        ontoggletrack,
        ondblclicktrack,
        onremovetrack,
        onplaytrack,
        oninsertnexttrack,
        onshowtrackfolder,
        onremovetrackfromlist,
        ondeletetrackfile,
    }: Props = $props();

    let contextMenuTrack = $state<Track | null>(null);
    let contextMenuIndex = $state(-1);
    let contextMenuX = $state(0);
    let contextMenuY = $state(0);
    let contextMenuOpen = $state(false);
    let currentTrackId = $derived(player.currentTrack?.id);
    let gridTemplate = $derived(
        [
            selectable ? "48px" : null,
            ...columns.map((column) => columnWidths[column]),
        ]
            .filter(Boolean)
            .join(" "),
    );

    // 虚拟列表相关
    let scrollElement = $state<HTMLDivElement | null>(null);

    const options = $derived({
        count: tracks.length,
        getScrollElement: () => scrollElement,
        estimateSize: () => 52, // 根据 TrackListRow 实际高度调整（行高 + 间距）
        overscan: 5,
        getItemKey: index => tracks[index]?.id ?? index,
    } satisfies Parameters<typeof createVirtualizer>[0])

    const virtualizer = $derived.by(() => createVirtualizer(options))

    // 当 tracks 变化时更新虚拟列表配置
    $effect(() => {
        const len = tracks.length;
        const v = get(virtualizer);
        v.setOptions({
            count: len,
            getItemKey: (index) => tracks[index]?.id ?? index,
        });
    });

    function isTrackSelected(track: Track) {
        return selectedIds.includes(track.id);
    }

    function setTrackSelected(track: Track, checked: boolean) {
        ontoggletrack?.(track, checked);
    }

    function playTrack(track: Track, index: number) {
        if (onplaytrack) {
            return onplaytrack(track, index);
        }
        return undefined;
    }

    function openTrackContextMenu(e: MouseEvent, track: Track, index: number) {
        e.preventDefault();
        e.stopPropagation();
        contextMenuTrack = track;
        contextMenuIndex = index;
        contextMenuX = e.clientX;
        contextMenuY = e.clientY;
        contextMenuOpen = true;
    }

    function closeContextMenu() {
        contextMenuOpen = false;
        contextMenuTrack = null;
        contextMenuIndex = -1;
    }

    function handleWindowClick() {
        if (contextMenuOpen) {
            closeContextMenu();
        }
    }

    function handleWindowKeydown(event: KeyboardEvent) {
        if (event.key === "Escape" && contextMenuOpen) {
            closeContextMenu();
        }
    }
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

<!-- 注意：确保 TrackList 的父容器有明确高度，否则 h-full 不会生效 -->
<mdui-list class="flex flex-col w-full h-full" role="list">
    {#if !hideHeader}
        <mdui-list-subheader
            class="sticky top-0 grid items-center gap-4 w-full pl-4 pr-6 text-xs uppercase tracking-wider font-semibold themed-text-secondary themed-border-b shrink-0"
            style:grid-template-columns={gridTemplate}
            noninteractive
        >
            {#if selectable}
                <div></div>
            {/if}

            {#if columns.includes("index")}
                <div class="text-center">#</div>
            {/if}

            {#if columns.includes("title")}
                <div class="pl-2 text-left">标题</div>
            {/if}

            {#if columns.includes("album")}
                <div class="text-left hidden md:block">专辑</div>
            {/if}

            {#if columns.includes("duration")}
                <div class="text-left">时长</div>
            {/if}
        </mdui-list-subheader>
    {/if}

    <!-- 滚动容器 -->
    <div
        bind:this={scrollElement}
        class="flex-1 overflow-auto min-h-0"
    >
        <!-- 内容占位容器，高度为所有行的总高度 -->
        <div
            style="position: relative; height: {$virtualizer.getTotalSize()}px;"
        >
    {#each $virtualizer.getVirtualItems() as virtualItem (virtualItem.key)}
        {@const track = tracks[virtualItem.index]}
        <div
            style="position: absolute; top: 0; left: 0; width: 100%; transform: translateY({virtualItem.start}px);"
        >
            <TrackListRow
                {track}
                index={virtualItem.index}
                {columns}
                {selectable}
                selected={isTrackSelected(track)}
                isCurrent={currentTrackId === track.id}
                playing={player.playing}
                {gridTemplate}
                ontoggle={(checked) => setTrackSelected(track, checked)}
                {ondblclicktrack}
                {onremovetrack}
                onplay={() => playTrack(track, virtualItem.index)}
                oncontextmenu={(e) => openTrackContextMenu(e, track, virtualItem.index)}
            />
        </div>
    {/each}
        </div>
    </div>
</mdui-list>

<TrackContextMenu
    open={contextMenuOpen}
    track={contextMenuTrack}
    index={contextMenuIndex}
    x={contextMenuX}
    y={contextMenuY}
    onclose={closeContextMenu}
    onplay={playTrack}
    oninsertnext={oninsertnexttrack}
    onshowfolder={onshowtrackfolder}
    onremovefromlist={onremovetrackfromlist}
    ondelete={ondeletetrackfile}
/>

<style>
    .themed-text-secondary {
        color: rgb(var(--mdui-color-on-surface-variant));
    }

    .themed-border-b {
        border-bottom: 1px solid rgb(var(--mdui-color-outline-variant));
    }
</style>