import { invoke } from '@tauri-apps/api/core'
import type { Track } from '../types/music'

type RecentPlayedTrack = {
    track: Track
    playedAt: string
}

class RecentlyPlayed {
    tracks = $state<Track[]>([])
    isLoading = $state(false)
    error = $state<string | null>(null)

    private loadPromise: Promise<Track[]> | null = null

    async load(): Promise<Track[]> {
        if (this.loadPromise) {
            return this.loadPromise
        }

        if (this.tracks.length > 0) {
            return this.tracks
        }

        this.isLoading = true
        this.error = null

        this.loadPromise = invoke<RecentPlayedTrack[]>('load_recently_played', {
            limit: 100,
            offset: 0,
        })
            .then(records => {
                const tracks = records.map(record => record.track)
                this.tracks = tracks
                return tracks
            })
            .catch(err => {
                console.error('加载最近播放失败:', err)
                this.error = String(err)
                return this.tracks
            })
            .finally(() => {
                this.isLoading = false
                this.loadPromise = null
            })

        return this.loadPromise
    }

    async add(track: Track): Promise<Track[]> {
        this.tracks = [
            track,
            ...this.tracks.filter(item => item.id !== track.id),
        ].slice(0, 100)

        try {
            await invoke('add_recently_played', {
                trackId: track.id,
                playedAt: new Date().toISOString(),
            })

            return this.tracks
        } catch (err) {
            console.error('写入最近播放失败:', err)
            this.error = String(err)
            return this.tracks
        }
    }

    async clear() {
        try {
            await invoke('clear_recently_played')
            this.tracks = []
        } catch (err) {
            console.error('清空最近播放失败:', err)
            this.error = String(err)
        }
    }
}

export const recentlyPlayed = new RecentlyPlayed()