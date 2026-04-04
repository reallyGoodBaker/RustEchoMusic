import { invoke } from '@tauri-apps/api/core'

export async function playMusicFromPath(path: string): Promise<string> {
  return await invoke<string>('play_music_from_path', { path })
}

export async function pauseMusic(): Promise<void> {
  return await invoke<void>('pause_music')
}

export async function resumeMusic(path: string): Promise<string> {
  return await invoke<string>('resume_music', { path })
}

export async function stopMusic(): Promise<void> {
  return await invoke<void>('stop_music')
}
