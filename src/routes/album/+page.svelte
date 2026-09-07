<script lang="ts">
    import Heading from '$lib/components/base/Heading.svelte'
    import IconButton from '$lib/components/base/IconButton.svelte'
    import SearchBar from '$lib/components/base/SearchBar.svelte'
    import MediaGrid, {
        type MediaGridItem,
    } from '$lib/components/media/MediaGrid.svelte'
    import TrackList from '$lib/features/track-list/TrackList.svelte'
    import { trackCovers } from '$lib/state/covers.svelte'
    import { musicLibrary } from '$lib/state/library.svelte'
    import { player } from '$lib/state/player.svelte'
    import { settings } from '$lib/state/settings.svelte'
    import type { Album, Track } from '$lib/types'

    import '@mdui/icons/arrow-back.js'
    import '@mdui/icons/play-arrow.js'
    import 'mdui/components/circular-progress.js'
    import 'mdui/components/list-item.js'
    import 'mdui/components/list.js'
    import 'mdui/components/switch.js'

    let searchQuery = $state('')
    let selectedAlbumId = $state<string | null>(null)
    let windowWidth = $state(1024)

    const isMobile = $derived(windowWidth < 768)
    const selectedAlbum: Album | undefined = $derived(
        selectedAlbumId === null
            ? undefined
            : musicLibrary.getAlbum(selectedAlbumId),
    )

    const collator = new Intl.Collator('zh-Hans-CN', {
        numeric: true,
        sensitivity: 'base',
    })

    const filteredAlbums = $derived.by(() => {
        const keyword = searchQuery.trim().toLowerCase()
        const albums = musicLibrary.albums
        const list = keyword
            ? albums.filter(
                  album =>
                      album.title.toLowerCase().includes(keyword) ||
                      album.artist.toLowerCase().includes(keyword),
              )
            : albums

        return list.toSorted((a, b) => collator.compare(a.title, b.title))
    })

    const albumGridItems = $derived(
        filteredAlbums.map(album => ({
            id: album.id,
            title: album.title,
            subtitle: `${album.trackCount} 首歌曲`,
            image: trackCovers.get(album.representativeTrack) ?? album.cover,
            shape: 'square' as const,
            onvisible: () => {
                void trackCovers.load(album.representativeTrack)
            },
        })),
    )

    function playAll(tracks: Track[]) {
        if (tracks.length === 0) return
        player.replacePlaylistAndPlay(tracks, tracks[0].id)
    }

    function selectAlbumItem(item: MediaGridItem) {
        const album = musicLibrary.getAlbum(String(item.id))
        if (album === undefined) return
        selectedAlbumId = album.id
    }

    function playAlbumItem(item: MediaGridItem, event: Event) {
        event.preventDefault()
        event.stopPropagation()
        const album = musicLibrary.getAlbum(String(item.id))
        if (album === undefined) return
        playAll(album.tracks)
    }

    function handleAlbumArtistGroupingChange(event: Event) {
        const checked = (event.currentTarget as HTMLInputElement).checked
        if (checked === settings.data.useAlbumArtistGrouping) return
        settings.update({ useAlbumArtistGrouping: checked })
        selectedAlbumId = null
    }
</script>

<svelte:window bind:innerWidth={windowWidth} />

<svelte:head>
    <title>专辑</title>
</svelte:head>

