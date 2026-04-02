<script lang="ts">
  let {
    artist,
  }: {
    artist: {
      id: number
      name: string
      avatar: string
    }
  } = $props()
</script>

<div class="h-full">
  <mdui-card
    variant="elevated"
    class="flex h-full w-full flex-col items-center cursor-pointer p-4 text-center"
    style="border-radius: 12px;"
  >
    <div
      class="mb-4 aspect-square w-32 overflow-hidden rounded-full transition-all duration-300 hover:scale-105 hover:shadow-lg"
    >
      <img
        src={artist.avatar}
        alt={artist.name}
        class="h-full w-full object-cover"
      />
    </div>
    <div class="mt-auto w-full">
      <h3 class="mb-1 truncate text-base font-medium" title={artist.name}>
        {artist.name}
      </h3>
    </div>
  </mdui-card>
</div>
