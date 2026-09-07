<script lang="ts">
<<<<<<< HEAD
  import Appbar from '$lib/components/Appbar.svelte'
  import NavRail from '$lib/components/NavRail.svelte'
  import PlayerBar from '$lib/components/PlayerBar.svelte'
  import 'mdui'
  import 'mdui/mdui.css'
  import '../app.css'
=======
    import PlayerBar from '$lib/features/PlayerBar.svelte'
    import QueueDrawer from '$lib/features/QueueDrawer.svelte'
    import PluginNativeViewHost from '$lib/features/PluginNativeViewHost.svelte'
    import Appbar from '$lib/features/shell/Appbar.svelte'
    import NavRail from '$lib/features/shell/NavRail.svelte'
    import { musicLibrary } from '$lib/state/library.svelte'
    import { trackCovers } from '$lib/state/covers.svelte'
    import { player } from '$lib/state/player.svelte'
    import { pluginState } from '$lib/state/plugins.svelte'
    import { recentlyPlayed } from '$lib/state/recent.svelte'
    import { settings } from '$lib/state/settings.svelte'
    import { registerBuiltinViews } from '$lib/plugins/register-builtin-views'
    import 'mdui'
    import 'mdui/mdui.css'
    import { onMount } from 'svelte'
    import '../app.css'
>>>>>>> pr/4

    import { goto } from '$app/navigation'
    import { listen } from '@tauri-apps/api/event'

    let { children } = $props()

    let activeNativeView = $derived(
        pluginState.activeNativePanel
            ? pluginState.nativeViewExtensions.find(
                  nv =>
                      nv.pluginId === pluginState.activeNativePanel &&
                      nv.state === 'Enabled',
              )
            : undefined,
    )

    onMount(() => {
        registerBuiltinViews()
        void settings.load()
        void player.loadState()
        void pluginState.loadManifests()
        void pluginState.loadSidebarExtensions()
        void pluginState.loadNativeViewExtensions()
        void pluginState.loadKernel().then(registerBuiltinViews)
        void recentlyPlayed.load()
        void musicLibrary.load().then(tracks => {
            if (!tracks || tracks.length === 0) return
            const BATCH = 10
            const DELAY = 50
            let i = 0
            const loadNext = () => {
                const batch = tracks.slice(i, i + BATCH)
                if (batch.length === 0) return
                for (const track of batch) {
                    void trackCovers.load(track)
                }
                i += BATCH
                setTimeout(loadNext, DELAY)
            }
            loadNext()
        })

        const trayUnlistenPromise = listen<string>('tray:navigate', event => {
            if (event.payload === 'settings') {
                void goto('/settings')
            }
        })

        const globalEventUnlistenPromise = listen<{
            type: string
            payload: any
        }>('global-app-event', event => {
            const { type, payload } = event.payload

            if (type === 'SettingsChanged') {
                settings.updateLocalState(payload)
            }
        })

        return () => {
            void trayUnlistenPromise.then(unlisten => unlisten())
            void globalEventUnlistenPromise.then(unlisten => unlisten())
            player.destroy()
        }
    })

    $effect(() => {
        document.documentElement.style.setProperty(
            '--transition-duration',
            settings.data.reduceMotion ? '0s' : '0.2s',
        )
    })
</script>

<mdui-layout full-height>
<<<<<<< HEAD
  <Appbar />
  <PlayerBar />
  <NavRail />

  <mdui-layout-main
    class="flex flex-col h-screen w-screen overflow-hidden bg-(--controlWhite) text-(--controlBlack)"
  >
    {@render children()}
  </mdui-layout-main>
=======
    <Appbar />
    <PlayerBar />
    <NavRail />
    <mdui-layout-main
        class="flex flex-col h-screen w-screen overflow-hidden bg-(--controlWhite) text-(--controlBlack)"
    >
        {@render children()}
    </mdui-layout-main>
>>>>>>> pr/4
</mdui-layout>

<QueueDrawer />

{#if activeNativeView}
    <div
        class="fixed bottom-24 right-4 z-50 w-[420px] h-96 native-view-box"
    >
        <PluginNativeViewHost token={activeNativeView.token} pluginId={activeNativeView.pluginId} />
    </div>
{/if}

<style lang="postcss">
    @reference "tailwindcss";
    @layer base {
        :global(*) {
            transition: all var(--transition-duration, 0.2s) ease;
        }

        .native-view-box {
            display: block;
            transition:
                opacity 0.4s ease,
                display 0.4s ease allow-discrete;
        }

        @starting-style {
            .native-view-box {
                opacity: 0;
            }
        }
    }
</style>
