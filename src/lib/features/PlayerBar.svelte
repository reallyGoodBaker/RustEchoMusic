<script lang="ts">
    import IconButton from '$lib/components/base/IconButton.svelte'
    import MduiSlider from '$lib/components/base/MduiSlider.svelte'
    import Slider from '$lib/components/base/Slider.svelte'
    import { trackCovers } from '$lib/state/covers.svelte'
    import { player } from '$lib/state/player.svelte'
    import { pluginState } from '$lib/state/plugins.svelte'
    import { formatTime } from '$lib/utils'
    import type { Slider as MduiSliderElement } from 'mdui'

    let slider = $state<MduiSliderElement | null>(null)
    $effect(() => {
        if (slider) {
            slider.labelFormatter = (value: number) => formatTime(value)
        }
    })

    let isDragging = $state(false)
    let localProgress = $state(0)

    let currentCover = $derived(
        player.currentTrack?.cover ?? trackCovers.get(player.currentTrack),
    )
    let volumeIcon = $derived(getVolumeIcon(player.volume, player.muted))
    let playModeIcon = $derived.by(() => {
        if (player.queue.playMode === 'SingleLoop') return 'repeat_one--rounded'
        if (player.queue.playMode === 'Shuffle') return 'shuffle--rounded'
        return 'repeat--rounded'
    })

    function toggleNativePanel(pluginId: string) {
        pluginState.activeNativePanel =
            pluginState.activeNativePanel === pluginId ? null : pluginId
    }

    function getVolumeIcon(volume: number, muted: boolean) {
        if (muted) return 'volume_off--rounded'
        if (volume === 0) return 'volume_mute--rounded'
        return volume > 50 ? 'volume_up--rounded' : 'volume_down--rounded'
    }

    function handleSliderInput(e: Event): void {
        const target = e.currentTarget as HTMLElement & { value: number | string }

        isDragging = true

        localProgress = Number(target.value)
    }

    async function handleSliderChange(e: Event): Promise<void> {
        const target = e.currentTarget as HTMLElement & { value: number | string }
        const finalValue = Number(target.value)

        player.currentTime = finalValue
        await player.seek(finalValue)

        isDragging = false
    }
</script>

{#snippet progressArea()}
    <div
        class="absolute top-2 left-0 w-full flex items-center -translate-y-1/2"
    >
        <span class="text-xs tabular-nums text-black min-w-10 text-right">
            {formatTime(player.currentTime)}
        </span>

        <div class="flex-1 z-9999!">
            <MduiSlider
                bind:el={slider}
                value={isDragging ? localProgress : player.currentTime}
                duration={(player.currentTrack?.duration ?? 0) / 1000}
                oninput={handleSliderInput}
                onchange={handleSliderChange}
            />
        </div>

        <span class="text-xs tabular-nums text-black min-w-10">
            {formatTime((player.currentTrack?.duration ?? 0) / 1000)}
        </span>
    </div>
{/snippet}

{#snippet nowPlayingInfo()}
    <div class="flex flex-1 min-w-37.5 gap-2">
        <img
            src={currentCover ?? '/default_cover.png'}
            alt="Album Cover"
            class="w-16 h-16 rounded object-cover shadow-sm bg-gray-200"
        />

        <div class="flex flex-col justify-center overflow-hidden">
            <div class="text-sm font-medium truncate">
                {player.currentTrack?.title ?? '未在播放'}
            </div>

            <div class="text-xs opacity-70 truncate">
                {player.currentTrack?.artist ?? '未知艺术家'}
            </div>
        </div>
    </div>
{/snippet}

{#snippet transportControls()}
    <div class="flex items-center gap-2">
        <IconButton
            icon="skip_previous--rounded"
            onclick={() => player.prev()}
        />

        <IconButton
            variant="filled"
            icon={player.playing ? 'pause--rounded' : 'play_arrow--rounded'}
            onclick={() => player.toggle()}
        />

        <IconButton icon="skip_next--rounded" onclick={() => player.next()} />
    </div>
{/snippet}

{#snippet volumeControls()}
    <div class="flex flex-1 items-center min-w-37.5 justify-end gap-1">
        <IconButton
            icon={playModeIcon}
            onclick={() => player.cyclePlayMode()}
            class="opacity-70 hover:opacity-100"
        />

        {#each pluginState.nativeViewExtensions.filter(nv => nv.state === 'Enabled') as nv (nv.pluginId + nv.id)}
            <IconButton
                icon={nv.icon ?? 'extension'}
                onclick={() => toggleNativePanel(nv.pluginId)}
                class={pluginState.activeNativePanel === nv.pluginId ? 'text-[rgb(var(--mdui-color-primary))]' : 'opacity-70 hover:opacity-100'}
            />
        {/each}

        <IconButton
            icon="playlist_play--rounded"
            onclick={() => player.toggleQueue()}
            class={player.queueOpen ? 'text-[rgb(var(--mdui-color-primary))]' : 'opacity-70 hover:opacity-100'}
        />

        <IconButton
            icon={volumeIcon}
            onclick={() => (player.muted = !player.muted)}
            class="text-lg opacity-70 hover:opacity-100"
        />

        <Slider bind:value={player.volume} />
    </div>
{/snippet}

<mdui-bottom-app-bar
    class="relative flex flex-col h-24 px-4 pb-2"
    style="--z-index: 10"
>
    {@render progressArea()}
    <div class="flex items-center w-full h-full mt-2">
        {@render nowPlayingInfo()}
        {@render transportControls()}
        {@render volumeControls()}
    </div>
</mdui-bottom-app-bar>
