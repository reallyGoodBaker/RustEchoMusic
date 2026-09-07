import { invoke } from '@tauri-apps/api/core'
import type { Track } from '../types/music'

class TrackCovers {
    covers = $state<Record<string, string | null>>({})
    private promises = new Map<number, Promise<string | null>>()

    async load(track: Track | null | undefined): Promise<string | null> {
        if (!track) return null

        if (this.covers[track.id] !== undefined) {
            return this.covers[track.id]
        }

        if (this.promises.has(track.id)) {
            return await this.promises.get(track.id)!
        }

        const promise = invoke<string | null>('get_track_cover', { trackId: track.id })
            .then((cover) => {
                this.covers[track.id] = cover
                return cover
            })
            .catch((err) => {
                console.error('加载封面失败:', track.id, err)
                this.covers[track.id] = null
                return null
            })
            .finally(() => {
                this.promises.delete(track.id)
            })

        this.promises.set(track.id, promise)

        return await promise
    }

    get(track: Track | null | undefined): string | null {
        if (!track) return null
        return this.covers[track.id] ?? null
    }
}

export const trackCovers = new TrackCovers()
