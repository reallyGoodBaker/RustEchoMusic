<script lang="ts">
    import IconButton from '$lib/components/base/IconButton.svelte'
    import { player } from '$lib/state/player.svelte'
    import type { Track } from '$lib/types'
    import TrackList from './track-list/TrackList.svelte'

    function handlePlay(_track: Track, index: number) {
        player.playTrackInQueue(index)
    }

    function handleRemove(track: Track, _index: number) {
        player.removeTrack(track.id)
    }

    function handleClearQueue() {
        if (player.queue.tracks.length === 0) return
        player.clearQueue()
    }
</script>

<mdui-navigation-drawer
    open={player.queueOpen}
    placement="right"
    modal
    close-on-overlay-click
    close-on-esc
    onclose={() => {
        player.queueOpen = false
    }}
    class={[
        'mt-32',
        'h-[calc(100vh-var(--spacing)*(24+14+20))]',
        '[&::part(panel)]:mt-20',
        '[&::part(overlay)]:bg-transparent',
    ]}
>
    <div class="flex h-full min-h-0 flex-col bg-(--mdui-color-surface)">
        <div
            class="sticky top-0 z-10 flex items-center justify-between bg-(--mdui-color-surface) p-4 themed-border-b"
        >
            <div class="flex items-center gap-2">
                <span class="text-lg font-semibold themed-text-primary">
                    播放队列
                </span>
                <span
                    class="border rounded-full px-2 py-0.5 text-xs font-medium themed-tag-chip"
                >
                    {player.queue.tracks.length}
                </span>
            </div>
            <div class="flex items-center gap-1">
                <IconButton
                    icon="delete_sweep--rounded"
                    disabled={player.queue.tracks.length === 0}
                    onclick={handleClearQueue}
                />
                <IconButton
                    icon="close--rounded"
                    onclick={() => {
                        player.queueOpen = false
                    }}
                />
            </div>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto p-2">
            {#if player.queue.tracks.length === 0}
                <div
                    class="flex h-64 flex-col items-center justify-center p-6 text-center themed-text-secondary"
                >
                    <span class="text-4xl mb-2">🎵</span>
                    <p class="text-sm">队列为空</p>
                </div>
            {:else}
                <TrackList
                    tracks={player.queue.tracks}
                    columns={['title', 'duration']}
                    currentTrackId={player.currentTrack?.id}
                    hideHeader
                    ondblclicktrack={handlePlay}
                    onremovetrack={handleRemove}
                />
            {/if}
        </div>
    </div>
</mdui-navigation-drawer>

<style>
    .themed-text-primary {
        color: rgb(var(--mdui-color-on-surface));
    }

    .themed-text-secondary {
        color: rgb(var(--mdui-color-on-surface-variant));
    }
    

    .themed-border-b {
        border-bottom: 1px solid rgb(var(--mdui-color-outline-variant));
    }

    .themed-tag-chip {
        color: rgb(var(--mdui-color-on-surface-variant));
        background-color: rgb(var(--mdui-color-surface-container-high));
        border-color: rgb(var(--mdui-color-outline-variant));
    }
</style>
