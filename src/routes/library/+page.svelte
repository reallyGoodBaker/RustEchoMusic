<script lang="ts">
  import ContextMenu from '$lib/components/ContextMenu.svelte'
  import SectionHeader from '$lib/components/SectionHeader.svelte'
  import SongCard from '$lib/components/SongCard.svelte'
  import { type Track } from '$lib/player.svelte'

  const myPlaylists = [
    {
      id: 1,
      name: 'Favorite Songs',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 2,
      name: 'Anime OSTs',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
  ]

  const librarySongs = [
    {
      id: 1,
      title: '残酷天使的行动纲领',
      artist: '高桥洋子',
      album: 'NEON GENESIS EVANGELION',
      releaseDate: '1995-10-25',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 2,
      title: '魂のルフラン',
      artist: '高桥洋子',
      album: 'NEON GENESIS EVANGELION',
      releaseDate: '1995-10-25',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 3,
      title: 'Beautiful World',
      artist: '宇多田光',
      album: 'Beautiful World/Kiss & Cry',
      releaseDate: '2007-08-29',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 4,
      title: 'One Last Kiss',
      artist: '宇多田光',
      album: 'One Last Kiss',
      releaseDate: '2021-03-09',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 5,
      title: '千本樱',
      artist: '初音未来',
      album: '千本樱',
      releaseDate: '2011-09-17',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
  ]

  let playlistsCollapsed = $state(false)

  let contextMenuConfig = $state<{ x: number; y: number; track: Track } | null>(
    null,
  )

  function handleContextMenu(e: MouseEvent) {
    const target = e.target as HTMLElement
    const card = target.closest('[data-song-id]') as HTMLElement
    if (card) {
      e.preventDefault()
      const id = Number(card.getAttribute('data-song-id'))
      const title = card.getAttribute('data-song-title')!
      const artist = card.getAttribute('data-song-artist')!
      const album = card.getAttribute('data-song-album')!
      const cover = card.getAttribute('data-song-cover')!

      contextMenuConfig = {
        x: e.clientX,
        y: e.clientY,
        track: {
          id,
          title,
          artist,
          album,
          cover,
          path: `music/${artist} - ${title}.flac`,
        },
      }
    }
  }

  // Group by album
  const groupedTracks = librarySongs.reduce(
    (acc, song) => {
      if (!acc[song.album]) {
        acc[song.album] = {
          collapsed: false,
          songs: [],
        }
      }
      acc[song.album].songs.push(song)
      return acc
    },
    {} as Record<string, { collapsed: boolean; songs: typeof librarySongs }>,
  )

  const albums = $state(
    Object.entries(groupedTracks).map(([name, data]) => ({
      name,
      collapsed: data.collapsed,
      songs: data.songs,
    })),
  )
</script>

<div class="overflow-y-auto p-8">
  <div class="mb-8">
    <h2 class="text-3xl font-bold mb-6">Saved Songs</h2>
    <div
      class="flex flex-col gap-8"
      oncontextmenu={handleContextMenu}
      role="presentation"
    >
      {#each albums as album (album.name)}
        <section>
          <SectionHeader title={album.name} bind:collapsed={album.collapsed} />
          {#if !album.collapsed}
            <div
              class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 mt-4 transition-all"
            >
              {#each album.songs as song (song.id)}
                <SongCard {song} />
              {/each}
            </div>
          {/if}
        </section>
      {/each}
    </div>
  </div>
</div>

{#if contextMenuConfig}
  <ContextMenu
    x={contextMenuConfig.x}
    y={contextMenuConfig.y}
    track={contextMenuConfig.track}
    onclose={() => (contextMenuConfig = null)}
  />
{/if}

<style lang="postcss">
  @reference "tailwindcss";
</style>
