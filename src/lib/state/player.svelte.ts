import type { GlobalAppEvent, PlaybackStatusSnapshot, PlayMode } from "$lib/types"
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import type { PlaybackQueue, Track } from "../types/music"
import { lyrics } from "./lyrics.svelte"
import { recentlyPlayed } from "./recent.svelte"

class Player {
    queue = $state<PlaybackQueue>({
        tracks: [],
        currentIndex: null,
        playMode: 'ListLoop',
        history: [],
    })

    playing = $state(false)
    currentTime = $state(0)
    queueOpen = $state(false)
    #muted = $state<boolean>(false)
    #volume = $state<number>(80)
    #previousVolume = 80
    #globalUnlisten: UnlistenFn | null = null
    #isInitialized = false
    #eventBuffer: GlobalAppEvent[] = []

    constructor() {
        this.#setupMediaSession()
    }

    #setupMediaSession() {
        if (typeof navigator === 'undefined' || !('mediaSession' in navigator)) return
        if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) return

        navigator.mediaSession.setActionHandler('play', () => this.resume())
        navigator.mediaSession.setActionHandler('pause', () => this.pause())
        navigator.mediaSession.setActionHandler('previoustrack', () => this.prev())
        navigator.mediaSession.setActionHandler('nexttrack', () => this.next())
        navigator.mediaSession.setActionHandler('seekto', details => {
            const seekTime = details.seekTime
            if (seekTime == null) return
            void this.seek(seekTime)
        })
        navigator.mediaSession.setActionHandler('seekforward', () => {
            if (!this.currentTrack) return
            void this.seek(Math.min(this.currentTime + 10, this.#trackDurationSeconds(this.currentTrack)))
        })
        navigator.mediaSession.setActionHandler('seekbackward', () => {
            if (!this.currentTrack) return
            void this.seek(Math.max(this.currentTime - 10, 0))
        })
    }

    #updateMediaMetadata() {
        if (typeof navigator === 'undefined' || !('mediaSession' in navigator) || !this.currentTrack) return

        navigator.mediaSession.metadata = new MediaMetadata({
            title: this.currentTrack.title,
            artist: this.currentTrack.artist ?? '未知歌手',
            album: this.currentTrack.album ?? '未知专辑',
            artwork: this.#buildArtwork(),
        })
    }

    #buildArtwork() {
        const cover = this.currentTrack?.cover
        return cover ? [{ src: cover }] : []
    }

    #trackDurationSeconds(track: Track): number {
        return track.duration / 1000
    }

    #updatePlaybackState() {
        if (typeof navigator === 'undefined' || !('mediaSession' in navigator)) return
        navigator.mediaSession.playbackState = this.playing ? 'playing' : 'paused'
    }

    #updatePositionState() {
        const track = this.currentTrack
        if (!track || typeof navigator === 'undefined' || !('mediaSession' in navigator) || !('setPositionState' in navigator.mediaSession)) return

        navigator.mediaSession.setPositionState({
            duration: this.#trackDurationSeconds(track),
            playbackRate: 1,
            position: this.currentTime,
        })
    }

    #syncMediaSession() {
        this.#updateMediaMetadata()
        this.#updatePlaybackState()
        this.#updatePositionState()
    }

    #clearMediaSession() {
        if (typeof navigator === 'undefined' || !('mediaSession' in navigator)) return
        navigator.mediaSession.metadata = null
        navigator.mediaSession.playbackState = 'none'
    }

    get currentTrack(): Track | undefined {
        const index = this.queue.currentIndex
        if (index === null || index < 0 || index >= this.queue.tracks.length) {
            return undefined
        }
        return this.queue.tracks[index]
    }

    get volume() { return this.#volume }
    set volume(volume: number) {
        invoke('set_volume', { volume }).catch(console.error)
    }

    get muted() { return this.#muted }
    set muted(value: boolean) {
        if (value) {
            this.#previousVolume = this.#volume
            this.#muted = true
            invoke('set_volume', { volume: 0 }).catch(console.error)
        } else {
            this.#muted = false
            invoke('set_volume', { volume: this.#previousVolume }).catch(console.error)
        }
    }

    async replacePlaylistAndPlay(tracks: Track[], targetId: number) {
        await invoke('replace_playlist_and_play', { tracks, targetId })
    }

    async insertTracksAsNext(tracks: Track[]) {
        await invoke('insert_tracks_as_next', { tracks })
    }

    async insertTrackAsNext(track: Track) {
        await invoke('insert_track_as_next', { track })
    }

    async removeTrack(trackId: number) {
        await invoke('remove_track_from_queue', { trackId })
    }

    async playTrackInQueue(index: number) {
        await invoke('play_queue_track', { index })
    }

    async cyclePlayMode() {
        const modes: PlayMode[] = ['ListLoop', 'SingleLoop', 'Shuffle']
        const index = modes.indexOf(this.queue.playMode)
        const mode = modes[(index + 1) % modes.length]
        await invoke('set_play_mode', { mode })
    }

    async clearQueue() {
        await invoke('clear_queue')
    }

    async next() {
        await invoke('play_next_track')
    }

    async prev() {
        await invoke('play_previous_track')
    }

    resume = async () => {
        try {
            await invoke('resume_track')
        } catch (err) {
            console.error(err)
        }
    }

    pause = async () => {
        try {
            await invoke('pause_track')
        } catch (err) {
            console.error(err)
        }
    }

    toggle = async () => {
        if (this.playing) {
            await this.pause()
        } else {
            await this.resume()
        }
    }

    seek = async (time: number) => {
        await invoke('set_current_time', { time })
    }

    toggleQueue() {
        this.queueOpen = !this.queueOpen
    }

    async loadState() {
        await this.#setupPlaybackListeners()

        try {
            const queue = await invoke<PlaybackQueue>('get_playback_queue')
            this.queue = queue

            const status = await invoke<PlaybackStatusSnapshot>('get_current_status')
            this.playing = status.playing
            this.currentTime = status.currentTime
            this.#syncMediaSession()
        } catch (err) {
            console.error(err)
        } finally {
            this.#isInitialized = true
            this.#flushEventBuffer()
        }
    }

    #flushEventBuffer() {
        while (this.#eventBuffer.length > 0) {
            const event = this.#eventBuffer.shift()
            if (event) {
                this.#handleGlobalEvent(event)
            }
        }
    }

    async #setupPlaybackListeners() {
        if (this.#globalUnlisten) return

        this.#globalUnlisten = await listen<GlobalAppEvent>('global-app-event', event => {
            if (!this.#isInitialized) {
                this.#eventBuffer.push(event.payload)
                return
            }
            this.#handleGlobalEvent(event.payload)
        })
    }

    #handleGlobalEvent(payload: GlobalAppEvent) {
        const { type, payload: data } = payload

        switch (type) {
            case 'PlaybackProgress':
                this.currentTime = data.currentTime
                this.#updatePositionState()
                break

            case 'TrackStarted':
                this.currentTime = 0
                this.playing = true
                this.queue = { ...this.queue, currentIndex: data.index }
                this.#syncMediaSession()
                recentlyPlayed.add(data.track)
                break

            case 'VolumeChanged':
                this.#volume = Math.round(data * 100)
                this.#muted = this.#volume === 0
                break

            case 'QueueChanged':
                this.queue = data
                break

            case 'PlaybackStateChanged':
                this.playing = data.playing
                this.currentTime = data.currentTime
                this.#syncMediaSession()
                break

            case 'LyricsLoaded':
                lyrics.handleLyricsLoaded(data.songId, data.lines)
                break
        }
    }

    destroy() {
        if (this.#globalUnlisten) {
            this.#globalUnlisten()
            this.#globalUnlisten = null
        }
    }
}

export const player = new Player()