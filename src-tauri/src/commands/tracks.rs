use tauri::{command, State};

use crate::errors::AppError;
use crate::models::{NewTrack, Track, TrackSearchQuery, UpdateTrack};
use crate::state::AppState;

#[command]
pub async fn create_track(state: State<'_, AppState>, track: NewTrack) -> Result<Track, AppError> {
    state.tracks.create_track(track).await
}

#[command]
pub async fn upsert_track(state: State<'_, AppState>, track: NewTrack) -> Result<Track, AppError> {
    state.tracks.upsert_track(track).await
}

#[command]
pub async fn update_track(
    state: State<'_, AppState>,
    track: UpdateTrack,
) -> Result<Track, AppError> {
    state.tracks.update_track(track).await
}

#[command]
pub async fn delete_track(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    state.tracks.delete_track(id).await
}

#[command]
pub async fn delete_track_by_path(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
    state.tracks.delete_track_by_path(&path).await
}

#[command]
pub async fn get_track(state: State<'_, AppState>, id: i64) -> Result<Option<Track>, AppError> {
    state.tracks.get_track(id).await
}

#[command]
pub async fn get_track_by_path(
    state: State<'_, AppState>,
    path: String,
) -> Result<Option<Track>, AppError> {
    state.tracks.get_track_by_path(&path).await
}

#[command]
pub async fn list_tracks(state: State<'_, AppState>) -> Result<Vec<Track>, AppError> {
    state.tracks.list_tracks().await
}

#[command]
pub async fn search_tracks(
    state: State<'_, AppState>,
    query: TrackSearchQuery,
) -> Result<Vec<Track>, AppError> {
    state.tracks.search_tracks(query).await
}

#[command]
pub async fn mark_track_played(
    state: State<'_, AppState>,
    id: i64,
    played_at: String,
) -> Result<(), AppError> {
    state.tracks.mark_track_played(id, played_at).await
}
