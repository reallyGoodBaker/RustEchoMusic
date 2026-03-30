<script lang="ts">
  export let playlist: {
    id: number
    name: string
    cover: string
  }
</script>

<div class="transition-transform duration-200 hover:-translate-y-2">
  <md-card
    style="--md-card-container-color: var(--controlBackground2); --md-card-container-shape: 12px;"
    class="p-4 h-full flex flex-col"
  >
    <div
      class="w-full aspect-square rounded-lg overflow-hidden mb-4 transition-all duration-300 hover:scale-105 hover:shadow-lg"
    >
      <img
        src={playlist.cover}
        alt={playlist.name}
        class="w-full h-full object-cover"
      />
    </div>
    <div class="playlist-info">
      <h3 class="text-base font-medium mb-2">{playlist.name}</h3>
      <p class="text-sm text-(--controlBlack52)">播放列表</p>
    </div>
  </md-card>
</div>