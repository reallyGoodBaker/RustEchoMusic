<script lang="ts">
  import '@material/web/button/filled-button.js'
  import { invoke } from '@tauri-apps/api/core'

  async function playMusic() {
    await invoke('play_music', {
      name: 'music/40mP 初音ミク - 恋愛裁判.flac',
    })
  }
</script>

<header
  class="flex items-center justify-between px-8 py-4 bg-(--controlBackground) border-b border-(--controlGray) mt-14 shadow-sm"
>
  <div class="header-left">Header PlaceHolder</div>
  <md-filled-button
    onclick={playMusic}
    onkeydown={(e: KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') playMusic()
    }}
    class="px-4 py-2"
    role="none"
  >
    播放音乐
  </md-filled-button>
</header>
