<script lang="ts">
  import { goto } from '$app/navigation'
  import { playerState, type Track } from '$lib/player.svelte'
  import AudioEqualizerModal from './AudioEqualizerModal.svelte'
  import PlaylistManagerModal from './PlaylistManagerModal.svelte'
  import Slider from './Slider.svelte'

  let showEqualizer = $state(false)
  let showPlaylist = $state(false)

  export function play(track?: Track) {
    playerState.play(track)
  }
  export function pause() {
    playerState.pause()
  }
  export function toggle() {
    playerState.toggle()
  }
  export function next() {
    playerState.next()
  }
  export function prev() {
    playerState.prev()
  }
  export function toggleMute() {
    playerState.toggleMute()
  }

  const defaultCover =
    'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI2NCIgaGVpZ2h0PSI2NCIgdmlld0JveD0iMCAwIDY0IDY4Ij48cmVjdCB3aWR0aD0iNjQiIGhlaWdodD0iNjQiIGZpbGw9IiNlMGUwZTAiLz48cGF0aCBkPSJNMzIgMjBhMTIgMTIgMCAxIDAgMCAyNCAxMiAxMiAwIDAgMCAwLTI0em0wIDE4YTggOCAwIDEgMSAwLTE2IDggOCAwIDAgMSAwIDE2eiIgZmlsbD0iI2EwYTBhMCIvPjwvc3ZnPg=='

  function handleImageError(e: Event) {
    ;(e.target as HTMLImageElement).src = defaultCover
  }

  function formatDuration(seconds: number): string {
    if (!seconds || seconds <= 0) return '--:--'
    const m = Math.floor(seconds / 60)
    const s = Math.floor(seconds % 60)
    return `${m}:${s.toString().padStart(2, '0')}`
  }
</script>

<mdui-bottom-app-bar
  class="relative h-20 min-w-[320px] px-4"
  style="--z-index: 10; background-color: var(--controlBackground2);"
