<script lang="ts">
  import { playerState } from '$lib/player.svelte'
  import { deleteFile, openContainingFolder } from '$lib/services/file'
  import { type Track } from '$lib/types/tracks'
  import { withErrorHandling } from '$lib/utils/error-handler'
  import 'mdui/components/button.js'
  import 'mdui/components/dialog.js'
  import 'mdui/components/list-item.js'
  import 'mdui/components/list.js'
  import { snackbar } from 'mdui/functions/snackbar.js'
  import { fade, scale } from 'svelte/transition'

  let {
    x = 0,
    y = 0,
    track,
    songIndex = -1,
    onclose,
    onremovefromlibrary,
  }: {
    x: number
    y: number
    track: Track
    songIndex?: number
    onclose: () => void
    onremovefromlibrary?: (index: number) => void
  } = $props()

  let menuElement: HTMLElement | null = $state(null)
  let showDeleteDialog = $state(false)

  $effect(() => {
    if (menuElement) {
      if (!menuElement.matches(':popover-open')) {
        menuElement.showPopover()
      }

      const rect = menuElement.getBoundingClientRect()
      const viewportWidth = window.innerWidth
      const viewportHeight = window.innerHeight
      const padding = 8

      let finalX = x + 4
      let finalY = y

      if (finalX + rect.width > viewportWidth - padding) {
        finalX = viewportWidth - rect.width - padding
      }

      if (finalY + rect.height > viewportHeight - padding) {
        finalY = y - rect.height - 4

        if (finalY < padding) {
          finalY = padding
        }
      }

      menuElement.style.left = `${finalX}px`
      menuElement.style.top = `${finalY}px`
      menuElement.style.opacity = '1'
    }
  })

  function closeMenu() {
    if (menuElement) {
      menuElement.hidePopover()
    }
    onclose()
  }

  async function handleAddToPlaylist() {
    playerState.insertNext(track)
    snackbar({ message: `已将 "${track.title}" 添加到播放队列下一首` })
    closeMenu()
  }

  function handleShowDeleteConfirm() {
    showDeleteDialog = true
  }

  async function handleDeleteFile() {
    const result = await withErrorHandling(
      async () => {
        await deleteFile(track.path)
        if (onremovefromlibrary && songIndex >= 0) {
          onremovefromlibrary(songIndex)
        }
        return true
      },
      `已删除文件 "${track.title}"`,
      '删除文件失败',
    )

    if (result !== null) {
      showDeleteDialog = false
      closeMenu()
    }
  }

  async function handleOpenFolder() {
    const result = await withErrorHandling(
      async () => {
        await openContainingFolder(track.path)
        return true
      },
      undefined,
      '无法打开文件夹',
    )

    if (result !== null) {
      closeMenu()
    }
  }

  function handleRemoveFromLibrary() {
    if (onremovefromlibrary && songIndex >= 0) {
      onremovefromlibrary(songIndex)
      snackbar({ message: `已从音乐库移除 "${track.title}"` })
    }
    closeMenu()
  }

  function handleToggle(e: ToggleEvent) {
    if (e.newState === 'closed') {
      onclose()
    }
  }
</script>

<svelte:window onkeydown={e => e.key === 'Escape' && closeMenu()} />

<div
  popover="hint"
  bind:this={menuElement}
  ontoggle={handleToggle}
  class="fixed m-0 z-50 min-w-56 overflow-hidden rounded-xl bg-(--controlBackground) py-2 shadow-xl border border-(--controlGray) opacity-0 transition-opacity"
>
  <mdui-list>
    <mdui-list-item
      icon="playlist_add--outlined"
      onclick={handleAddToPlaylist}
      onkeydown={(e: KeyboardEvent) =>
        e.key === 'Enter' && handleAddToPlaylist()}
      role="menuitem"
      tabindex="0"
    >
      添加到播放列表（下一首）
    </mdui-list-item>

    <mdui-list-item
      icon="folder_open--outlined"
      onclick={handleOpenFolder}
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handleOpenFolder()}
      role="menuitem"
      tabindex="0"
    >
      打开所在文件夹
    </mdui-list-item>

    <mdui-list-item
      icon="remove_circle_outline--outlined"
      onclick={handleRemoveFromLibrary}
      onkeydown={(e: KeyboardEvent) =>
        e.key === 'Enter' && handleRemoveFromLibrary()}
      role="menuitem"
      tabindex="0"
    >
      从音乐库中移除
    </mdui-list-item>

    <div class="my-1 border-t border-(--controlGray)/50"></div>

    <mdui-list-item
      icon="delete_outline--rounded"
      class="text-red-500 hover:bg-red-500/10"
      onclick={handleShowDeleteConfirm}
      onkeydown={(e: KeyboardEvent) =>
        e.key === 'Enter' && handleShowDeleteConfirm()}
      role="menuitem"
      tabindex="0"
    >
      删除文件
    </mdui-list-item>
  </mdui-list>
</div>

{#if showDeleteDialog}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-60 flex items-center justify-center bg-black/50"
    onmousedown={() => (showDeleteDialog = false)}
    transition:fade={{ duration: 150 }}
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="relative w-full max-w-sm rounded-2xl bg-(--controlBackground) p-6 shadow-2xl border border-(--controlGray)"
      onpointerdown={e => e.stopPropagation()}
      onkeydown={e => e.stopPropagation()}
      transition:scale={{ duration: 150, start: 0.95 }}
      role="alertdialog"
      aria-label="确认删除"
      tabindex="0"
    >
      <h3 class="text-lg font-bold mb-2">确认删除文件？</h3>
      <p class="text-sm text-(--controlBright) mb-6">
        此操作将永久删除文件<br />
        <span class="font-medium text-(--controlDark)">
          {track.title} - {track.artist}
        </span><br />
        且无法撤销。
      </p>

      <div class="flex justify-end gap-3">
        <mdui-button
          variant="text"
          onclick={() => (showDeleteDialog = false)}
          onkeydown={(e: KeyboardEvent) =>
            e.key === 'Enter' && (showDeleteDialog = false)}
          role="button"
          tabindex="0"
        >
          取消
        </mdui-button>
        <mdui-button
          variant="filled"
          class="bg-red-500 hover:bg-red-600 text-white"
          onclick={handleDeleteFile}
          onkeydown={(e: KeyboardEvent) =>
            e.key === 'Enter' && handleDeleteFile()}
          role="button"
          tabindex="0"
        >
          确认删除
        </mdui-button>
      </div>
    </div>
  </div>
{/if}

<style lang="postcss">
  @reference "tailwindcss";
</style>
