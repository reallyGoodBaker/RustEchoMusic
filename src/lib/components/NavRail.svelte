<script lang="ts">
  import 'mdui/components/navigation-rail-item.js'
  import 'mdui/components/navigation-rail.js'
</script>

<mdui-navigation-rail style="--z-index: 1">
  <mdui-fab lowered icon="search--outlined" slot="top"></mdui-fab>
  <!-- <mdui-button-icon icon="settings" slot="bottom"></mdui-button-icon> -->

  <mdui-navigation-rail-item icon="watch_later--outlined" class="mb-4"
    >Recent</mdui-navigation-rail-item
  >
  <mdui-navigation-rail-item icon="library_music--outlined"
    >Library</mdui-navigation-rail-item
  >
  <mdui-navigation-rail-item icon="track_changes--outlined"
    >Track</mdui-navigation-rail-item
  >
  <mdui-navigation-rail-item icon="person--outlined"
    >Artist</mdui-navigation-rail-item
  >


</mdui-navigation-rail>
