<script lang="ts">
  import { playerState } from '$lib/player.svelte'

  let {
    song,
  }: {
    song: {
      id: number
      title: string
      artist: string
      album: string
      cover: string
      releaseDate?: string
    }
  } = $props()

  function handlePlay() {
    playerState.play({
      id: song.id,
      title: song.title,
      artist: song.artist,
      album: song.album,
      cover: song.cover,
      path: `music/${song.artist} - ${song.title}.flac`
    })
  }
</script>

<mdui-card
  variant="elevated"
  class="flex items-center gap-4 p-4 w-full shadow-md hover:bg-[var(--controlBackgroundHover)] transition-colors cursor-pointer"
  style="border-radius: 12px;"
  ondblclick={handlePlay}
  onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handlePlay()}
  role="button"
  tabindex="0"
  data-song-id={song.id}
  data-song-title={song.title}
  data-song-artist={song.artist}
  data-song-album={song.album}
  data-song-cover={song.cover}
>
  <div class="h-16 w-16 shrink-0 overflow-hidden rounded-xl">
    <img
      src={song.cover}
      alt={song.title}
      class="h-full w-full object-cover"
    />
  </div>
  <div class="flex-1 min-w-0">
    <h3 class="mb-1 truncate text-base font-medium" title={song.title}>
      {song.title}
    </h3>
    <div class="flex flex-col text-sm opacity-70 sm:flex-row sm:items-center sm:gap-2">
      <p class="truncate" title={`${song.artist} - ${song.album}`}>
        {song.artist} - {song.album}
      </p>
      {#if song.releaseDate}
        <span class="hidden sm:inline">•</span>
        <p class="truncate">{song.releaseDate}</p>
      {/if}
    </div>
  </div>
</mdui-card>