<script lang="ts">
  import LibraryHeader from '$lib/components/LibraryHeader.svelte'
  import MusicTable from '$lib/components/MusicTable.svelte'
  import { libraryStore, type SongItem } from '$lib/library-store.svelte'
  import { playerState } from '$lib/player.svelte'

  const defaultCover =
    'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0OCIgaGVpZ2h0PSI0OCIgdmlld0JveD0iMCAwIDQ4IDQ4Ij48cmVjdCB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIGZpbGw9IiNlMGUwZTAiLz48cGF0aCBkPSJNMjQgMThhNiA2IDAgMSAwIDAgMTIgNiA2IDAgMCAwIDAtMTJ6bTAgOGE0IDQgMCAxIDEgMC04IDQgNCAwIDAgMSAwIDh6IiBmaWxsPSIjYTBiMGIwIi8+PC9zdmc+'

  let filterText = $state('')

  function handlePlay(song: SongItem) {
    playerState.play(libraryStore.toTrack(song))
    libraryStore.incrementPlayCount(song.id)
  }

  function handleRemove(index: number) {
    libraryStore.removeSong(index)
  }
</script>

<div class="flex flex-col h-full">
  <LibraryHeader
    title="最近播放"
    count={libraryStore.count}
    bind:filterText
    onscan={() => {}}
  />

  <div class="flex-1 overflow-y-auto px-8 pb-8">
    {#if libraryStore.songs.length === 0}
      <div
        class="flex flex-col items-center justify-center h-full text-(--controlBright) gap-4 py-20"
      >
        <svg
          class="w-20 h-20 opacity-30"
          fill="none"
          stroke="currentColor"
          stroke-width="1"
          viewBox="0 0 24 24"
        >
          <path
            d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <p class="text-base">暂无最近播放记录</p>
        <p class="text-sm opacity-60">播放音乐后将在此显示历史记录</p>
      </div>
    {:else}
      <MusicTable
        songs={libraryStore.songs}
        {defaultCover}
        {filterText}
        onplay={handlePlay}
        onremove={handleRemove}
      />
    {/if}
  </div>
</div>

<style lang="postcss">
  @reference "tailwindcss";
</style>
