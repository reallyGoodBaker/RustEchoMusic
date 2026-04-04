<script lang="ts">
  import { playerState } from '$lib/player.svelte'

  type LyricLine = {
    time: number
    text: string
  }

  function parseLrc(lrcText: string): LyricLine[] {
    const lines: LyricLine[] = []
    const regex = /\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)/g
    let match

    while ((match = regex.exec(lrcText)) !== null) {
      const minutes = parseInt(match[1], 10)
      const seconds = parseInt(match[2], 10)
      const ms = parseInt(match[3].padEnd(3, '0'), 10)
      const time = minutes * 60 + seconds + ms / 1000
      const text = match[4].trim()

      if (text) {
        lines.push({ time, text })
      }
    }

    return lines.sort((a, b) => a.time - b.time)
  }

  const demoLrc = `[ti:残酷天使的行动纲领]
[ar:高桥洋子]
[al:NEON GENESIS EVANGELION]
[00:00.00]残酷天使的行动纲领
[00:04.50]少年よ神話になれ
[00:08.00]-
[00:12.00]蒼い風が今 胸のドアを優しく叩いたら
[00:17.50]透き通る声で 眠れる不思議
[00:23.00]目覚めたとき 体は軽く 風に溶けていく
[00:28.50>永遠という時の中に
[00:34.00]君と出会えた 奇跡
[00:39.50]あの日あの時 未来の姿が
[00:45.00>映し出された ライト
[00:50.50>輝くだろ 狂った星たちも
[01:02.00]-
[01:06.00]この熱さを 信じて
[01:11.50]もっと 力くわって走れずっと
[01:17.00]届け この想い 刻まれた時空を越えて
[01:22.50>キラキラに 輝く君の夢 叶うように
[01:28.00]-
[01:32.50>残酷な天使のように
[01:38.00]少年よ神話になれ
[01:43.50>広がるコスモスに 夢を描いて
[01:49.00>遠くから 君が来て微笑む
[01:54.50>涙ひとつ ひとつ 星になる
[02:00.00]-
[02:04.50>愛すること 守ること 強くなること
[02:10.00>迷いながら 探す答えは一つ
[02:15.50>傷ついた翼でも 飛べるから
[02:21.00>追いかける 光の彼方へ
[02:26.50>-
[02:31.00>この世界の果てまで
[02:36.50>君と一緒に行こう
[02:42.00>いつかまた 会える日まで
[02:47.50>約束の場所で 待っているよ
[02:53.00>-
[02:57.50>残酷な天使のように
[03:03.00]少年よ神話になれ
[03:08.50>広がるコスモスに 夢を描いて
[03:14.00>遠くから 君が来て微笑む
[03:19.50>涙ひとつ ひとつ 星になる`

  let lyricLines = $state<LyricLine[]>(parseLrc(demoLrc))
  let containerElement: HTMLElement | null = $state(null)

  const currentProgress = $derived(
    (playerState.progress / 100) * (playerState.duration || 180),
  )

  const activeIndex = $derived(() => {
    let idx = -1
    for (let i = 0; i < lyricLines.length; i++) {
      if (lyricLines[i].time <= currentProgress) {
        idx = i
      } else {
        break
      }
    }
    return idx
  })

  $effect(() => {
    const idx = activeIndex()
    if (idx >= 0 && containerElement) {
      const activeEl = containerElement.querySelector(
        `[data-line-index="${idx}"]`,
      )
      if (activeEl) {
        const containerRect = containerElement.getBoundingClientRect()
        const elementRect = activeEl.getBoundingClientRect()
        const offset =
          elementRect.top -
          containerRect.top -
          containerRect.height / 2 +
          elementRect.height / 2

        containerElement.scrollBy({
          top: offset,
          behavior: 'smooth',
        })
      }
    }
  })
</script>

<div class="flex flex-col h-full">
  <div class="px-8 pt-6 pb-4 shrink-0">
    <h2 class="text-2xl font-bold">歌词</h2>
    {#if playerState.current}
      <p class="text-sm text-(--controlBright) mt-1 truncate">
        {playerState.current.title}
        {#if playerState.current.artist}
          - {playerState.current.artist}
        {/if}
      </p>
    {:else}
      <p class="text-sm text-(--controlBright)/60 mt-1">请先播放音乐</p>
    {/if}
  </div>

  <div class="flex-1 overflow-hidden px-8 pb-8">
    {#if !playerState.current || (!playerState.isPlaying && !playerState.isBuffering)}
      <div
        class="flex flex-col items-center justify-center h-full text-(--controlBright) gap-4"
      >
        <svg
          class="w-20 h-20 opacity-30"
          fill="none"
          stroke="currentColor"
          stroke-width="1"
          viewBox="0 0 24 24"
        >
          <path
            d="M9 19V6l12-3v13M9 19c0 1.66-1.34 3-3 3S3 20.66 3 19s1.34-3 3-3 3 1.34 3 3zm12-3c0 1.66-1.34 3-3 3s-3-1.34-3-3 1.34-3 3-3 3 1.34 3 3zM9 10l12-3"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <p class="text-base">暂无歌词</p>
        <p class="text-sm opacity-60">播放音乐后自动显示歌词</p>
      </div>
    {:else}
      <div
        bind:this={containerElement}
        class="h-full overflow-y-auto scroll-smooth"
        role="list"
        aria-label="歌词列表"
      >
        <div class="flex flex-col items-center py-[40vh] gap-5">
          {#each lyricLines as line, i (i)}
            {@const isActive = i === activeIndex()}
            {@const isNext = i === activeIndex() + 1}
            <div
              data-line-index={i}
              role="listitem"
              class={`transition-all duration-300 ease-out px-4 py-1 rounded-lg text-center cursor-default ${isActive ? 'text-primary text-xl font-bold scale-105' : isNext ? 'text-(--controlDark) text-base' : 'text-(--controlBright)/50 text-sm'}`}
            >
              {#if line.text === '-'}
                <span class="opacity-30">♩ ♪ ♫ ♬</span>
              {:else}
                line.text
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style lang="postcss">
  @reference "tailwindcss";

  div::-webkit-scrollbar {
    width: 4px;
  }

  div::-webkit-scrollbar-track {
    background: transparent;
  }

  div::-webkit-scrollbar-thumb {
    background: var(--controlGray);
    border-radius: 2px;
  }
</style>
