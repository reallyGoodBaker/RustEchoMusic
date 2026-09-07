import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { SvelteMap } from 'svelte/reactivity'
import type { Album, Artist, Track } from '../types/music'
import { settings } from './settings.svelte'

class MusicLibrary {
    tracks = $state<Track[]>([])
    isLoading = $state(false)
    error = $state<string | null>(null)
    initialized = false

    get useAlbumArtistGrouping() {
        return settings.data.useAlbumArtistGrouping ?? false
    }

    #unlisten: UnlistenFn | null = null
    
    #refreshPromise: Promise<Track[]> | null = null
    #albumsMap = $derived.by(() => {
        const albumMap = new SvelteMap<string, Album>()
        const useAlbumArtistGrouping = this.useAlbumArtistGrouping

        for (const track of this.tracks) {
            const albumTitle = track.album?.trim() || '未知专辑'

            const artistName = track.artist?.trim() || '未知歌手'
            const albumKey = useAlbumArtistGrouping
                ? `${artistName}_${albumTitle}`.toLowerCase()
                : albumTitle.toLowerCase()
            const id = encodeURIComponent(albumKey)

            if (!albumMap.has(id)) {
                albumMap.set(id, {
                    id,
                    title: albumTitle,
                    artist: artistName,
                    tracks: [],
                    cover: track.cover ?? null,
                    trackCount: 0,
                    representativeTrack: track,
                })
            }

            const album = albumMap.get(id)!
            album.tracks.push(track)
            album.trackCount = album.tracks.length

            if (!album.cover && track.cover) {
                album.cover = track.cover
            }
        }

        return albumMap
    })

    #artistsMap = $derived.by(() => {
        const artistMap = new SvelteMap<
            string,
            Artist & {
                albumKeys: Set<string>
            }
        >()

        for (const track of this.tracks) {
            const artistName = track.artist?.trim() || '未知歌手'
            const artistKey = artistName.toLowerCase()
            const artistId = encodeURIComponent(artistKey)

            if (!artistMap.has(artistId)) {
                artistMap.set(artistId, {
                    id: artistId,
                    name: artistName,
                    cover: track.cover ?? null,
                    trackCount: 0,
                    albumCount: 0,
                    tracks: [],
                    albumKeys: new Set(),
                })
            }

            const artist = artistMap.get(artistId)!

            artist.tracks.push(track)
            artist.trackCount = artist.tracks.length

            const albumTitle = track.album?.trim() || '未知专辑'
            const albumKey = albumTitle.toLowerCase()

            artist.albumKeys.add(albumKey)
            artist.albumCount = artist.albumKeys.size

            if (!artist.cover && track.cover) {
                artist.cover = track.cover
            }
        }

        return new SvelteMap(
            Array.from(artistMap.values()).map(({ albumKeys, ...artist }) => [
                artist.id,
                artist,
            ]),
        )
    })

    constructor() {
        void this.#setupListener()
    }

    get albums(): Album[] {
        return Array.from(this.#albumsMap.values())
    }

    get artists(): Artist[] {
        return Array.from(this.#artistsMap.values())
    }

    get albumCount(): number {
        return this.#albumsMap.size
    }

    get artistCount(): number {
        return this.#artistsMap.size
    }

    get trackCount(): number {
        return this.tracks.length
    }

    getAlbum(id: string): Album | undefined {
        return this.#albumsMap.get(id)
    }

    getArtist(id: string): Artist | undefined {
        return this.#artistsMap.get(id)
    }

    async #setupListener() {
        if (this.#unlisten) return
        
        this.#unlisten = await listen<Track[]>('library:refreshed', event => {
            this.tracks = event.payload
        })
    }

    async load(options: { force?: boolean } = {}): Promise<Track[]> {
        const { force = false } = options

        if (this.#refreshPromise) {
            return this.#refreshPromise
        }

        if (!force && this.tracks.length > 0) {
            return this.tracks
        }

        this.isLoading = true
        this.error = null

        this.#refreshPromise = (async () => {
            try {
                const tracks = await invoke<Track[]>('load_track_library')
                this.tracks = tracks
                return tracks
            } catch (err) {
                console.error('全局加载媒体库失败:', err)
                this.error = String(err)
                return this.tracks
            } finally {
                this.isLoading = false
                this.#refreshPromise = null
            }
        })()

        return this.#refreshPromise
    }

    async scan(): Promise<Track[]> {
        try {
            this.isLoading = true
            this.error = null

            const tracks = await invoke<Track[]>('scan_track_directories', {
                dirs: settings.data.libraryDirs,
            })

            this.tracks = tracks

            return tracks
        } catch (err) {
            console.error('扫描媒体库失败:', err)
            this.error = String(err)
            return this.tracks
        } finally {
            this.isLoading = false
        }
    }
    
    destroy() {
        if (this.#unlisten) {
            this.#unlisten()
            this.#unlisten = null
        }
    }
}

export const musicLibrary = new MusicLibrary()