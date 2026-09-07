use serde::Serialize;

#[derive(Clone)]
pub struct NativeTrackMetadata {
    pub title: String,
    pub album: String,
    pub artist: String,
    pub duration: Option<f64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackStatusSnapshot {
    pub playing: bool,
    pub current_time: f64,
}