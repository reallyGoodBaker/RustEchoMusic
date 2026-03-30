<script lang="ts">
  export let song: {
    id: number
    title: string
    artist: string
    album: string
    cover: string
  }
</script>

<md-card
  style="--md-card-container-color: var(--controlBackground2); --md-card-container-shape: 8px;"
  class="p-4 flex items-center gap-4"
>
  <div class="w-14 h-14 rounded overflow-hidden">
    <img
      src={song.cover}
      alt={song.title}
      class="w-full h-full object-cover"
    />
  </div>
  <div class="flex-1">
    <h3 class="text-base font-medium mb-1">{song.title}</h3>
    <p class="text-sm text-(--controlBlack52)">
      {song.artist} - {song.album}
    </p>
  </div>
  <div class="flex gap-2">
    <md-icon-button>
      <md-icon class="material-symbols-rounded">play_arrow</md-icon>
    </md-icon-button>
    <md-icon-button>
      <md-icon class="material-symbols-rounded">more_vert</md-icon>
    </md-icon-button>
  </div>
</md-card>