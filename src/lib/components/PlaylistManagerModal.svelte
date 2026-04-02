<script lang="ts">
  import { lockScroll } from '$lib/actions/lockScroll'
  import { portal } from '$lib/actions/portal'
  import 'mdui/components/button-icon.js'
  import 'mdui/components/button.js'
  import 'mdui/components/chip.js'
  import 'mdui/components/text-field.js'
  import { cubicOut } from 'svelte/easing'
  import { fade, scale } from 'svelte/transition'

  let { onclose }: { onclose: () => void } = $props()

  function handleOutsideClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose()
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<div
  use:portal
  use:lockScroll
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onmousedown={handleOutsideClick}
  transition:fade={{ duration: 200, easing: cubicOut }}
  role="presentation"
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="relative flex h-[80vh] w-full max-w-lg flex-col rounded-2xl bg-[var(--controlBackground)] shadow-2xl border border-[var(--controlGray)]"
    transition:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    onmousedown={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between border-b border-[var(--controlGray)] p-4">
      <h2 class="text-xl font-bold">Playlists</h2>
      <mdui-button-icon icon="close--rounded" onclick={onclose} onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && onclose()} role="button" tabindex="0"></mdui-button-icon>
    </div>

    <div class="p-4">
      <mdui-text-field icon="search--rounded" placeholder="Search playlists..." variant="outlined" class="w-full"></mdui-text-field>
      <div class="mt-4 flex gap-2">
        <mdui-chip selected>All</mdui-chip>
        <mdui-chip>Favorites</mdui-chip>
        <mdui-chip>Local</mdui-chip>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-4">
      <div class="flex flex-col gap-2">
        {#each Array(10) as _, i}
          <div class="flex items-center justify-between rounded-lg p-2 transition-colors hover:bg-[var(--controlBackgroundHover)]">
            <div class="flex items-center gap-3">
              <div class="h-12 w-12 rounded bg-[var(--controlGray)]"></div>
              <div>
                <div class="font-medium">Playlist {i + 1}</div>
                <div class="text-xs opacity-70">10 songs</div>
              </div>
            </div>
            <mdui-button-icon icon="play_arrow--rounded"></mdui-button-icon>
          </div>
        {/each}
      </div>
    </div>

    <div class="border-t border-[var(--controlGray)] p-4">
      <mdui-button variant="filled" icon="add--rounded" class="w-full">
        New Playlist
      </mdui-button>
    </div>
  </div>
</div>