>
  <Slider
    value={playerState.progress.toString()}
    classList="absolute top-0 left-0 w-full px-2 -translate-y-1/2"
  />

  <div
    class="grid h-full w-full items-center gap-4"
    style="grid-template-columns: minmax(140px, 1fr) auto minmax(120px, 1fr);"
  >
    <!-- Left: Cover and Info -->
    <div class="flex items-center gap-3 overflow-hidden">
      <div
        class="group relative h-16 w-16 shrink-0 overflow-hidden rounded-lg bg-(--controlGray) shadow-sm cursor-pointer ring-2 ring-transparent transition-all hover:ring-primary/40"
        role="link"
        tabindex="0"
        title={playerState.current
          ? `${playerState.current.title} - ${playerState.current.artist}${playerState.current.album !== 'Unknown Album' ? `\n专辑: ${playerState.current.album}` : ''}`
          : ''}
        onclick={() => goto(`/lyrics?trackId=${playerState.current?.id}`)}
        onkeydown={(e: KeyboardEvent) =>
          e.key === 'Enter' &&
          goto(`/lyrics?trackId=${playerState.current?.id}`)}
      >
        <img
          src={playerState.current?.cover || defaultCover}
          alt={playerState.current?.title || 'Cover'}
          class="h-full w-full object-cover transition-opacity group-hover:scale-105"
          loading="lazy"
          onerror={handleImageError}
        />
        {#if playerState.isPlaying}
          <div
            class="absolute inset-0 flex items-center justify-center bg-black/20 opacity-0 transition-opacity group-hover:opacity-100"
          >
            <div
              class="h-6 w-6 rounded-full bg-white/90 flex items-center justify-center"
            >
              <div class="flex gap-0.5 items-end h-3 px-1">
                <span
                  class="w-0.75 bg-primary animate-pulse"
                  style="height: 40%"
                ></span>
                <span
                  class="w-0.75 bg-primary animate-pulse"
                  style="height: 70%; animation-delay: 0.15s"
                ></span>
                <span
                  class="w-0.75 bg-primary animate-pulse"
                  style="height: 50%; animation-delay: 0.3s"
                ></span>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <div class="flex min-w-0 flex-col justify-center">
        <div
          class="truncate text-sm font-medium leading-tight cursor-pointer hover:text-primary transition-colors"
          title={playerState.current?.title ?? '未在播放'}
          onclick={() => goto(`/lyrics?trackId=${playerState.current?.id}`)}
          role="button"
          tabindex="0"
          onkeydown={(e: KeyboardEvent) =>
            e.key === 'Enter' &&
            goto(`/lyrics?trackId=${playerState.current?.id}`)}
        >
          <!-- {#if playerState.isPlaying}
            <span
              class="inline-block w-2 h-2 rounded-full bg-green-500 mr-1.5 align-middle animate-pulse"
            ></span>
          {/if} -->
          {playerState.current?.title ?? '未在播放'}
        </div>
        <div
          class="truncate text-xs opacity-70 mt-0.5 leading-tight"
          title={`${playerState.current?.artist ?? '未知艺术家'}${playerState.current?.album && playerState.current.album !== 'Unknown Album' ? ` · ${playerState.current.album}` : ''}`}
        >
          {playerState.current?.artist ?? '未知艺术家'}
          {#if playerState.current?.album && playerState.current.album !== 'Unknown Album'}
            <span class="opacity-50 mx-1">·</span>
            <span>{playerState.current.album}</span>
          {/if}
        </div>
      </div>
    </div>

    <!-- Center: Main Controls -->
    <div class="flex items-center gap-2 justify-center">
      <mdui-button-icon
        icon="skip_previous--rounded"
        role="button"
        tabindex="0"
        onclick={prev}
        onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && prev()}
      ></mdui-button-icon>
      <mdui-button-icon
        variant="filled"
        icon={playerState.isPlaying ? 'pause--rounded' : 'play_arrow--rounded'}
        role="button"
        tabindex="0"
        onclick={toggle}
        onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && toggle()}
      >
      </mdui-button-icon>
      <mdui-button-icon
        icon="skip_next--rounded"
        role="button"
        tabindex="0"
        onclick={next}
        onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && next()}
      ></mdui-button-icon>
    </div>

    <!-- Right: Additional Controls -->
    <div class="flex items-center justify-end gap-2">
      <mdui-button-icon
        icon={playerState.loopMode === 'one'
          ? 'repeat_one--rounded'
          : playerState.loopMode === 'all'
            ? 'repeat--rounded'
            : playerState.loopMode === 'random'
              ? 'shuffle--rounded'
              : 'repeat--rounded'}
        class={playerState.loopMode === 'none'
          ? 'opacity-50'
          : 'opacity-100 text-primary'}
        role="button"
        tabindex="0"
        onclick={() => playerState.toggleLoopMode()}
        onkeydown={(e: KeyboardEvent) =>
          e.key === 'Enter' && playerState.toggleLoopMode()}
      ></mdui-button-icon>

      <mdui-button-icon
        icon="tune--rounded"
        role="button"
        tabindex="0"
        onclick={() => (showEqualizer = true)}
        onkeydown={(e: KeyboardEvent) =>
          e.key === 'Enter' && (showEqualizer = true)}
      ></mdui-button-icon>

      <mdui-button-icon
        icon="queue_music--rounded"
        role="button"
        tabindex="0"
        onclick={() => (showPlaylist = true)}
        onkeydown={(e: KeyboardEvent) =>
          e.key === 'Enter' && (showPlaylist = true)}
      ></mdui-button-icon>

      <div class="flex items-center w-32 justify-end">
        <mdui-button-icon
          icon={playerState.mute ? 'volume_off--rounded' : 'volume_up--rounded'}
          class="text-lg shrink-0 cursor-pointer"
          role="button"
          tabindex="0"
          onclick={() => toggleMute()}
          onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && toggleMute()}
        ></mdui-button-icon>
        <Slider value="100" classList="w-24" />
      </div>
    </div>
  </div>
</mdui-bottom-app-bar>

{#if showEqualizer}
  <AudioEqualizerModal onclose={() => (showEqualizer = false)} />
{/if}

{#if showPlaylist}
  <PlaylistManagerModal onclose={() => (showPlaylist = false)} />
{/if}
