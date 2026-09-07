use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::{
    AddPlaylistTrack, NewPlaylist, Playlist, PlaylistTrack, PlaylistWithTracks, RenamePlaylist,
    Track,
};

#[derive(Clone)]
pub struct SqlitePlaylistRepository {
    pool: SqlitePool,
}

impl SqlitePlaylistRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, playlist: NewPlaylist) -> Result<Playlist, AppError> {
        let res = sqlx::query_as::<_, Playlist>(
            "INSERT INTO playlists (name, created_at)
             VALUES (?1, datetime('now'))
             RETURNING id, name, created_at",
        )
        .bind(playlist.name)
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn rename(&self, playlist: RenamePlaylist) -> Result<Playlist, AppError> {
        let res = sqlx::query_as::<_, Playlist>(
            "UPDATE playlists SET name = ?1 WHERE id = ?2 RETURNING id, name, created_at",
        )
        .bind(playlist.name)
        .bind(playlist.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM playlists WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Playlist>, AppError> {
        let res = sqlx::query_as::<_, Playlist>(
            "SELECT id, name, created_at FROM playlists WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn list_all(&self) -> Result<Vec<Playlist>, AppError> {
        let rows = sqlx::query_as::<_, Playlist>(
            "SELECT id, name, created_at FROM playlists ORDER BY created_at DESC, id DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_with_tracks(&self) -> Result<Vec<PlaylistWithTracks>, AppError> {
        let playlists = self.list_all().await?;
        let mut result = Vec::with_capacity(playlists.len());

        for playlist in playlists {
            let tracks = tracks_for_playlist(&self.pool, playlist.id).await?;
            result.push(PlaylistWithTracks { playlist, tracks });
        }

        Ok(result)
    }

    pub async fn get_with_tracks(&self, id: i64) -> Result<Option<PlaylistWithTracks>, AppError> {
        let Some(playlist) = self.find_by_id(id).await? else {
            return Ok(None);
        };

        let tracks = tracks_for_playlist(&self.pool, playlist.id).await?;
        Ok(Some(PlaylistWithTracks { playlist, tracks }))
    }

    pub async fn add_track(&self, track: AddPlaylistTrack) -> Result<PlaylistTrack, AppError> {
        let position = match track.position {
            Some(position) => position,
            None => next_position(&self.pool, track.playlist_id).await?,
        };

        let res = sqlx::query_as::<_, PlaylistTrack>(
            "INSERT INTO playlist_tracks (playlist_id, track_id, position)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(playlist_id, track_id) DO UPDATE SET position = excluded.position
             RETURNING id, playlist_id, track_id, position",
        )
        .bind(track.playlist_id)
        .bind(track.track_id)
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn remove_track(&self, playlist_id: i64, track_id: i64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2")
            .bind(playlist_id)
            .bind(track_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn clear_tracks(&self, playlist_id: i64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?1")
            .bind(playlist_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn reorder_track(
        &self,
        playlist_id: i64,
        track_id: i64,
        position: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE playlist_tracks SET position = ?1 WHERE playlist_id = ?2 AND track_id = ?3",
        )
        .bind(position)
        .bind(playlist_id)
        .bind(track_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

async fn next_position(pool: &SqlitePool, playlist_id: i64) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        "SELECT coalesce(max(position), -1) + 1 FROM playlist_tracks WHERE playlist_id = ?1",
    )
    .bind(playlist_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

async fn tracks_for_playlist(pool: &SqlitePool, playlist_id: i64) -> Result<Vec<Track>, AppError> {
    let rows = sqlx::query_as::<_, Track>(
        "SELECT t.id, t.title, t.artist, t.album, t.duration, t.path, t.cover, t.file_size, t.play_count, t.last_played_at, t.created_at, t.updated_at
         FROM tracks t
         INNER JOIN playlist_tracks pt ON pt.track_id = t.id
         WHERE pt.playlist_id = ?1
         ORDER BY pt.position ASC, pt.id ASC",
    )
    .bind(playlist_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
