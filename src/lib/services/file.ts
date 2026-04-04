import { invoke } from '@tauri-apps/api/core'

export interface MusicFileInfo {
  path: string
  title: string
  artist: string
  album: string
  duration_secs: number
  duration_formatted: string
  sample_rate: number
  bit_rate: number
  channels: number
  track_number: number | null
  cover_base64: string
}

export async function selectMusicFolder(): Promise<MusicFileInfo[]> {
  return await invoke<MusicFileInfo[]>('select_music_folder')
}

export async function deleteFile(path: string): Promise<string> {
  return await invoke<string>('delete_file', { path })
}

export async function openContainingFolder(path: string): Promise<void> {
  return await invoke<void>('open_containing_folder', { path })
}
