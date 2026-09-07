use std::sync::Arc;

use crate::errors::AppError;
use crate::models::{
    AddPlaylistTrack, NewPlaylist, Playlist, PlaylistTrack, PlaylistWithTracks, RenamePlaylist,
};
use crate::repositories::sqlite::playlist_repository::SqlitePlaylistRepository;

#[derive(Clone)]
pub struct PlaylistService {
    playlists: Arc<SqlitePlaylistRepository>,
}

impl PlaylistService {
    pub fn new(playlists: Arc<SqlitePlaylistRepository>) -> Self {
        Self { playlists }
    }

    pub async fn create_playlist(&self, playlist: NewPlaylist) -> Result<Playlist, AppError> {
        self.playlists.create(playlist).await
    }

    pub async fn rename_playlist(&self, playlist: RenamePlaylist) -> Result<Playlist, AppError> {
        self.playlists.rename(playlist).await
    }

    pub async fn delete_playlist(&self, id: i64) -> Result<(), AppError> {
        self.playlists.delete(id).await
    }

    pub async fn get_playlist(&self, id: i64) -> Result<Option<Playlist>, AppError> {
        self.playlists.find_by_id(id).await
    }

    pub async fn list_playlists(&self) -> Result<Vec<Playlist>, AppError> {
        self.playlists.list_all().await
    }

    pub async fn list_playlists_with_tracks(&self) -> Result<Vec<PlaylistWithTracks>, AppError> {
        self.playlists.list_with_tracks().await
    }

    pub async fn get_playlist_with_tracks(
        &self,
        id: i64,
    ) -> Result<Option<PlaylistWithTracks>, AppError> {
        self.playlists.get_with_tracks(id).await
    }

    pub async fn add_track(&self, track: AddPlaylistTrack) -> Result<PlaylistTrack, AppError> {
        self.playlists.add_track(track).await
    }

    pub async fn remove_track(&self, playlist_id: i64, track_id: i64) -> Result<(), AppError> {
        self.playlists.remove_track(playlist_id, track_id).await
    }

    pub async fn clear_tracks(&self, playlist_id: i64) -> Result<(), AppError> {
        self.playlists.clear_tracks(playlist_id).await
    }

    pub async fn reorder_track(
        &self,
        playlist_id: i64,
        track_id: i64,
        position: i64,
    ) -> Result<(), AppError> {
        self.playlists
            .reorder_track(playlist_id, track_id, position)
            .await
    }
}
