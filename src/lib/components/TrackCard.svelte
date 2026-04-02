<script lang="ts">
  import { goto } from '$app/navigation'
  import 'mdui/components/card.js'

  let {
    track,
  }: {
    track: {
      id: number
      title: string
      artist: string
      album: string
      cover: string
      releaseDate?: string
    }
  } = $props()

  let imageLoaded = $state(false)

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      goto(`/track/${track.id}`)
    }
  }
</script>

<div class="h-full transition-transform duration-200 hover:-translate-y-1">
  <mdui-card
    variant="elevated"
    class="relative flex w-full cursor-pointer flex-col overflow-hidden"
    style="border-radius: 12px; aspect-ratio: 16/9;"
    onclick={() => goto(`/track/${track.id}`)}
    onkeydown={handleKeydown}
    role="link"
    tabindex="0"
    aria-label={`View details for track ${track.title} by ${track.artist}`}
  >
    <!-- Skeleton/Placeholder -->
    {#if !imageLoaded}
      <div class="absolute inset-0 animate-pulse bg-[var(--controlGray)]"></div>
    {/if}

    <!-- 16:9 Cover -->
    <img
      src={track.cover}
      alt={track.title}
      class="absolute inset-0 h-full w-full object-cover transition-opacity duration-300"
      style="opacity: {imageLoaded ? '1' : '0'};"
      loading="lazy"
      onload={() => (imageLoaded = true)}
    />

    <!-- Gradient Overlay -->
    <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent"></div>

    <!-- Text Content at Bottom -->
    <div class="absolute bottom-0 left-0 w-full p-4 text-white">
      <h3 class="mb-1 truncate text-lg font-bold shadow-black drop-shadow-md" title={track.title}>
        {track.title}
      </h3>
      <div class="flex flex-col text-sm opacity-90 sm:flex-row sm:items-center sm:gap-2 drop-shadow-md">
        <p class="truncate" title={`${track.artist} - ${track.album}`}>
          {track.artist} - {track.album}
        </p>
        {#if track.releaseDate}
          <span class="hidden sm:inline">•</span>
          <p class="truncate">{track.releaseDate}</p>
        {/if}
      </div>
    </div>
  </mdui-card>
</div>
