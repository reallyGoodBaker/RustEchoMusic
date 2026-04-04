<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/state'
  import { libraryStore, type SongItem } from '$lib/library-store.svelte'
  import { playerState } from '$lib/player.svelte'
  import { selectMusicFolder } from '$lib/services/file'
  import { type Track } from '$lib/types/tracks'
  import { snackbar } from 'mdui/functions/snackbar.js'

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
      path: '/lyrics',
      icon: 'lyrics--outlined',
      activeIcon: 'lyrics--filled',
      label: 'Lyrics',
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
      const result = await selectMusicFolder()
      const musicFiles = result as unknown as Array<{
        path: string
        title: string
        artist: string
        album: string
        duration_secs: number
        cover_base64: string
      }>

      if (musicFiles.length > 0) {
        const newSongs: SongItem[] = musicFiles.map((file, index) => ({
          id: Date.now() + index,
          title: file.title,
          artist: file.artist || 'Unknown Artist',
          album: file.album || 'Unknown Album',
          cover: file.cover_base64 || '',
          path: file.path,
          duration: file.duration_secs,
          playCount: 0,
        }))

        libraryStore.setSongs(newSongs)

        const newTracks: Track[] = newSongs.map(song => ({
          id: song.id,
          title: song.title,
          artist: song.artist,
          album: song.album,
          cover: song.cover,
          path: song.path,
          duration: song.duration,
        }))

        playerState.playlist = newTracks

        snackbar({
          message: `已加载 ${musicFiles.length} 首音乐文件`,
        })
      }
    } catch (error) {
      console.error('Failed to select music folder:', error)

      snackbar({
        message:
          error instanceof Error ? error.message : 'Failed to load music',
      })
    }
  }
</script>

<mdui-navigation-rail value={currentPath} style="--z-index: 1">
  <mdui-fab
    lowered
    icon="add--outlined"
    slot="top"
    onclick={handleAddMusic}
    onkeydown={(e: KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') handleAddMusic()
    }}
    role="button"
    tabindex="0"
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
