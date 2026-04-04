import { load } from '@tauri-apps/plugin-store'
import type { Track } from '$lib/types/tracks'

let store: Awaited<ReturnType<typeof load>> | null = null

const PLAYLIST_KEY = 'player-playlist'

async function getStore() {
  if (!store) {
    store = await load('player-state.json', { autoSave: false, defaults: {} })
  }
  return store
}

export async function savePlaylist(playlist: Track[]): Promise<void> {
  try {
    const s = await getStore()
    await s.set(PLAYLIST_KEY, playlist)
    await s.save()
  } catch (error) {
    console.error('Failed to save playlist to store:', error)
    throw error
  }
}

export async function loadPlaylist(): Promise<Track[] | null> {
  try {
    const s = await getStore()
    const playlist = await s.get<Track[]>(PLAYLIST_KEY)

    if (!playlist || !Array.isArray(playlist)) {
      return null
    }

    const isValid = playlist.every(
      track =>
        track &&
        typeof track.id === 'number' &&
        typeof track.title === 'string' &&
        typeof track.path === 'string',
    )

    if (!isValid) {
      console.warn('Invalid playlist data found, clearing...')
      await s.delete(PLAYLIST_KEY)
      await s.save()
      return null
    }

    return playlist as Track[]
  } catch (error) {
    console.error('Failed to load playlist from store:', error)
    return null
  }
}

export async function clearPlaylist(): Promise<void> {
  try {
    const s = await getStore()
    await s.delete(PLAYLIST_KEY)
    await s.save()
  } catch (error) {
    console.error('Failed to clear playlist from store:', error)
    throw error
  }
}
