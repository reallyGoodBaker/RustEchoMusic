use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Deserialize, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: i64,
    pub path: String,
    pub cover: Option<String>,
    pub file_size: Option<i64>,
    pub play_count: i64,
    pub last_played_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTrack {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: i64,
    pub path: String,
    pub cover: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTrack {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: i64,
    pub path: String,
    pub cover: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TrackSortBy {
    Title,
    Artist,
    Album,
    Duration,
    CreatedAt,
    UpdatedAt,
    PlayCount,
    LastPlayedAt,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackSearchQuery {
    pub keyword: Option<String>,
    pub sort_by: Option<TrackSortBy>,
    pub sort_direction: Option<SortDirection>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
