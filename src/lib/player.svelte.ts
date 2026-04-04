import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export type Track = {
  id: number
  title: string
  artist: string
  album: string
  cover: string
  path: string
  duration?: number
}

class Player {
  current = $state<Track | null>(null)
  isPlaying = $state(false)
  isBuffering = $state(false)
  playlist = $state<Track[]>([])
  progress = $state(0)
  duration = $state(0)
  loopMode = $state<'none' | 'all' | 'one' | 'random'>('none')
  mute = $state(false)
  error = $state<string | null>(null)

  constructor() {
    if (typeof window !== 'undefined') {
      try {
        const saved = localStorage.getItem('loopMode') as any
        if (['none', 'all', 'one', 'random'].includes(saved)) {
          this.loopMode = saved
        }
      } catch (e) {
        console.error('Failed to load loop mode:', e)
      }

      listen<boolean>('audio:buffering', event => {
        this.isBuffering = event.payload
      }).catch(console.error)

      listen<number>('audio:duration', event => {
        this.duration = event.payload
      }).catch(console.error)

      listen<number>('audio:progress', event => {
        this.progress = event.payload * 100
      }).catch(console.error)
    }
  }

  async play(track?: Track, retryCount = 0): Promise<void> {
    if (track) {
      this.current = track
      this.progress = 0
      this.duration = track.duration ?? 0
      this.error = null
      if (!this.playlist.find(t => t.id === track.id)) {
        this.playlist.push(track)
      }
    }

    if (!this.current) return

    try {
      this.isBuffering = true
      this.isPlaying = false

      await invoke('play_music_from_path', {
        path: this.current.path,
      })

      this.isPlaying = true
      this.isBuffering = false
    } catch (e) {
      this.isBuffering = false
      this.isPlaying = false
      const errMsg = e instanceof Error ? e.message : String(e)
      console.error('Failed to play music:', e)

      if (retryCount < 2) {
        console.log(`Retrying play (${retryCount + 1}/2)...`)
        await new Promise(resolve =>
          setTimeout(resolve, 500 * (retryCount + 1)),
        )
        return this.play(undefined, retryCount + 1)
      }

      this.error = errMsg
    }
  }

  async pause(): Promise<void> {
    try {
      await invoke('pause_music')
      this.isPlaying = false
      this.isBuffering = false
    } catch (e) {
      console.error('Failed to pause music:', e)
    }
  }

  async resume(): Promise<void> {
    if (!this.current) return

    try {
      this.isBuffering = true
      await invoke('resume_music', {
        path: this.current.path,
      })
      this.isPlaying = true
      this.isBuffering = false
    } catch (e) {
      console.error('Failed to resume music:', e)
      this.isBuffering = false
      this.isPlaying = false
    }
  }

  async toggle(): Promise<void> {
    if (this.isBuffering) return
    if (this.isPlaying) {
      await this.pause()
    } else {
      await this.resume()
    }
  }

  clearError() {
    this.error = null
  }

  async next(): Promise<void> {
    if (!this.playlist.length || !this.current) return
    const len = this.playlist.length
    const currentIndex = this.playlist.findIndex(t => t.id === this.current?.id)

    let newIndex: number

    if (this.loopMode === 'random') {
      if (len > 1) {
        do {
          newIndex = Math.floor(Math.random() * len)
        } while (newIndex === currentIndex)
      } else {
        newIndex = currentIndex
      }
    } else {
      newIndex = (currentIndex + 1) % len
    }

    await this.play(this.playlist[newIndex])
  }

  async prev(): Promise<void> {
    if (!this.playlist.length || !this.current) return
    const len = this.playlist.length
    const currentIndex = this.playlist.findIndex(t => t.id === this.current?.id)
    const newIndex = (currentIndex - 1 + len) % len
    await this.play(this.playlist[newIndex])
  }

  async stop(): Promise<void> {
    try {
      await invoke('stop_music')
      this.isPlaying = false
    } catch (e) {
      console.error('Failed to stop music:', e)
    }
  }

  insertNext(track: Track) {
    if (!this.current) {
      this.play(track)
      return
    }

    const currentIndex = this.playlist.findIndex(t => t.id === this.current?.id)

    if (currentIndex >= 0 && currentIndex < this.playlist.length - 1) {
      this.playlist.splice(currentIndex + 1, 0, track)
    } else {
      this.playlist.push(track)
    }
  }

  removeFromPlaylist(index: number) {
    const newPlaylist = [...this.playlist]
    newPlaylist.splice(index, 1)
    this.playlist = newPlaylist
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
    try {
      localStorage.setItem('loopMode', this.loopMode)
    } catch (e) {}
  }

  toggleMute() {
    this.mute = !this.mute
  }
}

export const playerState = new Player()
