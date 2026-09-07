import type { PlayMode } from "./playback"

export interface Track {
    id: number
    title: string
    artist: string | null
    album: string | null
    duration: number
    cover: string | null
    fileSize: number | null
    playCount: number
    lastPlayedAt: string | null
    createdAt: string
    updatedAt: string
}

export interface Playlist {
  id: number
  name: string
  cover: string
}

export interface Artist {
    id: string
    name: string
    cover: string | null
    trackCount: number
    albumCount: number
    tracks: Track[]
}

export interface Album {
    id: string
    title: string
    artist: string
    cover: string | null
    tracks: Track[]
    trackCount: number
    representativeTrack: Track
}

export interface PlaybackQueue {
    tracks: Track[]
    currentIndex: number | null
    playMode: PlayMode
    history: number[]
}