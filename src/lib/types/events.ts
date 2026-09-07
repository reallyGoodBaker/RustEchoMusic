import type { PlaybackQueue } from "./music"
import type { LyricLine } from "./lyrics"
import type { PlaybackProgressPayload, PlaybackStatePayload, PlaybackTrackStartedPayload } from "./playback"
import type { AppSettings } from "./settings"

export type GlobalAppEvent =
    | { type: 'VolumeChanged'; payload: number }
    | { type: 'SettingsChanged'; payload: AppSettings }
    | { type: 'TrackStarted'; payload: PlaybackTrackStartedPayload }
    | { type: 'PlaybackProgress'; payload: PlaybackProgressPayload }
    | { type: 'QueueChanged'; payload: PlaybackQueue }
    | { type: 'PlaybackStateChanged'; payload: PlaybackStatePayload }
    | { type: 'LyricsLoaded'; payload: { songId: number; lines: LyricLine[] } }