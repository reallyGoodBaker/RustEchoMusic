use serde::Serialize;

use crate::models::{AppSettings, PlaybackQueue, Track};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStatePayload {
    pub playing: bool,
    pub current_time: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackProgressPayload {
    pub current_time: f64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackStartedPayload {
    pub track: Track,
    pub index: usize,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLinePayload {
    pub timestamp_ms: u64,
    pub text: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsLoadedPayload {
    pub song_id: i64,
    pub lines: Vec<LyricLinePayload>,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    VolumeChanged(f32),

    SettingsChanged(AppSettings),

    QueueChanged(PlaybackQueue),

    PlaybackStateChanged(PlaybackStatePayload),

    PlaybackProgress(PlaybackProgressPayload),

    TrackStarted(TrackStartedPayload),

    LyricsLoaded(LyricsLoadedPayload),
}
