<script lang="ts">
  let {
    title,
    collapsible = true,
    collapsed = $bindable(false),
  }: {
    title: string
    collapsible?: boolean
    collapsed?: boolean
  } = $props()

  function toggle() {
    if (collapsible) {
      collapsed = !collapsed
    }
  }
</script>

<div
  class="mb-4 flex cursor-pointer items-center justify-between border-b border-[var(--controlGray)] pb-2 transition-opacity hover:opacity-80"
  role="button"
  tabindex="0"
  onclick={toggle}
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      toggle()
    }
  }}
>
  <h2 class="text-xl font-bold">{title}</h2>
  {#if collapsible}
    <mdui-button-icon
      icon={collapsed ? 'expand_more--rounded' : 'expand_less--rounded'}
    ></mdui-button-icon>
  {/if}
</div>
