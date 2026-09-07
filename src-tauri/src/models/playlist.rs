use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::track::Track;

#[derive(Clone, Debug, Deserialize, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistTrack {
    pub id: i64,
    pub playlist_id: i64,
    pub track_id: i64,
    pub position: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistWithTracks {
    pub playlist: Playlist,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPlaylist {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenamePlaylist {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPlaylistTrack {
    pub playlist_id: i64,
    pub track_id: i64,
    pub position: Option<i64>,
}
