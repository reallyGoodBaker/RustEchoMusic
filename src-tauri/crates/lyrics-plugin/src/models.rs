use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LyricLine {
    pub timestamp_ms: u64,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LyricDocument {
    pub song_id: i64,
    pub lines: Vec<LyricLine>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LyricsSearchResult {
    pub song_id: i64,
    pub title: String,
    pub artist: String,
    pub source: String,
}
