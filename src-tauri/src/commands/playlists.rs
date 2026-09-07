use tauri::{command, State};

use crate::errors::AppError;
use crate::models::{
    AddPlaylistTrack, NewPlaylist, Playlist, PlaylistTrack, PlaylistWithTracks, RenamePlaylist,
};
use crate::state::AppState;

#[command]
pub async fn create_playlist(
    state: State<'_, AppState>,
    playlist: NewPlaylist,
) -> Result<Playlist, AppError> {
    state.playlists.create_playlist(playlist).await
}

#[command]
pub async fn rename_playlist(
    state: State<'_, AppState>,
    playlist: RenamePlaylist,
) -> Result<Playlist, AppError> {
    state.playlists.rename_playlist(playlist).await
}

#[command]
pub async fn delete_playlist(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    state.playlists.delete_playlist(id).await
}

#[command]
pub async fn get_playlist(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<Playlist>, AppError> {
    state.playlists.get_playlist(id).await
}

#[command]
pub async fn list_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, AppError> {
    state.playlists.list_playlists().await
}

#[command]
pub async fn list_playlists_with_tracks(
    state: State<'_, AppState>,
) -> Result<Vec<PlaylistWithTracks>, AppError> {
    state.playlists.list_playlists_with_tracks().await
}

#[command]
pub async fn get_playlist_with_tracks(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<PlaylistWithTracks>, AppError> {
    state.playlists.get_playlist_with_tracks(id).await
}

#[command]
pub async fn add_track_to_playlist(
    state: State<'_, AppState>,
    track: AddPlaylistTrack,
) -> Result<PlaylistTrack, AppError> {
    state.playlists.add_track(track).await
}

#[command]
pub async fn remove_track_from_playlist(
    state: State<'_, AppState>,
    playlist_id: i64,
    track_id: i64,
) -> Result<(), AppError> {
    state.playlists.remove_track(playlist_id, track_id).await
}

#[command]
pub async fn clear_playlist_tracks(
    state: State<'_, AppState>,
    playlist_id: i64,
) -> Result<(), AppError> {
    state.playlists.clear_tracks(playlist_id).await
}

#[command]
pub async fn reorder_playlist_track(
    state: State<'_, AppState>,
    playlist_id: i64,
    track_id: i64,
    position: i64,
) -> Result<(), AppError> {
    state
        .playlists
        .reorder_track(playlist_id, track_id, position)
        .await
}
