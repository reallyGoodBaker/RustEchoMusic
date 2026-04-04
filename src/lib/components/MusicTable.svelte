<script lang="ts">
  import type { SongItem } from '$lib/library-store.svelte'
  import { playerState } from '$lib/player.svelte'
  import { type Track } from '$lib/types/tracks'
  import ContextMenu from './ContextMenu.svelte'

  let {
    songs,
    onplay,
    onremove,
    showAlbumGrouping = true,
    defaultCover,
    filterText = '',
  }: {
    songs: SongItem[]
    onplay: (song: SongItem) => void
    onremove?: (index: number) => void
    showAlbumGrouping?: boolean
    defaultCover: string
    filterText?: string
  } = $props()

  let contextMenuConfig = $state<{
    x: number
    y: number
    track: Track
    songIndex: number
  } | null>(null)

  let sortField = $state<
    'title' | 'artist' | 'album' | 'duration' | 'playCount'
  >('title')
  let sortAsc = $state(true)

  let expandedAlbums = $state<Set<string>>(new Set())
  let selectedSongIds = $state<Set<number>>(new Set())

  function toggleAlbumExpansion(albumName: string) {
    const newExpanded = new Set(expandedAlbums)
    if (newExpanded.has(albumName)) {
      newExpanded.delete(albumName)
    } else {
      newExpanded.add(albumName)
    }
    expandedAlbums = newExpanded
  }

  function selectAllSongsInAlbum(albumSongs: SongItem[]) {
    const newSelected = new Set(selectedSongIds)
    albumSongs.forEach(song => {
      newSelected.add(song.id)
    })
    selectedSongIds = newSelected
  }

  function isAlbumExpanded(albumName: string): boolean {
    return expandedAlbums.has(albumName)
  }

  function isSongSelected(songId: number): boolean {
    return selectedSongIds.has(songId)
  }

  function handleAlbumClick(
    e: MouseEvent,
    albumName: string,
    albumSongs: SongItem[],
  ) {
    if (e.detail === 1) {
      selectAllSongsInAlbum(albumSongs)
    }
  }

  function handleAlbumDblClick(e: MouseEvent, albumName: string) {
    e.stopPropagation()
    toggleAlbumExpansion(albumName)
  }

  function handleContextMenu(e: MouseEvent, song: SongItem, index: number) {
    e.preventDefault()
    const track: Track = {
      id: song.id,
      title: song.title,
      artist: song.artist,
      album: song.album,
      cover: song.cover,
      path: song.path,
      duration: song.duration,
    }
    contextMenuConfig = {
      x: e.clientX,
      y: e.clientY,
      track,
      songIndex: index,
    }
  }

  function handleSort(field: typeof sortField) {
    if (sortField === field) {
      sortAsc = !sortAsc
    } else {
      sortField = field
      sortAsc = true
    }
  }

  function formatDuration(seconds?: number): string {
    if (!seconds || seconds <= 0) return '--:--'
    const m = Math.floor(seconds / 60)
    const s = Math.floor(seconds % 60)
    return `${m}:${s.toString().padStart(2, '0')}`
  }

  async function handleSongDblClick(song: SongItem, albumSongs: SongItem[]) {
    try {
      const albumTracks: Track[] = albumSongs.map(s => ({
        id: s.id,
        title: s.title,
        artist: s.artist,
        album: s.album,
        cover: s.cover,
        path: s.path,
        duration: s.duration,
      }))

      playerState.setPlaylist(albumTracks)

      const currentTrack: Track = {
        id: song.id,
        title: song.title,
        artist: song.artist,
        album: song.album,
        cover: song.cover,
        path: song.path,
        duration: song.duration,
      }

      await playerState.play(currentTrack)
      onplay(song)
    } catch (error) {
      console.error('Failed to play song:', error)
      playerState.error = error instanceof Error ? error.message : String(error)
    }
  }

  async function playAlbum(albumSongs: SongItem[]) {
    if (albumSongs.length === 0) return

    try {
      const albumTracks: Track[] = albumSongs.map(s => ({
        id: s.id,
        title: s.title,
        artist: s.artist,
        album: s.album,
        cover: s.cover,
        path: s.path,
        duration: s.duration,
      }))

      playerState.setPlaylist(albumTracks)
      await playerState.play(
        albumTracks[0]
          ? {
              id: albumTracks[0].id,
              title: albumTracks[0].title,
              artist: albumTracks[0].artist,
              album: albumTracks[0].album,
              cover: albumTracks[0].cover,
              path: albumTracks[0].path,
              duration: albumTracks[0].duration,
            }
          : undefined,
      )
    } catch (error) {
      console.error('Failed to play album:', error)
      playerState.error = error instanceof Error ? error.message : String(error)
    }
  }

  function handleRemoveFromLibrary(index: number) {
    onremove?.(index)
  }

  function getAlbumCover(albumSongs: SongItem[]): string {
    const songWithCover = albumSongs.find(
      s => s.cover && s.cover !== defaultCover,
    )
    return songWithCover?.cover || defaultCover
  }

  const sortedAndFilteredSongs = $derived(() => {
    let result = [...songs]

    if (filterText.trim()) {
      const query = filterText.toLowerCase()
      result = result.filter(
        s =>
          s.title.toLowerCase().includes(query) ||
          s.artist.toLowerCase().includes(query) ||
          s.album.toLowerCase().includes(query),
      )
    }

    result.sort((a, b) => {
      let cmp = 0
      switch (sortField) {
        case 'title':
          cmp = a.title.localeCompare(b.title)
          break
        case 'artist':
          cmp = a.artist.localeCompare(b.artist)
          break
        case 'album':
          cmp = a.album.localeCompare(b.album)
          break
        case 'duration':
          cmp = (a.duration ?? 0) - (b.duration ?? 0)
          break
        case 'playCount':
          cmp = (a.playCount ?? 0) - (b.playCount ?? 0)
          break
      }
      return sortAsc ? cmp : -cmp
    })

    return result
  })

  const groupedByAlbum = $derived(() => {
    if (!showAlbumGrouping) {
      return [{ name: '', songs: sortedAndFilteredSongs(), cover: '' }]
    }

    const groups: Record<
      string,
      { name: string; year?: string; songs: SongItem[]; cover: string }
    > = {}

    for (const song of sortedAndFilteredSongs()) {
      if (!groups[song.album]) {
        const yearMatch = song.releaseDate?.match(/^(\d{4})/)
        groups[song.album] = {
          name: song.album,
          year: yearMatch ? yearMatch[1] : undefined,
          songs: [],
          cover: '',
        }
      }
      groups[song.album].songs.push(song)
    }

    for (const key of Object.keys(groups)) {
      groups[key].cover = getAlbumCover(groups[key].songs)
    }

    return Object.values(groups)
  })
