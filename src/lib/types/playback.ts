import type { Track } from "./music"

export type PlayMode = 'ListLoop' | 'SingleLoop' | 'Shuffle'

export type PlaybackTrackInfo = {
    id: number | null
    path: string | null
}

export type PlaybackStatusSnapshot = {
    hasMedia: boolean
    playing: boolean
    currentTime: number
    track: PlaybackTrackInfo | null
}

export type PlaybackProgressPayload = {
    currentTime: number
}

export type PlaybackStatePayload = {
    playing: boolean
    currentTime: number
}

export type PlaybackTrackStartedPayload = {
    track: Track
    index: number
}

export type BackendQueueTrack = Omit<Track, 'cover'>