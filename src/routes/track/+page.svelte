<script lang="ts">
  import AlbumCard from '$lib/components/AlbumCard.svelte'

  const trackSongs = [
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

  const groupedTracks = trackSongs.reduce(
    (acc, song) => {
      if (!acc[song.album]) {
        acc[song.album] = {
          name: song.album,
          artist: song.artist,
          cover: song.cover,
          songs: [],
        }
      }
      acc[song.album].songs.push(song)
      return acc
    },
    {} as Record<
      string,
      { name: string; artist: string; cover: string; songs: typeof trackSongs }
    >,
  )

  const albums = $state(Object.values(groupedTracks))
</script>

<div class="overflow-y-auto p-8">
  <div class="mb-8">
    <h2 class="text-3xl font-bold mb-6">All Albums</h2>

    {#if albums.length > 0}
      <div
        class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
      >
        {#each albums as album (album.name)}
          <AlbumCard {album} />
        {/each}
      </div>
    {:else}
      <div
        class="flex flex-col items-center justify-center py-16 text-[var(--controlGray)]"
      >
        <mdui-icon name="album--outlined" size="64"></mdui-icon>
        <p class="mt-4 text-lg">暂无专辑数据</p>
      </div>
    {/if}
  </div>
</div>

<style lang="postcss">
  @reference "tailwindcss";
</style>
