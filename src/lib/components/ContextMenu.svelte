<script lang="ts">
  import { portal } from '$lib/actions/portal'
  import { playerState, type Track } from '$lib/player.svelte'
  import { computePosition, flip, offset, shift } from '@floating-ui/dom'
  import 'mdui/components/list-item.js'
  import 'mdui/components/list.js'

  let {
    x = 0,
    y = 0,
    track,
    onclose,
  }: {
    x: number
    y: number
    track: Track
    onclose: () => void
  } = $props()

  let menuElement: HTMLElement | null = $state(null)

  $effect(() => {
    if (menuElement) {
      const virtualEl = {
        getBoundingClientRect() {
          return {
            width: 0,
            height: 0,
            x,
            y,
            top: y,
            left: x,
            right: x,
            bottom: y,
          }
        },
      }

      computePosition(virtualEl as any, menuElement, {
        placement: 'bottom-start',
        middleware: [offset(4), flip(), shift({ padding: 8 })],
      }).then(({ x: newX, y: newY }) => {
        if (menuElement) {
          menuElement.style.left = `${newX}px`
          menuElement.style.top = `${newY}px`
          menuElement.style.opacity = '1'
        }
      })
    }
  })

  function handleOutsideClick(e: MouseEvent) {
    if (menuElement && !menuElement.contains(e.target as Node)) {
      onclose()
    }
  }

  function handleAction(action: string) {
    if (action === 'next') {
      playerState.playlist.push(track)
    }
    onclose()
  }
</script>

<svelte:window
  onmousedown={handleOutsideClick}
  oncontextmenu={handleOutsideClick}
  onkeydown={e => e.key === 'Escape' && onclose()}
/>

<div
  use:portal
  bind:this={menuElement}
  class="fixed z-50 min-w-50 overflow-hidden rounded-xl bg-(--controlBackground) py-2 shadow-xl border border-[var(--controlGray)] opacity-0 transition-opacity"
  style="top: 0; left: 0;"
>
  <mdui-list>
    <mdui-list-item
      icon="queue_music--rounded"
      onclick={() => handleAction('next')}
      onkeydown={(e: KeyboardEvent) =>
        e.key === 'Enter' && handleAction('next')}
      role="menuitem"
      tabindex="0"
    >
      Play Next
    </mdui-list-item>
    <mdui-list-item
      icon="playlist_add--rounded"
      onclick={() => handleAction('add')}
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handleAction('add')}
      role="menuitem"
      tabindex="0"
    >
      Add to Playlist
    </mdui-list-item>
    <mdui-list-item
      icon="favorite_border--rounded"
      onclick={() => handleAction('favorite')}
      onkeydown={(e: KeyboardEvent) =>
        e.key === 'Enter' && handleAction('favorite')}
      role="menuitem"
      tabindex="0"
    >
      Favorite
    </mdui-list-item>
    <mdui-list-item
      icon="delete_outline--rounded"
      onclick={() => handleAction('delete')}
      onkeydown={(e: KeyboardEvent) =>
        e.key === 'Enter' && handleAction('delete')}
      role="menuitem"
      tabindex="0"
    >
      Delete
    </mdui-list-item>
  </mdui-list>
</div>
