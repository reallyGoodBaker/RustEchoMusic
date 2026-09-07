export type ThemeMode = 'auto' | 'light' | 'dark'

export type PluginLogLevel = 'off' | 'error' | 'warn' | 'info' | 'debug'

export interface AppSettings {
    theme: ThemeMode
    volume: number
    libraryDirs: string[]
    scanOnStartup: boolean
    reduceMotion: boolean
    useAlbumArtistGrouping: boolean
    pluginDirs: string[]
    pluginDevMode: boolean
    pluginScanOnStartup: boolean
    pluginLogLevel: PluginLogLevel
}
