<script>
  import { playerState } from '$lib/player.svelte'
  import Slider from './Slider.svelte'

  async function switchTrack(step) {
    if (!playerState.playlist.length || !playerState.current) return

    const len = playerState.playlist.length
    const currentIndex = playerState.playlist.indexOf(playerState.current)

    const newIndex = (currentIndex + step + len) % len

    playerState.current = playerState.playlist[newIndex]
    playerState.progress = 0
    playerState.isPlaying = true
  }

  export const next = () => switchTrack(1)
  export const prev = () => switchTrack(-1)

  export function toggle() {
    playerState.isPlaying = !playerState.isPlaying
  }
</script>

<mdui-bottom-app-bar
  class="relative flex flex-col h-24 px-4 pb-2"
  style="--z-index: 10"
>
  <Slider
    value="100"
    class="absolute top-0 left-0 w-full px-2 -translate-y-1/2"
  />

  <div class="info">
    <div>{playerState.current?.title}</div>
    <div class="artist">{playerState.current?.artist}</div>
  </div>

  <div class="flex items-center justify-between w-full h-full mt-2">
    <div class="flex flex-col justify-center min-w-37.5 overflow-hidden">
      <div class="text-sm font-medium truncate">
        {playerState.current?.title ?? '未在播放'}
      </div>
      <div class="text-xs opacity-70 truncate">
        {playerState.current?.artist ?? '未知艺术家'}
      </div>
    </div>

    <div class="flex items-center gap-2">
      <mdui-button-icon
        icon="skip_previous--rounded"
        role="button"
        tabindex="0"
        onclick={prev}
        onkeydown={e => e.key === 'Enter' && prev()}
      ></mdui-button-icon>
      <mdui-button-icon
        variant="filled" 
        icon={playerState.isPlaying ? 'pause--rounded' : 'play_arrow--rounded'}
        role="button"
        tabindex="0"
        onclick={toggle}
        onkeydown={e => e.key === 'Enter' && toggle()}
      >
      </mdui-button-icon>
      <mdui-button-icon
        icon="skip_next--rounded"
        role="button"
        tabindex="0"
        onclick={next}
        onkeydown={e => e.key === 'Enter' && next()}
      ></mdui-button-icon>
    </div>

    <div class="flex items-center min-w-37.5 justify-end">
      <mdui-button-icon icon="volume_up--rounded" class="text-lg opacity-70">
      </mdui-button-icon>
      <Slider value="100" class="w-32" />
    </div>
  </div>
</mdui-bottom-app-bar>
