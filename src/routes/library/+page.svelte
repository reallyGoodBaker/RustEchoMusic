<script lang="ts">
  import LibraryHeader from '$lib/components/LibraryHeader.svelte'
  import MusicTable from '$lib/components/MusicTable.svelte'
  import { libraryStore, type SongItem } from '$lib/library-store.svelte'
  import { playerState } from '$lib/player.svelte'
  import { selectMusicFolder } from '$lib/services/file'
  import { snackbar } from 'mdui/functions/snackbar.js'

  let filterText = $state('')

  async function loadMusicFromFolder() {
    try {
      const result = await selectMusicFolder()
      const musicFiles = result

      if (musicFiles.length > 0) {
        const newSongs: SongItem[] = musicFiles.map((file, index) => ({
          id: Date.now() + index,
          title: file.title,
          artist: file.artist || 'Unknown Artist',
          album: file.album || 'Unknown Album',
          cover: file.cover_base64 || defaultCover,
          path: file.path,
          duration: file.duration_secs,
          playCount: 0,
        }))

        libraryStore.setSongs(newSongs)

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

  function handlePlay(song: SongItem) {
    const track = libraryStore.toTrack(song)
    playerState.play(track)
    libraryStore.incrementPlayCount(song.id)
  }

  function handleRemove(index: number) {
    libraryStore.removeSong(index)
  }

  const defaultCover =
    'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0OCIgaGVpZ2h0PSI0OCIgdmlld0JveD0iMCAwIDQ4IDQ4Ij48cmVjdCB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIGZpbGw9IiNlMGUwZTAiLz48cGF0aCBkPSJNMjQgMThhNiA2IDAgMSAwIDAgMTIgNiA2IDAgMCAwIDAtMTJ6bTAgOGE0IDQgMCAxIDEgMC04IDQgNCAwIDAgMSAwIDh6IiBmaWxsPSIjYTBiMGIwIi8+PC9zdmc+'
</script>

<div class="flex flex-col h-full">
  <LibraryHeader
    title="音乐库"
    count={libraryStore.count}
    bind:filterText
    onscan={loadMusicFromFolder}
  />

  <div class="flex-1 overflow-y-auto px-8 pb-8">
    {#if libraryStore.songs.length === 0}
      <div
        class="flex flex-col items-center justify-center h-full text-(--controlBright) gap-4 py-20"
      >
        <svg
          class="w-20 h-20 opacity-30"
          fill="none"
          stroke="currentColor"
          stroke-width="1"
          viewBox="0 0 24 24"
        >
          <path
            d="M9 19V6l12-3v13M9 19c0 1.66-1.34 3-3 3S3 20.66 3 19s1.34-3 3-3 3 1.34 3 3zm12-3c0 1.66-1.34 3-3 3s-3-1.34-3-3 1.34-3 3-3 3 1.34 3 3zM9 10l12-3"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <p class="text-base">音乐库为空</p>
        <p class="text-sm opacity-60">点击右上角文件夹图标扫描音乐文件</p>
      </div>
    {:else}
      <MusicTable
        songs={libraryStore.songs}
        {defaultCover}
        {filterText}
        onplay={handlePlay}
        onremove={handleRemove}
      />
    {/if}
  </div>
</div>

<style lang="postcss">
  @reference "tailwindcss";
</style>
