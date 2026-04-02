<script lang="ts">
  import ArtistCard from '$lib/components/ArtistCard.svelte'

  const artists = [
    {
      id: 1,
      name: '高桥洋子',
      avatar:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 2,
      name: '宇多田光',
      avatar:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 3,
      name: '初音未来',
      avatar:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 4,
      name: '米玄律师',
      avatar:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
    {
      id: 5,
      name: 'ZARD',
      avatar:
        'https://pic1.zhimg.com/v2-b8472d97dd297753225bdae079343f50_1440w.jpeg',
    },
  ]
</script>

<div class="overflow-y-auto p-8">
  <div class="mb-8">
    <h2 class="text-2xl font-bold mb-4">Artists</h2>
    <div
      class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6"
    >
      {#each artists as artist (artist.id)}
        <ArtistCard {artist} />
      {/each}
    </div>
  </div>
</div>
