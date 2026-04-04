<script lang="ts">
  const defaultCover =
    'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyMDAiIGhlaWdodD0iMjAwIiB2aWV3Qm94PSIwIDAgMjAwIDIwMCI+PHJlY3Qgd2lkdGg9IjIwMCIgaGVpZ2h0PSIyMDAiIGZpbGw9IiNlMGUwZTAiLz48cGF0aCBkPSJNMTAwIDYwYTIwIDIwIDAgMSAwIDAgNDAgMjAgMjAgMCAwIDAgMC00MHptMCAzMGExMyAxMyAwIDEgMSAwLTI2IDEzIDEzIDAgMCAxIDAgMjZ6IiBmaWxsPSIjYTBiMGIwIi8+PC9zdmc+'

  export let playlist: {
    id: number
    name: string
    cover: string
  }

  function handleImageError(e: Event) {
    ;(e.target as HTMLImageElement).src = defaultCover
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
        src={playlist.cover || defaultCover}
        alt={playlist.name}
        class="w-full h-full object-cover"
        onerror={handleImageError}
      />
    </div>
    <div class="playlist-info">
      <h3 class="text-base font-medium mb-2">{playlist.name}</h3>
      <p class="text-sm text-(--controlBlack52)">播放列表</p>
    </div>
  </md-card>
</div>
