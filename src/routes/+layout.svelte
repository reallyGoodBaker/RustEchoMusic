<script>
  import Appbar from '$lib/components/Appbar.svelte'
  import Header from '$lib/components/Header.svelte'
  import NavRail from '$lib/components/NavRail.svelte'
  import PlayerBar from '$lib/components/PlayerBar.svelte'
  import 'mdui'
  import 'mdui/mdui.css'
  import '../app.css'

  let { children } = $props()
</script>

<mdui-layout full-height>
  <Appbar />
  <PlayerBar />
  <NavRail />

  <mdui-layout-main
    class="flex flex-col h-screen w-screen overflow-hidden bg-(--controlWhite) text-(--controlBlack)"
  >
    <Header />

    {@render children()}
  </mdui-layout-main>
</mdui-layout>
