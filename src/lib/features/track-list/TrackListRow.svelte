<script lang="ts">
    import IconButton from "$lib/components/base/IconButton.svelte";
    import PlayingIndicator from "$lib/components/base/PlayingIndicator.svelte";
    import { trackCovers } from "$lib/state/covers.svelte";
    import type { Track } from "$lib/types";
    import { formatDuration } from "$lib/utils";
    import "@mdui/icons/play-arrow--rounded.js";
    import "mdui/components/checkbox.js";
    import "mdui/components/list-item.js";

    type TrackColumn = "index" | "title" | "album" | "duration";

    interface Props {
        track: Track;
        index: number;
        columns?: TrackColumn[];
        selectable?: boolean;
        selected?: boolean;
        isCurrent?: boolean;
        playing?: boolean;
        ontoggle?: (checked: boolean) => void;
        ondblclicktrack?: (track: Track, index: number) => void;
        onplay?: () => void | Promise<void>;
        onremovetrack?: (track: Track, index: number) => void;
        oncontextmenu?: (e: MouseEvent) => void;
        gridTemplate: string;
    }

    let {
        track,
        index,
        columns = ["index", "title", "album", "duration"],
        selectable = false,
        selected = false,
        isCurrent = false,
        playing = false,
        ontoggle: ontoggletrack,
        ondblclicktrack,
        onplay,
        onremovetrack,
        oncontextmenu,
        gridTemplate,
    }: Props = $props();

    const cover = $derived(trackCovers.get(track));

    function isCheckboxEventTarget(target: EventTarget | null) {
        return (
            target instanceof Element &&
            Boolean(target.closest("mdui-checkbox"))
        );
    }

    function lazyCover(track: Track) {
        return (node: HTMLElement) => {
            const load = () => {
                void trackCovers.load(track);
            };

            const observer = new IntersectionObserver(
                (entries) => {
                    if (entries[0].isIntersecting) {
                        load();
                        observer.disconnect();
                    }
                },
                {
                    rootMargin: "300px",
                },
            );

            const rect = node.getBoundingClientRect();
            const isVisible =
                rect.top < window.innerHeight + 300 && rect.bottom > -300;

            if (isVisible) {
                load();
            } else {
                observer.observe(node);
            }

            return () => {
                observer.disconnect();
            };
        };
    }

    function handleTrackToggle(e: Event) {
        e.stopPropagation();
        const target = e.currentTarget as HTMLElement & {
            checked: boolean;
        };
        ontoggletrack?.(target.checked);
    }

    function handleTrackClick(e: MouseEvent) {
        if (!selectable || isCheckboxEventTarget(e.target)) return;
        ontoggletrack?.(!selected);
    }

    function handleTrackDoubleClick(e: MouseEvent) {
        if (selectable || isCheckboxEventTarget(e.target)) return;

        if (ondblclicktrack) {
            ondblclicktrack(track, index);
        } else {
            void onplay?.();
        }

        void trackCovers.load(track);
    }

    function handleTrackKeydown(e: KeyboardEvent) {
        if (isCheckboxEventTarget(e.target)) return;

        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            if (selectable) {
                ontoggletrack?.(!selected);
            } else {
                void onplay?.();
            }
        }
    }

    function openContextMenu(e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        oncontextmenu?.(e);
    }
</script>

