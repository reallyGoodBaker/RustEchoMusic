<script lang="ts">
  import '@mdui/icons/close--rounded.js'
  import '@mdui/icons/search--rounded.js'

  let { value = $bindable(''), placeholder="搜索歌曲、歌手、专辑..." } = $props()
</script>

<div
  class="flex items-center gap-2 w-full h-9 px-3 rounded-full transition-all duration-200 bg-[rgb(var(--mdui-color-surface-container-high))] focus-within:bg-[rgb(var(--mdui-color-surface-container-highest))] focus-within:shadow-sm"
  onpointerdown={e => e.stopPropagation()}
  role="searchbox"
  tabindex="0"
>
  <mdui-icon-search--rounded class="text-[rgb(var(--mdui-color-on-surface-variant))] text-base shrink-0"></mdui-icon-search--rounded>
  <input
    type="text"
    {placeholder}
    bind:value={value}
    class="flex-1 h-full bg-transparent border-none outline-none text-sm text-[rgb(var(--mdui-color-on-surface))] placeholder-[rgb(var(--mdui-color-on-surface-variant))]"
  />
  {#if value}
    <div
      class="flex items-center justify-center w-6 h-6 rounded-full cursor-pointer bg-transparent hover:bg-(--fade) active:bg-(--controlBlackAcrylic) text-base shrink-0"
      onclick={() => value = ''}
      onkeydown={e => { if (e.key === 'Enter' || e.key === ' ') value = '' }}
      role="button"
      tabindex="0"
    >
      <mdui-icon-close--rounded class="text-xs text-[rgb(var(--mdui-color-on-surface-variant))]"></mdui-icon-close--rounded>
    </div>
  {/if}
</div>