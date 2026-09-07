<script lang="ts">
    import IconButton from '$lib/components/base/IconButton.svelte'
    import { lyrics } from '$lib/state/lyrics.svelte'
    import { player } from '$lib/state/player.svelte'
    import { pluginState } from '$lib/state/plugins.svelte'
    import { tick } from 'svelte'

    let containerEl = $state<HTMLDivElement | null>(null)

    $effect(() => {
        if (lyrics.currentLineIndex >= 0 && containerEl) {
            void tick().then(() => {
                const lineEl = containerEl?.querySelector(
                    '[data-active="true"]',
                )
                if (lineEl) {
                    lineEl.scrollIntoView({
                        behavior: 'smooth',
                        block: 'center',
                    })
                }
            })
        }
    })

    $effect(() => {
        lyrics.updateCurrentTime(player.currentTime * 1000)
    })

    $effect(() => {
        if (
            player.currentTrack &&
            lyrics.currentLyrics?.songId !== player.currentTrack.id
        ) {
            lyrics.clear()
        }
    })
</script>

<div
    class="flex h-full flex-col overflow-hidden rounded-2xl bg-[rgb(var(--mdui-color-surface-container))] p-4"
>
    <div class="mb-3 flex items-center justify-between">
        <span
            class="text-sm font-medium text-[rgb(var(--mdui-color-on-surface))]"
        >
            歌词
        </span>
        <IconButton
            icon="close--rounded"
            onclick={() => (pluginState.activeNativePanel = null)}
        />
    </div>

    <div bind:this={containerEl} class="flex-1 overflow-y-auto">
        {#if lyrics.currentLyrics && lyrics.currentLyrics.lines.length > 0}
            <div class="flex flex-col gap-2 py-4">
                {#each lyrics.currentLyrics.lines as line, i (line.timestampMs + i)}
                    <div
                        class={[
                            'text-center text-sm transition-all duration-300',
                            i === lyrics.currentLineIndex
                                ? 'font-bold text-[rgb(var(--mdui-color-primary))]'
                                : 'text-[rgb(var(--mdui-color-on-surface-variant))] opacity-50',
                        ]}
                        data-active={i === lyrics.currentLineIndex}
                    >
                        {line.text || '···'}
                    </div>
                {/each}
            </div>
        {:else}
            <div
                class="flex h-full items-center justify-center text-sm text-[rgb(var(--mdui-color-on-surface-variant))]"
            >
                暂无歌词
            </div>
        {/if}
    </div>
</div>
