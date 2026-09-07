#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CommandArgs {
    None,
    TrackId(i64),
    TrackIds(Vec<i64>),
    RawPayload(String),
    LyricsSearch { title: String, artist: String },
}
