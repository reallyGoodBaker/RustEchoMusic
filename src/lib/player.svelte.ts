import { invoke } from '@tauri-apps/api/core'

export type Track = {
  id: number
  title: string
  artist: string
  album: string
  cover: string
  path: string
}

class Player {
  current = $state<Track | null>(null)
  isPlaying = $state(false)
  playlist = $state<Track[]>([])
  progress = $state(0)
  loopMode = $state<'none' | 'all' | 'one' | 'random'>('none')

  constructor() {
    if (typeof window !== 'undefined') {
      try {
        const saved = localStorage.getItem('loopMode') as any
        if (['none', 'all', 'one', 'random'].includes(saved)) {
          this.loopMode = saved
        }
      } catch (e) {}
    }
  }

  async play(track?: Track) {
    if (track) {
      this.current = track
      this.progress = 0
      if (!this.playlist.find(t => t.id === track.id)) {
        this.playlist.push(track)
      }
    }

    // if (!this.current) return

    try {
      await invoke('play_music', {
        name: this.current?.path || '40mP 初音ミク - 恋愛裁判.flac',
      })
      this.isPlaying = true
    } catch (e) {
      console.error('Failed to play music:', e)
      this.isPlaying = true
    }
  }

  pause() {
    this.isPlaying = false
    // await invoke('pause_music')
  }

  toggle() {
    if (this.isPlaying) {
      this.pause()
    } else {
      this.play()
    }
  }

  next() {
    if (!this.playlist.length || !this.current) return
    const len = this.playlist.length
    const currentIndex = this.playlist.findIndex(t => t.id === this.current?.id)

    let newIndex = (currentIndex + 1) % len

    if (this.loopMode === 'random') {
      if (len > 1) {
        do {
          newIndex = Math.floor(Math.random() * len)
        } while (newIndex === currentIndex)
      }
    }

    this.play(this.playlist[newIndex])
  }

  prev() {
    if (!this.playlist.length || !this.current) return
    const len = this.playlist.length
    const currentIndex = this.playlist.findIndex(t => t.id === this.current?.id)
    const newIndex = (currentIndex - 1 + len) % len
    this.play(this.playlist[newIndex])
  }

  toggleLoopMode() {
    const modes: ('none' | 'all' | 'one' | 'random')[] = [
      'none',
      'all',
      'one',
      'random',
    ]
    const idx = modes.indexOf(this.loopMode)
    this.loopMode = modes[(idx + 1) % modes.length]
    // 实际应该写入用户配置 store，此处用 localStorage 作为占位符
    try {
      localStorage.setItem('loopMode', this.loopMode)
    } catch (e) {}
  }
}

export const playerState = new Player()
