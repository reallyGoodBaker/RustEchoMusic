<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/state'
  import 'mdui/components/navigation-rail-item.js'
  import 'mdui/components/navigation-rail.js'

  const routes = [
    {
      path: '/recent',
      icon: 'watch_later--outlined',
      activeIcon: 'watch_later--filled',
      label: 'Recent',
    },
    {
      path: '/library',
      icon: 'library_music--outlined',
      activeIcon: 'library_music--filled',
      label: 'Library',
    },
    {
      path: '/track',
      icon: 'track_changes--outlined',
      activeIcon: 'track_changes--filled',
      label: 'Track',
    },
    {
      path: '/artist',
      icon: 'person--outlined',
      activeIcon: 'person--filled',
      label: 'Artist',
    },
  ]

  const currentPath = $derived(page.url.pathname)

  function handleKeydown(event: KeyboardEvent, path: string) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      goto(path)
    }
  }
</script>

<mdui-navigation-rail value={currentPath} style="--z-index: 1">
  <mdui-fab lowered icon="search--outlined" slot="top"></mdui-fab>

  {#each routes as route (route.path)}
    <mdui-navigation-rail-item
      icon={route.icon}
      active-icon={route.activeIcon}
      value={route.path}
      onclick={() => goto(route.path)}
      onkeydown={(e: KeyboardEvent) => handleKeydown(e, route.path)}
      role="button"
      tabindex="0"
    >
      {route.label}
    </mdui-navigation-rail-item>
  {/each}
</mdui-navigation-rail>
