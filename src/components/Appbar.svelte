<script lang="ts">
  import { Window } from '@tauri-apps/api/window'

  const appWindow = new Window('main')
  let fullScreen = $state(false)

  function min() {
    appWindow.minimize()
  }

  function toggleMax() {
    appWindow.toggleMaximize()
    fullScreen = !fullScreen
  }

  function close() {
    appWindow.close()
  }

  function drag(e: MouseEvent) {
    if (e.buttons === 1) {
      e.detail === 2 ? appWindow.toggleMaximize() : appWindow.startDragging()
    }
  }
</script>

<div
  class="fixed top-0 z-50 w-full h-14 flex items-center"
  // onpointerdown={drag}
  role="banner"
>
  <div class="flex items-center pl-6 w-28">
    <div class="opacity-70 font-medium">REM</div>
  </div>

  <div class="flex-1 flex items-center justify-center">
    <div class="max-w-md w-full"></div>
  </div>

  <div class="flex items-center pr-6 [-webkit-app-region:no-drag]">
    <div class="flex gap-1 bg-(--controlWhite) rounded-lg p-1">
      <div
        class="flex items-center justify-center w-8 h-8 rounded cursor-pointer bg-transparent hover:bg-(--fade) active:bg-(--controlBlackAcrylic) font-[material-symbols-rounded] text-lg"
        onclick={min}
        onkeydown={e => {
          if (e.key === 'Enter' || e.key === ' ') min()
        }}
        role="button"
        tabindex="0"
      >
        <md-icon class="material-symbols-rounded">minimize</md-icon>
      </div>
      <div
        class="flex items-center justify-center w-8 h-8 rounded cursor-pointer bg-transparent hover:bg-(--fade) active:bg-(--controlBlackAcrylic) text-lg mx-1"
        onclick={toggleMax}
        onkeydown={e => {
          if (e.key === 'Enter' || e.key === ' ') toggleMax()
        }}
        role="button"
        tabindex="0"
      >
        <md-icon class="material-symbols-rounded"
          >{fullScreen ? "fullscreen_exit" : "fullscreen"}</md-icon
        >
      </div>
      <div
        class="flex items-center justify-center w-8 h-8 rounded cursor-pointer bg-transparent hover:bg-red-600 hover:text-white active:bg-red-800 active:text-gray-300 text-lg"
        onclick={close}
        onkeydown={e => {
          if (e.key === 'Enter' || e.key === ' ') close()
        }}
        role="button"
        tabindex="0"
      >
        <md-icon class="material-symbols-rounded">close</md-icon>
      </div>
    </div>
  </div>
</div>
