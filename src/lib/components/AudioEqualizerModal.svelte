<script lang="ts">
  import { lockScroll } from '$lib/actions/lockScroll'
  import { portal } from '$lib/actions/portal'
  import 'mdui/components/button-icon.js'
  import 'mdui/components/slider.js'
  import { cubicOut } from 'svelte/easing'
  import { fade, scale } from 'svelte/transition'

  let { onclose }: { onclose: () => void } = $props()

  function handleOutsideClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose()
    }
  }
</script>

<svelte:window onkeydown={e => e.key === 'Escape' && onclose()} />

<div
  use:portal
  use:lockScroll
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
  onmousedown={handleOutsideClick}
  transition:fade={{ duration: 200, easing: cubicOut }}
  role="presentation"
>
  <div
    class="relative w-full max-w-md rounded-2xl bg-(--controlBackground) p-6 shadow-2xl border border-(--controlGray)"
    transition:scale={{ duration: 200, start: 0.95, easing: cubicOut }}
    onmousedown={(e: MouseEvent) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    tabindex="0"
  >
    <div class="mb-6 flex items-center justify-between">
      <h2 class="text-xl font-bold">Audio Equalizer</h2>
      <mdui-button-icon
        icon="close--rounded"
        onclick={onclose}
        onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && onclose()}
        role="button"
        tabindex="0"
      ></mdui-button-icon>
    </div>

    <div class="flex flex-col gap-6">
      <div>
        <div class="mb-2 flex justify-between text-sm font-medium">
          <span>Bass</span>
          <span class="opacity-70">0 dB</span>
        </div>
        <mdui-slider min="-12" max="12" value="0"></mdui-slider>
      </div>
      <div>
        <div class="mb-2 flex justify-between text-sm font-medium">
          <span>Mid</span>
          <span class="opacity-70">0 dB</span>
        </div>
        <mdui-slider min="-12" max="12" value="0"></mdui-slider>
      </div>
      <div>
        <div class="mb-2 flex justify-between text-sm font-medium">
          <span>Treble</span>
          <span class="opacity-70">0 dB</span>
        </div>
        <mdui-slider min="-12" max="12" value="0"></mdui-slider>
      </div>
    </div>
  </div>
</div>
