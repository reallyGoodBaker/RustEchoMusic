<script lang="ts">
  import { playerState, type Track } from '$lib/player.svelte'
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
      // 1. 显示 Popover
      if (!menuElement.matches(':popover-open')) {
        menuElement.showPopover()
      }

      // 2. 原生实现 flip 和 shift（边界防溢出检测）
      const rect = menuElement.getBoundingClientRect()
      const viewportWidth = window.innerWidth
      const viewportHeight = window.innerHeight
      const padding = 8 // 贴边时的安全距离

      let finalX = x + 4 // 模拟 offset(4)
      let finalY = y

      // 处理右侧溢出 (类似 shift)
      if (finalX + rect.width > viewportWidth - padding) {
        finalX = viewportWidth - rect.width - padding
      }
      
      // 处理底部溢出 (类似 flip)
      if (finalY + rect.height > viewportHeight - padding) {
        finalY = y - rect.height - 4 // 向上翻转
        
        // 如果向上翻转后顶部又溢出了，则强制限制在顶部
        if (finalY < padding) {
          finalY = padding
        }
      }

      // 3. 应用计算后的位置并显示
      menuElement.style.left = `${finalX}px`
      menuElement.style.top = `${finalY}px`
      menuElement.style.opacity = '1'
    }
  })

  function handleAction(action: string) {
    if (action === 'next') {
      playerState.playlist.push(track)
    }
    // 关闭时调用原生方法隐藏
    if (menuElement) {
      menuElement.hidePopover()
    }
  }

  // 监听 popover 的原生关闭事件
  function handleToggle(e: ToggleEvent) {
    if (e.newState === 'closed') {
      onclose()
    }
  }
</script>

<div
  popover="auto"
  bind:this={menuElement}
  ontoggle={handleToggle}
  class="fixed m-0 z-50 min-w-50 overflow-hidden rounded-xl bg-(--controlBackground) py-2 shadow-xl border border-(--controlGray) opacity-0 transition-opacity"
>
  <mdui-list>
    <mdui-list-item
      icon="queue_music--rounded"
      onclick={() => handleAction('next')}
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handleAction('next')}
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
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handleAction('favorite')}
      role="menuitem"
      tabindex="0"
    >
      Favorite
    </mdui-list-item>
    <mdui-list-item
      icon="delete_outline--rounded"
      onclick={() => handleAction('delete')}
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handleAction('delete')}
      role="menuitem"
      tabindex="0"
    >
      Delete
    </mdui-list-item>
  </mdui-list>
</div>