</script>

<div
  class="border border-(--controlGray) rounded-xl overflow-hidden bg-(--controlBackground)"
  role="grid"
  aria-label="音乐列表"
>
  <table class="w-full text-sm">
    <thead>
      <tr
        class="border-b border-(--controlGray) text-left text-xs text-(--controlBlack) uppercase tracking-wider"
      >
        <th
          class="w-20 pl-4 py-3 font-medium cursor-pointer hover:text-primary transition-colors"
          onclick={() => handleSort('album')}
        >
          专辑
        </th>
        {#snippet SortHeader({
          field,
          label,
          align = 'left',
          width,
        }: {
          field: typeof sortField
          label: string
          align?: 'left' | 'right'
          width?: string
        })}
          <th
            class="py-3 font-medium cursor-pointer hover:text-primary transition-colors {align ===
            'right'
              ? 'text-right pr-4'
              : ''} {width ?? ''}"
            onclick={() => handleSort(field)}
          >
            <span class:inline-block={sortField === field}>
              {label}
              {#if sortField === field}
                <span class="ml-1">{sortAsc ? '↑' : '↓'}</span>
              {/if}
            </span>
          </th>
        {/snippet}
        {@render SortHeader({ field: 'title', label: '标题' })}
        {@render SortHeader({
          field: 'artist',
          label: '艺术家',
          width: 'min-w-[140px]',
        })}
        {@render SortHeader({
          field: 'playCount',
          label: '播放次数',
          align: 'right',
          width: 'w-24',
        })}
        {@render SortHeader({
          field: 'duration',
          label: '时间',
          align: 'right',
          width: 'w-20',
        })}
        <th class="w-10 pr-2"></th>
      </tr>
    </thead>
    <tbody>
      {#each groupedByAlbum() as album (album.name || 'ungrouped')}
        {@const albumKey = album.name || 'ungrouped'}
        {@const shouldShow = showAlbumGrouping && album.name}
        {@const songCount =
          shouldShow && isAlbumExpanded(albumKey)
            ? album.songs.length
            : !shouldShow
              ? album.songs.length
              : 0}

        {#if shouldShow}
          <tr
            class={`group/album cursor-pointer transition-colors hover:bg-(--controlGray)/20 ${isAlbumExpanded(albumKey) ? 'bg-primary/5' : ''}`}
            onclick={e => handleAlbumClick(e, albumKey, album.songs)}
            ondblclick={e => handleAlbumDblClick(e, albumKey)}
            role="button"
            tabindex="0"
            aria-expanded={isAlbumExpanded(albumKey)}
            data-album-name={album.name}
          >
            <td colspan="6" class="px-0 py-0">
              <div
                class="flex items-center gap-3 px-4 py-2 bg-(--controlGray)/30 border-b border-(--controlGray)/50"
              >
                <mdui-icon-expand-more
                  class={`transition-transform duration-200 text-(--controlBlack) ${isAlbumExpanded(albumKey) ? 'rotate-180' : ''}`}
                ></mdui-icon-expand-more>
                <span class="font-medium text-sm text-primary truncate flex-1">
                  {album.name}
                </span>
                {#if album.year}
                  <span class="text-xs text-(--controlBlack) tabular-nums">
                    {album.year}
                  </span>
                {/if}
                <mdui-button-icon
                  icon="play_arrow--outlined"
                  class="opacity-0 group-hover/album:opacity-100 transition-opacity text-primary"
                  role="button"
                  tabindex="0"
                  onclick={(e: MouseEvent) => {
                    e.stopPropagation()
                    playAlbum(album.songs)
                  }}
                  onkeydown={(e: KeyboardEvent) => {
                    if (e.key === 'Enter') {
                      e.stopPropagation()
                      playAlbum(album.songs)
                    }
                  }}
                ></mdui-button-icon>
              </div>
            </td>
          </tr>
        {/if}

        {#if !shouldShow || isAlbumExpanded(albumKey)}
          {#each album.songs as song, index (song.id)}
            {@const globalIndex = songs.findIndex(s => s.id === song.id)}
            {@const isFirstSong = index === 0}
            <tr
              class={`group/song border-b border-(--controlGray)/20 transition-colors hover:bg-(--controlBackgroundHover) ${playerState.current?.id === song.id && playerState.isPlaying ? 'bg-primary/5' : ''} ${isSongSelected(song.id) ? 'bg-primary/10' : ''}`}
              ondblclick={() => handleSongDblClick(song, album.songs)}
              oncontextmenu={e => handleContextMenu(e, song, globalIndex)}
              data-song-id={song.id}
              data-song-title={song.title}
              data-song-artist={song.artist}
              data-song-album={song.album}
              data-song-cover={song.cover}
            >
              {#if shouldShow && isFirstSong}
                <td
                  rowspan={album.songs.length}
                  class="pl-4 py-2 w-20 align-top"
                >
                  <div
                    class="w-16 h-16 overflow-hidden rounded-lg bg-(--controlGray) shrink-0 shadow-sm"
                  >
                    <img
                      src={album.cover}
                      alt={album.name}
                      class="h-full w-full object-cover group-hover/song:scale-105 transition-transform"
                      loading="lazy"
                    />
                  </div>
                </td>
              {/if}
              <td class="py-2 pr-2">
                <div
                  class={`truncate max-w-75 ${playerState.current?.id === song.id ? 'text-primary font-medium' : ''}`}
                >
                  {song.title}
                </div>
              </td>
              <td class="py-2 pr-2 text-(--controlBlack) text-xs">
                <div class="truncate max-w-50">{song.artist}</div>
              </td>
              <td
                class="py-2 pr-4 text-right text-(--controlBlack) text-xs tabular-nums"
              >
                {song.playCount ?? 0}
              </td>
              <td
                class="py-2 pr-4 text-right text-(--controlBlack) text-xs tabular-nums whitespace-nowrap"
              >
                {formatDuration(song.duration)}
              </td>
            </tr>
          {/each}
        {/if}
      {/each}
    </tbody>
  </table>

  {#if groupedByAlbum().length === 0 && songs.length > 0}
    <div
      class="flex flex-col items-center justify-center py-12 text-(--controlBright)"
    >
      <p class="text-sm">没有匹配的歌曲</p>
      <p class="text-xs mt-1 opacity-60">尝试调整筛选条件</p>
    </div>
  {/if}
</div>

{#if contextMenuConfig}
  <ContextMenu
    x={contextMenuConfig.x}
    y={contextMenuConfig.y}
    track={contextMenuConfig.track}
    songIndex={contextMenuConfig.songIndex}
    onclose={() => (contextMenuConfig = null)}
    onremovefromlibrary={index => handleRemoveFromLibrary(index)}
  />
{/if}

<style lang="postcss">
  @reference "tailwindcss";

  :global(mdui-icon-expand-more) {
    font-size: 18px;
  }
</style>
