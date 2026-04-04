<script lang="ts">
  import { lockScroll } from '$lib/actions/lockScroll'
  import { portal } from '$lib/actions/portal'
  import { playerState } from '$lib/player.svelte'
  import { type Track } from '$lib/types/tracks'
  import 'mdui/components/button-icon.js'
  import 'mdui/components/chip.js'
  import { cubicOut } from 'svelte/easing'
  import { fade, scale } from 'svelte/transition'

  let { onclose }: { onclose: () => void } = $props()

  const defaultCover =
    'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI2NCIgaGVpZ2h0PSI2NCIgdmlld0JveD0iMCAwIDY0IDY0Ij48cmVjdCB3aWR0aD0iNjQiIGhlaWdodD0iNjQiIGZpbGw9IiNlMGUwZTAiLz48cGF0aCBkPSJNMzIgMjBhMTIgMTIgMCAxIDAgMCAyNCAxMiAxMiAwIDAgMCAwLTI0em0wIDE4YTggOCAwIDEgMSAwLTE2IDggOCAwIDAgMSAwIDE2eiIgZmlsbD0iI2EwYTBhMCIvPjwvc3ZnPg=='

  function handleOutsideClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose()
    }
  }

  function handleImageError(e: Event) {
    ;(e.target as HTMLImageElement).src = defaultCover
  }

  function formatDuration(seconds: number): string {
    if (!seconds || seconds <= 0) return '--:--'
    const m = Math.floor(seconds / 60)
    const s = Math.floor(seconds % 60)
    return `${m}:${s.toString().padStart(2, '0')}`
  }

  function handlePlayTrack(track: Track) {
    playerState.play(track)
  }

  function handleRemoveTrack(index: number) {
    const newPlaylist = [...playerState.playlist]
    newPlaylist.splice(index, 1)
    playerState.playlist = newPlaylist
  }

  function isCurrentTrack(songId: number): boolean {
    return playerState.current?.id === songId
  }
</script>

<svelte:window onkeydown={e => e.key === 'Escape' && onclose()} />

<div
  use:portal
  use:lockScroll
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onmousedown={handleOutsideClick}
  transition:fade={{ duration: 200, easing: cubicOut }}
  role="dialog"
  aria-label="播放列表"
  tabindex="0"
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="relative flex h-[80vh] w-full max-w-lg flex-col rounded-2xl bg-(--controlBackground) shadow-2xl border border-(--controlGray) overflow-hidden"
    transition:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    onmousedown={e => e.stopPropagation()}
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between border-b border-(--controlGray) p-4 shrink-0"
    >
      <div class="flex items-center gap-3">
        <h2 class="text-xl font-bold">播放列表</h2>
        <span
          class="text-xs text-(--controlBright) font-medium px-2 py-0.5 rounded-full bg-(--controlBlackAcrylic)"
        >
          {playerState.playlist.length} 首
        </span>
      </div>
      <mdui-button-icon
        icon="close--rounded"
        onclick={onclose}
        onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && onclose()}
        role="button"
        tabindex="0"
        aria-label="关闭"
      ></mdui-button-icon>
    </div>

    <!-- Playlist Content -->
    <div class="flex-1 overflow-y-auto">
      {#if playerState.playlist.length === 0}
        <div
          class="flex flex-col items-center justify-center h-full text-(--controlBright) gap-3 p-8"
        >
          <svg
            class="w-16 h-16 opacity-40"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path
              d="M9 18V5l12-2v13"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
            <circle cx="6" cy="18" r="3" />
            <circle cx="18" cy="16" r="3" />
          </svg>
          <p class="text-sm">播放列表为空</p>
          <p class="text-xs opacity-60">添加音乐文件开始播放</p>
        </div>
      {:else}
        <div class="flex flex-col divide-y divide-(--controlGray)/30">
          {#each playerState.playlist as song, i (song.id)}
            <div
              class={`group flex items-center justify-between gap-2 px-4 py-2.5 transition-colors cursor-pointer hover:bg-(--controlBackgroundHover) ${isCurrentTrack(song.id) ? 'bg-primary/5' : ''}`}
              onclick={() => handlePlayTrack(song)}
              role="button"
              tabindex="0"
              onkeydown={(e: KeyboardEvent) =>
                e.key === 'Enter' && handlePlayTrack(song)}
            >
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <!-- Index / Playing Indicator -->
                <div class="w-6 text-center shrink-0">
                  {#if isCurrentTrack(song.id) && playerState.isPlaying}
                    <div class="flex gap-0.5 items-end h-3 justify-center">
                      <span
                        class="w-0.5 bg-primary animate-pulse"
                        style="height: 40%"
                      ></span>
                      <span
                        class="w-0.5 bg-primary animate-pulse"
                        style="height: 70%; animation-delay: 0.15s"
                      ></span>
                      <span
                        class="w-0.5 bg-primary animate-pulse"
                        style="height: 50%; animation-delay: 0.3s"
                      ></span>
                    </div>
                  {:else}
                    <span
                      class="text-xs text-(--controlBright) group-hover:hidden"
                      >{i + 1}</span
                    >
                  {/if}
                </div>

                <!-- Cover -->
                <div
                  class="h-11 w-11 shrink-0 overflow-hidden rounded-md bg-(--controlGray)"
                >
                  <img
                    src={song.cover || defaultCover}
                    alt={song.title}
                    class="h-full w-full object-cover transition-transform group-hover:scale-105"
                    loading="lazy"
                    onerror={handleImageError}
                  />
                </div>

                <!-- Info -->
                <div class="min-w-0 flex-1">
                  <div
                    class={`truncate text-sm font-medium leading-tight ${isCurrentTrack(song.id) ? 'text-primary' : ''}`}
                    title={song.title}
                  >
                    {song.title}
                  </div>
                  <div class="truncate text-xs opacity-60 mt-0.5 leading-tight">
                    {song.artist}
                    {#if song.album !== 'Unknown Album'}
                      <span class="opacity-40 mx-1">·</span>
                      <span>{song.album}</span>
                    {/if}
                  </div>
                </div>

                <!-- Duration -->
                <div
                  class="text-xs text-(--controlBright) shrink-0 hidden sm:block"
                >
                  {formatDuration(0)}
                </div>
              </div>

              <!-- Remove Button -->
              <mdui-button-icon
                icon="close--outlined"
                class="opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                role="button"
                tabindex="0"
                onclick={(e: MouseEvent) => {
                  e.stopPropagation()
                  handleRemoveTrack(i)
                }}
                onkeydown={(e: KeyboardEvent) => {
                  e.key === 'Enter' && (e.target as HTMLElement).click()
                }}
                aria-label={`移除 ${song.title}`}
              ></mdui-button-icon>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Footer -->
    {#if playerState.playlist.length > 0}
      <div class="border-t border-(--controlGray) p-4 shrink-0">
        <div
          class="flex items-center justify-between text-xs text-(--controlBright)"
        >
          <span>共 {playerState.playlist.length} 首歌曲</span>
          <button
            class="hover:text-red-500 transition-colors cursor-pointer"
            onclick={() => {
              playerState.playlist = []
            }}
          >
            清空列表
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>
