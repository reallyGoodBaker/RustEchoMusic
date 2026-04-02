<script lang="ts">
  import 'mdui/components/button.js'

  let {
    title,
    description,
    cover,
    onPlayAll,
  }: {
    title: string
    description: string
    cover: string
    onPlayAll?: () => void
  } = $props()
</script>

<div class="relative w-full overflow-hidden rounded-xl bg-black/40 text-white shadow-md md:h-[320px]">
  <!-- Blurred Background -->
  <div
    class="absolute inset-0 z-0 bg-cover bg-center blur-2xl filter"
    style="background-image: url('{cover}');"
  ></div>
  <div class="absolute inset-0 z-0 bg-black/50"></div>

  <!-- Content Container -->
  <div class="relative z-10 flex h-full flex-col md:flex-row">
    
    <!-- Mobile: Cover on Top (≤768px -> md is 768px, so flex-col below md) -->
    <!-- Desktop: Left 40% Text, Right 60% Cover -->
    
    <!-- Left 40% Text Area (on md) -->
    <div class="order-2 flex w-full flex-col justify-center p-8 md:order-1 md:w-[40%]">
      <h1 class="mb-4 text-3xl font-bold md:text-4xl drop-shadow-md">{title}</h1>
      <p class="mb-8 text-sm opacity-90 drop-shadow-md md:text-base">
        {description}
      </p>
      <div>
        <mdui-button variant="filled" class="rounded-full" onclick={onPlayAll} onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && onPlayAll?.()} role="button" tabindex="0">
          Play All
        </mdui-button>
      </div>
    </div>

    <!-- Right 60% Cover Area (on md) -->
    <div class="order-1 flex w-full items-center justify-center p-8 md:order-2 md:w-[60%]">
      <div class="aspect-square w-48 overflow-hidden rounded-xl shadow-2xl md:w-64">
        <img
          src={cover}
          alt={title}
          class="h-full w-full object-cover"
          loading="lazy"
        />
      </div>
    </div>

  </div>
</div>