{#snippet trackIndexCell()}
    <div class="flex justify-center items-center text-sm themed-text-secondary">
        {#if isCurrent && playing}
            <PlayingIndicator />
        {:else}
            <span
                class={isCurrent
                    ? "font-bold themed-text-accent"
                    : "themed-text-secondary"}
            >
                {index + 1}
            </span>
        {/if}
    </div>
{/snippet}

{#snippet trackCoverCell(cover: string | null)}
    <div
        {@attach lazyCover(track)}
        class="relative w-10 h-10 rounded-md overflow-hidden shrink-0 transition-transform duration-100! group-hover:scale-105 group-hover:shadow-md"
    >
        {#if cover}
            <img
                src={cover}
                alt="cover"
                class="w-full h-full object-cover"
                loading="lazy"
            />
        {:else}
            <div
                class="w-full h-full flex items-center justify-center text-base themed-surface-high"
            >
                🎵
            </div>
        {/if}

        {#if !columns.includes("index") && isCurrent}
            <div
                class="absolute inset-0 flex items-center justify-center bg-black/40 text-lg text-(--mdui-color-inverse-primary-light)"
            >
                {#if playing}
                    <PlayingIndicator
                        color="rgb(var(--mdui-color-inverse-primary-light))"
                    />
                {:else}
                    <mdui-icon-play-arrow--rounded
                        class="text-[rgb(var(--mdui-color-inverse-primary-light))]"
                    ></mdui-icon-play-arrow--rounded>
                {/if}
            </div>
        {/if}
    </div>
{/snippet}

{#snippet trackTitleCell(cover: string | null)}
    <div class="flex items-center gap-4 pl-3 overflow-hidden min-w-0">
        {@render trackCoverCell(cover)}

        <div class="flex flex-col min-w-0">
            <span
                class={[
                    "text-sm font-medium truncate",
                    isCurrent ? "themed-text-accent" : "themed-text-primary",
                ]}
            >
                {track.title}
            </span>

            <span class="text-xs truncate themed-text-secondary mt-0.5">
                {track.artist ?? "未知歌手"}
            </span>
        </div>
    </div>
{/snippet}

{#snippet trackAlbumCell()}
    <div class="text-sm truncate hidden md:block themed-text-secondary min-w-0">
        {track.album ?? "未知专辑"}
    </div>
{/snippet}

{#snippet trackDurationCell()}
    <div class="flex items-center justify-between gap-2 w-full min-w-0">
        <div class="text-sm font-mono themed-text-secondary text-left">
            {formatDuration(track.duration / 1000)}
        </div>
        {#if onremovetrack}
            <IconButton
                icon="close--rounded"
                class="text-xs themed-text-secondary opacity-0 transition-opacity duration-200 hover:text-red-500 group-hover:opacity-100"
                onclick={(e: Event) => {
                    e.stopPropagation();
                    onremovetrack?.(track, index);
                }}
            />
        {/if}
    </div>
{/snippet}

<mdui-list-item
    class={[
        "group w-full rounded-xl transition-all duration-200 text-left overflow-hidden outline-none active:scale-[0.995] themed-item",
        isCurrent && "themed-item-active",
    ]}
    onclick={handleTrackClick}
    ondblclick={handleTrackDoubleClick}
    oncontextmenu={openContextMenu}
    onkeydown={handleTrackKeydown}
    role="button"
    tabindex="0"
>
    <mdui-ripple></mdui-ripple>

    <div
        class="grid items-center gap-4 w-full"
        style:grid-template-columns={gridTemplate}
    >
        {#if selectable}
            <div class="flex justify-center items-center">
                <mdui-checkbox checked={selected} onchange={handleTrackToggle}
                ></mdui-checkbox>
            </div>
        {/if}

        {#if columns.includes("index")}
            {@render trackIndexCell()}
        {/if}

        {#if columns.includes("title")}
            {@render trackTitleCell(cover)}
        {/if}

        {#if columns.includes("album")}
            {@render trackAlbumCell()}
        {/if}

        {#if columns.includes("duration")}
            {@render trackDurationCell()}
        {/if}
    </div>
</mdui-list-item>

<style>
    .themed-text-primary {
        color: rgb(var(--mdui-color-on-surface));
    }

    .themed-text-secondary {
        color: rgb(var(--mdui-color-on-surface-variant));
    }

    .themed-text-accent {
        color: rgb(var(--mdui-color-primary));
    }

    .themed-surface-high {
        background-color: rgb(var(--mdui-color-surface-container-high));
    }

    .themed-item {
        background-color: transparent;
        border: 1px solid transparent;
    }

    .themed-item:hover {
        background-color: rgb(var(--mdui-color-surface-container));
        box-shadow:
            0 4px 6px -1px rgba(0, 0, 0, 0.1),
            0 2px 4px -1px rgba(0, 0, 0, 0.06);
    }

    .themed-item-active {
        background-color: rgb(var(--mdui-color-surface-container));
        border-color: rgb(var(--mdui-color-outline-variant));
    }
</style>
