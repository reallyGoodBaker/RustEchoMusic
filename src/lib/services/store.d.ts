export type { Track } from '$lib/types/tracks'

export interface PlayerStoreService {
  savePlaylist(playlist: import('$lib/types/tracks').Track[]): Promise<void>
  loadPlaylist(): Promise<import('$lib/types/tracks').Track[] | null>
  clearPlaylist(): Promise<void>
}

declare module './services/store' {
  export function savePlaylist(playlist: import('$lib/types/tracks').Track[]): Promise<void>
  export function loadPlaylist(): Promise<import('$lib/types/tracks').Track[] | null>
  export function clearPlaylist(): Promise<void>
}
