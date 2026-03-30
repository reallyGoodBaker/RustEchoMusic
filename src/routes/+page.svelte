<script lang="ts">
  import Playlists from "$lib/components/Playlists.svelte"
  import RecentSongs from "$lib/components/RecentSongs.svelte"

  const playlists = [
    {
      id: 1,
      name: '我的歌单',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 2,
      name: '收藏的艺术家',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 3,
      name: '收藏的专辑',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 4,
      name: '最近播放',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
  ]

  const recentSongs = [
    {
      id: 1,
      title: '残酷天使的行动纲领',
      artist: '高桥洋子',
      album: 'NEON GENESIS EVANGELION',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 2,
      title: '魂のルフラン',
      artist: '高桥洋子',
      album: 'NEON GENESIS EVANGELION II',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 3,
      title: 'Beautiful World',
      artist: '宇多田光',
      album: 'Beautiful World/Kiss & Cry',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 4,
      title: 'One Last Kiss',
      artist: '宇多田光',
      album: 'One Last Kiss',
      cover:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
  ]
</script>

<div class="overflow-y-auto p-8">
  <Playlists {playlists} />
  <RecentSongs songs={recentSongs} />
</div>

<style lang="postcss">
  @reference "tailwindcss";
  :global(html) {
    background-color: theme(--color-gray-100);
  }
</style>
