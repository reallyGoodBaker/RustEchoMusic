import type { Track } from './player.svelte'

const STORAGE_KEY = 'rust-echo-music-library'

export type SongItem = {
  id: number
  title: string
  artist: string
  album: string
  releaseDate?: string
  cover: string
  path: string
  duration?: number
  playCount?: number
}

function loadFromStorage(): SongItem[] {
  if (typeof window === 'undefined') return []
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []
    return JSON.parse(raw) as SongItem[]
  } catch {
    return []
  }
}

function saveToStorage(songs: SongItem[]) {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(songs))
  } catch (e) {
    console.error('Failed to save library:', e)
  }
}

class LibraryStore {
  songs = $state<SongItem[]>(loadFromStorage())

  get count(): number {
    return this.songs.length
  }

  setSongs(songs: SongItem[]) {
    this.songs = songs
    this.persist()
  }

  addSongs(songs: SongItem[]) {
    this.songs = [...this.songs, ...songs]
    this.persist()
  }

  removeSong(index: number) {
    this.songs = this.songs.filter((_, i) => i !== index)
    this.persist()
  }

  removeSongById(id: number) {
    this.songs = this.songs.filter(s => s.id !== id)
    this.persist()
  }

  incrementPlayCount(id: number) {
    const song = this.songs.find(s => s.id === id)
    if (song) {
      song.playCount = (song.playCount ?? 0) + 1
      this.persist()
    }
  }

  clear() {
    this.songs = []
    this.persist()
  }

  persist() {
    saveToStorage(this.songs)
  }

  toTrack(song: SongItem): Track {
    return {
      id: song.id,
      title: song.title,
      artist: song.artist,
      album: song.album,
      cover: song.cover,
      path: song.path,
      duration: song.duration,
    }
  }
}

export const libraryStore = new LibraryStore()
