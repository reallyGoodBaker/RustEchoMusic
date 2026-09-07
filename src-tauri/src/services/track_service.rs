use std::sync::Arc;

use crate::errors::AppError;
use crate::models::{NewTrack, Track, TrackSearchQuery, UpdateTrack};
use crate::repositories::sqlite::track_repository::SqliteTrackRepository;

#[derive(Clone)]
pub struct TrackService {
    tracks: Arc<SqliteTrackRepository>,
}

impl TrackService {
    pub fn new(tracks: Arc<SqliteTrackRepository>) -> Self {
        Self { tracks }
    }

    pub async fn create_track(&self, track: NewTrack) -> Result<Track, AppError> {
        self.tracks.create(track).await
    }

    pub async fn upsert_track(&self, track: NewTrack) -> Result<Track, AppError> {
        self.tracks.upsert_by_path(track).await
    }

    pub async fn upsert_tracks(&self, tracks: Vec<NewTrack>) -> Result<Vec<Track>, AppError> {
        let mut saved = Vec::with_capacity(tracks.len());
        for track in tracks {
            saved.push(self.tracks.upsert_by_path(track).await?);
        }
        Ok(saved)
    }

    pub async fn update_track(&self, track: UpdateTrack) -> Result<Track, AppError> {
        self.tracks.update(track).await
    }

    pub async fn delete_track(&self, id: i64) -> Result<(), AppError> {
        self.tracks.delete(id).await
    }

    pub async fn delete_track_by_path(&self, path: &str) -> Result<(), AppError> {
        self.tracks.delete_by_path(path).await
    }

    pub async fn get_track(&self, id: i64) -> Result<Option<Track>, AppError> {
        self.tracks.find_by_id(id).await
    }

    pub async fn get_track_by_path(&self, path: &str) -> Result<Option<Track>, AppError> {
        self.tracks.find_by_path(path).await
    }

    pub async fn list_tracks(&self) -> Result<Vec<Track>, AppError> {
        self.tracks.list_all().await
    }

    pub async fn search_tracks(&self, query: TrackSearchQuery) -> Result<Vec<Track>, AppError> {
        self.tracks.search(query).await
    }

    pub async fn mark_track_played(&self, id: i64, played_at: String) -> Result<(), AppError> {
        self.tracks.increment_play_count(id, played_at).await
    }
}
