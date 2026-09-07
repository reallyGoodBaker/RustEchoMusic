<script lang="ts">
    import { page } from '$app/state'

    import { scanDirectory } from '$lib/utils/library'

    import 'mdui/components/navigation-rail-item.js'
    import 'mdui/components/navigation-rail.js'

    type NavItem = {
        label: string
        href: string
        icon: string
        slot?: 'bottom'
    }

    const navItems: NavItem[] = [
        {
            label: 'Recent',
            href: '/recent',
            icon: 'watch_later--outlined',
        },
        {
            label: 'Library',
            href: '/library',
            icon: 'library_music--outlined',
        },
        {
            label: 'Album',
            href: '/album',
            icon: 'track_changes--outlined',
        },
        {
            label: 'Artist',
            href: '/artists',
            icon: 'person--outlined',
        },
        {
            label: 'Plugins',
            href: '/plugins',
            icon: 'extension--outlined',
            slot: 'bottom',
        },
        {
            label: 'Settings',
            href: '/settings',
            icon: 'settings--outlined',
            slot: 'bottom',
        },
    ]

    const currentRoute = $derived.by(() => {
        return (
            navItems.find(item => page.url.pathname.startsWith(item.href))
                ?.href ?? '/library'
        )
    })
</script>

<mdui-navigation-rail style="--z-index: 1" value={currentRoute}>
    <mdui-fab
        lowered
        icon="playlist_add--rounded"
        slot="top"
        onclick={scanDirectory}
        onkeydown={(e: KeyboardEvent) => {
            if (e.key === 'Enter' || e.key === ' ') {
                scanDirectory()
            }
        }}
        role="button"
        tabindex="0"
    ></mdui-fab>

    {#each navItems as item (item.href)}
        <mdui-navigation-rail-item
            value={item.href}
            href={item.href}
            icon={item.icon}
            slot={item.slot}
        >
            {item.label}
        </mdui-navigation-rail-item>
    {/each}
</mdui-navigation-rail>
