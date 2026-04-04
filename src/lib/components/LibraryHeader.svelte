<script lang="ts">
  let {
    title,
    count,
    filterText = $bindable(''),
    onscan,
  }: {
    title: string
    count: number
    filterText?: string
    onscan: () => void
  } = $props()
</script>

<div class="flex items-center justify-between px-8 pt-6 pb-4 shrink-0">
  <div class="flex items-center gap-4">
    <h2 class="text-2xl font-bold">{title}</h2>
    <span
      class="text-xs text-(--controlBright) px-2 py-0.5 rounded-full bg-(--controlBlackAcrylic)"
    >
      {count} 首
    </span>
  </div>
  <div class="flex items-center gap-3">
    <div class="relative">
      <input
        type="text"
        bind:value={filterText}
        placeholder="搜索歌曲、艺术家、专辑..."
        class="w-64 h-9 pl-9 pr-4 text-sm rounded-lg border border-(--controlGray) bg-transparent outline-none focus:border-primary transition-colors placeholder:text-(--controlBright)/50"
      />
      <svg
        class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-(--controlBright)"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
        />
      </svg>
    </div>
    <mdui-button-icon
      icon="folder_open--outlined"
      role="button"
      tabindex="0"
      onclick={onscan}
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && onscan()}
      aria-label="扫描音乐文件夹"
    ></mdui-button-icon>
  </div>
</div>

<style lang="postcss">
  @reference "tailwindcss";
</style>
