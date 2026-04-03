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

  function handleImageError(e: Event) {
    const target = e.target as HTMLImageElement
    target.src =
      'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI2NCIgaGVpZ2h0PSI2NCIgdmlld0JveD0iMCAwIDY0IDY0Ij48cmVjdCB3aWR0aD0iNjQiIGhlaWdodD0iNjQiIGZpbGw9IiNlMGUwZTAiLz48cGF0aCBkPSJNMzIgMjBhMTIgMTIgMCAxIDAgMCAyNCAxMiAxMiAwIDAgMCAwLTI0em0wIDE4YTggOCAwIDEgMSAwLTE2IDggOCAwIDAgMSAwIDE2eiIgZmlsbD0iI2EwYTBhMCIvPjwvc3ZnPg=='
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
    style="grid-template-columns: minmax(120px, 1fr) auto minmax(120px, 1fr);"
  >
    <!-- Left: Cover and Info -->
    <div class="flex items-center gap-3 overflow-hidden">
      <div
        class="h-16 w-16 shrink-0 overflow-hidden rounded-lg bg-(--controlGray) shadow-sm cursor-pointer"
        role="link"
        tabindex="0"
        onclick={() => goto(`/lyrics?trackId=${playerState.current?.id}`)}
        onkeydown={e =>
          e.key === 'Enter' &&
          goto(`/lyrics?trackId=${playerState.current?.id}`)}
      >
        <img
          src={playerState.current?.cover ||
            'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI2NCIgaGVpZ2h0PSI2NCIgdmlld0JveD0iMCAwIDY0IDY0Ij48cmVjdCB3aWR0aD0iNjQiIGhlaWdodD0iNjQiIGZpbGw9IiNlMGUwZTAiLz48cGF0aCBkPSJNMzIgMjBhMTIgMTIgMCAxIDAgMCAyNCAxMiAxMiAwIDAgMCAwLTI0em0wIDE4YTggOCAwIDEgMSAwLTE2IDggOCAwIDAgMSAwIDE2eiIgZmlsbD0iI2EwYTBhMCIvPjwvc3ZnPg=='}
          alt={playerState.current?.title || 'Cover'}
          class="h-full w-full object-cover transition-opacity"
          loading="lazy"
          onerror={handleImageError}
        />
      </div>
      <div class="flex min-w-0 flex-col justify-center">
        <div
          class="truncate text-sm font-medium leading-tight"
          title={playerState.current?.title ?? '未在播放'}
        >
          {playerState.current?.title ?? '未在播放'}
        </div>
        <div
          class="truncate text-xs opacity-70 mt-1 leading-tight"
          title={playerState.current?.artist ?? '未知艺术家'}
        >
          {playerState.current?.artist ?? '未知艺术家'}
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

    <!-- Right: Additional Controls (Volume, etc) -->
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