<div class="flex flex-col h-full overflow-hidden">
    <header
        class="pb-4 shrink-0 border-b border-[rgb(var(--mdui-color-outline-variant))]"
    >
        <Heading eyebrow="Albums" title="专辑" />
        <div class="flex mt-2 px-1 gap-16 justify-end items-center">
            <div class="max-w-md">
                <SearchBar bind:value={searchQuery} />
            </div>

            <div
                class="flex items-center gap-2 text-sm text-[rgb(var(--mdui-color-on-surface-variant))]"
            >
                <span>按专辑艺术家分组</span>
                <mdui-switch
                    checked={musicLibrary.useAlbumArtistGrouping}
                    onchange={handleAlbumArtistGroupingChange}
                ></mdui-switch>
            </div>
        </div>
    </header>

    {#if musicLibrary.isLoading}
        <div class="flex flex-1 items-center justify-center">
            <mdui-circular-progress></mdui-circular-progress>
        </div>
    {:else if musicLibrary.error}
        <div class="flex flex-1 items-center justify-center text-red-500">
            {musicLibrary.error}
        </div>
    {:else if isMobile}
        <div class="flex-1 overflow-hidden min-h-0 flex flex-col">
            {#if selectedAlbum === undefined}
                <div class="flex-1 overflow-y-auto pb-8">
                    {#if filteredAlbums.length === 0}
                        <div
                            class="flex flex-col items-center justify-center h-64 text-[rgb(var(--mdui-color-on-surface-variant))]"
                        >
                            <span class="text-4xl mb-2">🎵</span>
                            <span class="text-sm">没有找到专辑</span>
                        </div>
                    {:else}
                        <mdui-list>
                            {#each filteredAlbums as album (album.id)}
                                {@const cover = trackCovers.get(album.representativeTrack) ?? album.cover}
                                <mdui-list-item
                                    headline={album.title}
                                    description={`${album.trackCount} 首歌曲`}
                                    onclick={() => { selectedAlbumId = album.id }}
                                    onkeydown={(e: KeyboardEvent) =>
                                        (e.key === 'Enter' || e.key === ' ') &&
                                        (selectedAlbumId = album.id)}
                                    role="button"
                                    tabindex="0"
                                >
                                    {#if cover}
                                        <img src={cover} slot="icon" class="w-10 h-10 rounded object-cover shrink-0" alt="" />
                                    {:else}
                                        <div slot="icon" class="w-10 h-10 rounded bg-[rgb(var(--mdui-color-surface-container-highest))] flex items-center justify-center text-lg shrink-0">
                                            🎵
                                        </div>
                                    {/if}
                                </mdui-list-item>
                            {/each}
                        </mdui-list>
                    {/if}
                </div>
            {:else}
                <div class="flex-1 flex flex-col overflow-hidden min-h-0">
                    <div class="flex items-center gap-3 py-3 border-b border-[rgb(var(--mdui-color-outline-variant))] shrink-0">
                        <IconButton
                            icon="arrow_back--rounded"
                            onclick={() => { selectedAlbumId = null }}
                        />
                        <div class="flex-1 min-w-0">
                            <h2 class="text-base font-semibold truncate text-[rgb(var(--mdui-color-on-surface))]">
                                {selectedAlbum.title}
                            </h2>
                            <p class="text-xs text-[rgb(var(--mdui-color-on-surface-variant))]">
                                {selectedAlbum.trackCount} 首歌曲
                            </p>
                        </div>
                    </div>
                    <div class="flex-1 overflow-y-auto pb-8">
                        <TrackList tracks={selectedAlbum.tracks} />
                    </div>
                </div>
            {/if}
        </div>
    {:else}
        <div class="flex-1 flex overflow-hidden min-h-0">
            <div
                class="flex-2 shrink-0 border-r border-[rgb(var(--mdui-color-outline-variant))] overflow-y-auto pb-8 pr-4"
            >
                <MediaGrid
                    items={albumGridItems}
                    selectedId={selectedAlbumId}
                    onselect={selectAlbumItem}
                    onplay={playAlbumItem}
                    emptyTitle="没有找到专辑"
                    emptyDescription="尝试调整搜索关键词"
                />
            </div>
            <div class="flex-1 flex flex-col overflow-hidden min-h-0">
                {#if selectedAlbum === undefined}
                    <div class="flex-1 flex flex-col items-center justify-center text-[rgb(var(--mdui-color-on-surface-variant))]">
                        <span class="text-5xl mb-2">🎵</span>
                        <span class="text-sm">选择一个专辑以查看歌曲</span>
                    </div>
                {:else}
                    <div class="flex-1 overflow-y-auto pb-8">
                        <TrackList
                            tracks={selectedAlbum.tracks}
                            columns={['title', 'duration']}
                        />
                    </div>
                {/if}
            </div>
        </div>
    {/if}
</div>
