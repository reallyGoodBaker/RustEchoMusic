<script lang="ts">
<<<<<<< HEAD
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
=======
    import TrackList from '$lib/features/track-list/TrackList.svelte'
    import { recentlyPlayed } from '$lib/state/recent.svelte'

    import Button from '$lib/components/base/Button.svelte'
    import Heading from '$lib/components/base/Heading.svelte'
    import 'mdui/components/button.js'
    import 'mdui/components/circular-progress.js'
</script>

<svelte:head>
    <title>最近播放</title>
</svelte:head>

<header class="border-b border-[rgb(var(--mdui-color-outline-variant))] pb-5">
    <Heading eyebrow="Recent" title="最近播放" />
    <div class="flex items-center gap-2">
        <Button
            variant="filled"
            icon="play_arrow--rounded"
            disabled={recentlyPlayed.tracks.length === 0}
            onclick={() => recentlyPlayed.clear()}
        >
            播放全部
        </Button>

        {#if recentlyPlayed.tracks.length > 0}
            <Button
                variant="outlined"
                onclick={() => recentlyPlayed.clear()}
            >
                清空
            </Button>
        {/if}
    </div>
</header>

{#if recentlyPlayed.isLoading}
    <div class="flex-1 flex items-center justify-center">
        <mdui-circular-progress></mdui-circular-progress>
    </div>
{:else if recentlyPlayed.error}
    <div class="flex-1 flex items-center justify-center text-red-500">
        {recentlyPlayed.error}
    </div>
{:else if recentlyPlayed.tracks.length === 0}
    <div
        class="flex-1 flex flex-col items-center justify-center gap-2 themed-text-secondary"
    >
        <div class="text-5xl">🎧</div>
        <div class="text-base font-medium">还没有最近播放记录</div>
        <div class="text-sm">播放一首歌后，它会出现在这里</div>
    </div>
{:else}
    <div class="flex-1 overflow-auto">
        <TrackList tracks={recentlyPlayed.tracks} />
    </div>
{/if}
>>>>>>> pr/4
