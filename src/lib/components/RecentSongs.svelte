<script lang="ts">
  import SongCard from './SongCard.svelte'

  export let songs: Array<{
    id: number
    title: string
    artist: string
    album: string
    cover: string
  }>
</script>

<section>
  <div class="flex flex-col gap-4">
    {#each songs as song}
      <SongCard {song} />
    {/each}
  </div>
</section>