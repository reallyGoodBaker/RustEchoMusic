<script lang="ts">
  import type { SongItem } from '$lib/library-store.svelte'
  import { playerState, type Track } from '$lib/player.svelte'
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

  function handleSongDblClick(song: SongItem) {
    onplay(song)
  }

  function handleRemoveFromLibrary(index: number) {
    onremove?.(index)
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
      return [{ name: '', songs: sortedAndFilteredSongs() }]
    }

    const groups: Record<
      string,
      { name: string; year?: string; songs: SongItem[] }
    > = {}

    for (const song of sortedAndFilteredSongs()) {
      if (!groups[song.album]) {
        const yearMatch = song.releaseDate?.match(/^(\d{4})/)
        groups[song.album] = {
          name: song.album,
          year: yearMatch ? yearMatch[1] : undefined,
          songs: [],
        }
      }
      groups[song.album].songs.push(song)
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
          class="w-14 pl-4 py-3 font-medium cursor-pointer hover:text-primary transition-colors"
          onclick={() => handleSort('title')}
        >
          封面
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
      {#each groupedByAlbum() as album (album.name)}
        {#if album.name && showAlbumGrouping}
          <tr class="group/album">
            <td colspan="6" class="px-0 py-0">
              <div
                class="flex items-center gap-3 px-4 py-2 bg-(--controlGray)/30 border-b border-(--controlGray)/50"
              >
                <span
                  class="font-medium text-sm text-(--controlDark) truncate flex-1"
                >
                  {album.name}
                </span>
                {#if album.year}
                  <span class="text-xs text-(--controlBlack) tabular-nums">
                    {album.year}
                  </span>
                {/if}
              </div>
            </td>
          </tr>
        {/if}

        {#each album.songs as song (song.id)}
          {@const globalIndex = songs.findIndex(s => s.id === song.id)}
          <tr
            class={`group/song border-b border-(--controlGray)/20 transition-colors cursor-pointer hover:bg-(--controlBackgroundHover) ${playerState.current?.id === song.id && playerState.isPlaying ? 'bg-primary/5' : ''}`}
            ondblclick={() => handleSongDblClick(song)}
            oncontextmenu={e => handleContextMenu(e, song, globalIndex)}
            data-song-id={song.id}
            data-song-title={song.title}
            data-song-artist={song.artist}
            data-song-album={song.album}
            data-song-cover={song.cover}
          >
            <td class="pl-4 py-2">
              <div
                class="h-11 w-11 overflow-hidden rounded-md bg-(--controlGray) shrink-0"
              >
                <img
                  src={song.cover || defaultCover}
                  alt={song.title}
                  class="h-full w-full object-cover group-hover/song:scale-105 transition-transform"
                  loading="lazy"
                />
              </div>
            </td>
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
            <td class="pr-2 py-2">
              <mdui-button-icon
                icon="more_vert--outlined"
                class="opacity-0 group-hover/song:opacity-100 transition-opacity text-(--controlBlack)"
                role="button"
                tabindex="0"
                oncontextmenu={(e: PointerEvent) => {
                  e.preventDefault()
                  handleContextMenu(e, song, globalIndex)
                }}
                onkeydown={(e: KeyboardEvent) => {
                  if (e.key === 'Enter') {
                    handleContextMenu(
                      e as unknown as PointerEvent,
                      song,
                      globalIndex,
                    )
                  }
                }}
              ></mdui-button-icon>
            </td>
          </tr>
        {/each}
      {/each}
    </tbody>
  </table>
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
</style>
