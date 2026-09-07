<script lang="ts">
    import MediaGrid from '$lib/components/media/MediaGrid.svelte'
    import type { Playlist } from '$lib/types'

    let { playlists }: { playlists: Playlist[] } = $props()

    let items = $derived(
        playlists.map((playlist) => ({
            id: playlist.id,
            title: playlist.name,
            subtitle: '播放列表',
            image: playlist.cover,
            href: `/playlists/${playlist.id}`,
            shape: 'square' as const,
        })),
    )
</script>

<MediaGrid
    {items}
    emptyTitle="暂无播放列表"
    emptyDescription="创建一个播放列表后，它会显示在这里"
/>