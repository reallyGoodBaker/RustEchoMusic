<script lang="ts">
  import SongCard from './SongCard.svelte'

  let {
    songs,
  }: {
    songs: Array<{
      id: number
      title: string
      artist: string
      album: string
      cover: string
    }>
  } = $props()
</script>

<section>
  <div class="flex flex-col gap-4">
    {#each songs as song (song.id)}
      <SongCard {song} />
    {/each}
  </div>
</section>