import type { AppSettings } from '$lib/types'
import { invoke } from '@tauri-apps/api/core'

export const DEFAULT_SETTINGS: AppSettings = {
    theme: 'auto',
    volume: 80,
    libraryDirs: [],
    scanOnStartup: false,
    reduceMotion: false,
    useAlbumArtistGrouping: false,
    pluginDirs: [],
    pluginDevMode: false,
    pluginScanOnStartup: true,
    pluginLogLevel: 'warn',
}

class SettingsState {
    data = $state<AppSettings>({ ...DEFAULT_SETTINGS })
    isLoading = $state(false)
    error = $state<string | null>(null)

    #saveTimer: ReturnType<typeof setTimeout> | null = null
    #loaded = false

    async load(): Promise<AppSettings> {
        if (this.#loaded) {
            return this.data
        }

        this.#loaded = true
        this.isLoading = true
        this.error = null

        try {
            this.data = await invoke<AppSettings>('load_settings')
            return this.data
        } catch (error) {
            console.error(error)
            this.error = String(error)
            return this.data
        } finally {
            this.isLoading = false
        }
    }

    updateLocalState(newSettings: AppSettings) {
        this.data = newSettings
    }

    update(patch: Partial<AppSettings>) {
        const nextSettings = {
            ...this.data,
            ...patch,
        }
        this.data = nextSettings
        this.scheduleSave(nextSettings)
    }

    async save(targetSettings: AppSettings): Promise<void> {
        this.error = null

        try {
            const savedSettings = await invoke<AppSettings>('save_settings', {
                settings: targetSettings,
            })
            this.data = savedSettings
        } catch (error) {
            console.error(error)
            this.error = String(error)
        }
    }

    scheduleSave(targetSettings: AppSettings) {
        if (this.#saveTimer) {
            clearTimeout(this.#saveTimer)
        }

        this.#saveTimer = setTimeout(() => {
            void this.save(targetSettings)
        }, 300)
    }

    addLibraryDir(dir: string) {
        const value = dir.trim()

        if (!value) return
        if (this.data.libraryDirs.includes(value)) return

        this.update({
            libraryDirs: [...this.data.libraryDirs, value],
        })
    }

    removeLibraryDir(dir: string) {
        this.update({
            libraryDirs: this.data.libraryDirs.filter((item) => item !== dir),
        })
    }
}

export const settings = new SettingsState()
