<script lang="ts">
  import PlaylistCard from './PlaylistCard.svelte'

  export let playlists: Array<{
    id: number
    name: string
    cover: string
  }>
</script>

<section class="mb-12">
  <div
    class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6"
  >
    {#each playlists as playlist}
      <PlaylistCard {playlist} />
    {/each}
  </div>
</section>