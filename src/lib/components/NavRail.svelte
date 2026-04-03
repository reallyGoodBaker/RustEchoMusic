<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/state'
  import { invoke } from '@tauri-apps/api/core'
  import 'mdui/components/navigation-rail-item.js'
  import 'mdui/components/navigation-rail.js'
  import 'mdui/components/snackbar.js'

  const routes = [
    {
      path: '/recent',
      icon: 'watch_later--outlined',
      activeIcon: 'watch_later--filled',
      label: 'Recent',
    },
    {
      path: '/library',
      icon: 'library_music--outlined',
      activeIcon: 'library_music--filled',
      label: 'Library',
    },
    {
      path: '/track',
      icon: 'track_changes--outlined',
      activeIcon: 'track_changes--filled',
      label: 'Track',
    },
    {
      path: '/artist',
      icon: 'person--outlined',
      activeIcon: 'person--filled',
      label: 'Artist',
    },
  ]

  const currentPath = $derived(page.url.pathname)

  function handleKeydown(event: KeyboardEvent, path: string) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      goto(path)
    }
  }

  async function handleAddMusic() {
    try {
      const result = await invoke('select_music_folder')
      const musicFiles = result as Array<{ path: string; name: string }>

      if (musicFiles.length > 0) {
        await invoke('play_music_from_path', { path: musicFiles[0].path })

        const snackbar = document.createElement('mdui-snackbar')
        snackbar.textContent = `已加载 ${musicFiles.length} 首音乐文件`
        snackbar.open = true
      }
    } catch (error) {
      console.error('Failed to select music folder:', error)

      const snackbar = document.createElement('mdui-snackbar')
      snackbar.textContent =
        error instanceof Error ? error.message : 'Failed to load music'
      snackbar.open = true
    }
  }
</script>

<mdui-navigation-rail value={currentPath} style="--z-index: 1">
  <mdui-fab lowered icon="add--outlined" slot="top" onclick={handleAddMusic}
  ></mdui-fab>

  {#each routes as route (route.path)}
    <mdui-navigation-rail-item
      icon={route.icon}
      active-icon={route.activeIcon}
      value={route.path}
      onclick={() => goto(route.path)}
      onkeydown={(e: KeyboardEvent) => handleKeydown(e, route.path)}
      role="button"
      tabindex="0"
    >
      {route.label}
    </mdui-navigation-rail-item>
  {/each}
</mdui-navigation-rail>
