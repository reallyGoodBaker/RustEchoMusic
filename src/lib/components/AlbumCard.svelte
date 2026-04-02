<script lang="ts">
  import { goto } from '$app/navigation'
  import 'mdui/components/card.js'

  let {
    album,
  }: {
    album: {
      name: string
      cover: string
      artist: string
      releaseDate?: string
    }
  } = $props()

  let imageLoaded = $state(false)

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      goto(`/album/${encodeURIComponent(album.name)}`)
    }
  }
</script>

<div
  class="group relative h-full transition-all duration-300 hover:-translate-y-1 hover:shadow-xl"
>
  <mdui-card
    variant="elevated"
    class="relative flex w-full cursor-pointer flex-col overflow-hidden rounded-12"
    onclick={() => goto(`/album/${encodeURIComponent(album.name)}`)}
    onkeydown={handleKeydown}
    role="link"
    tabindex="0"
    aria-label={`View album ${album.name} by ${album.artist}`}
  >
    <div class="relative aspect-square w-full overflow-hidden">
      <!-- Skeleton/Placeholder -->
      {#if !imageLoaded}
        <div
          class="absolute inset-0 animate-pulse bg-[var(--controlGray)]"
        ></div>
      {/if}

      <img
        src={album.cover}
        alt={album.name}
        class="absolute inset-0 h-full w-full object-cover transition-opacity duration-300"
        style="opacity: {imageLoaded ? '1' : '0'};"
        loading="lazy"
        onload={() => (imageLoaded = true)}
      />
    </div>

    <div class="flex-1 p-3">
      <h3 class="truncate text-base font-bold" title={album.name}>
        {album.name}
      </h3>
      <p class="truncate text-sm opacity-90" title={album.artist}>
        {album.artist}
      </p>
    </div>
  </mdui-card>
</div>
