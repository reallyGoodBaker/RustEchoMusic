use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::{RecentPlayedWithTrack, Track};

#[derive(Clone)]
pub struct SqliteRecentRepository {
    pool: SqlitePool,
}

impl SqliteRecentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_with_tracks(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RecentPlayedWithTrack>, AppError> {
        let rows = sqlx::query_as::<_, RecentRow>(
            "SELECT t.id, t.title, t.artist, t.album, t.duration, t.path, t.cover, t.file_size, t.play_count, t.last_played_at, t.created_at, t.updated_at, r.played_at
             FROM recent_played r
             INNER JOIN tracks t ON t.id = r.track_id
             ORDER BY r.played_at DESC
             LIMIT ?1 OFFSET ?2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| RecentPlayedWithTrack {
                track: Track {
                    id: row.id,
                    title: row.title,
                    artist: row.artist,
                    album: row.album,
                    duration: row.duration,
                    path: row.path,
                    cover: row.cover,
                    file_size: row.file_size,
                    play_count: row.play_count,
                    last_played_at: row.last_played_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                },
                played_at: row.played_at,
            })
            .collect())
    }

    pub async fn upsert(&self, track_id: i64, played_at: String) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO recent_played (track_id, played_at)
             VALUES (?1, ?2)
             ON CONFLICT(track_id) DO UPDATE SET played_at = excluded.played_at",
        )
        .bind(track_id)
        .bind(played_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn clear(&self) -> Result<(), AppError> {
        sqlx::query("DELETE FROM recent_played")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn count(&self) -> Result<i64, AppError> {
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM recent_played")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }

    pub async fn remove_oldest(&self, keep: i64) -> Result<(), AppError> {
        sqlx::query(
            "DELETE FROM recent_played WHERE track_id NOT IN (
                SELECT track_id FROM recent_played ORDER BY played_at DESC LIMIT ?1
             )",
        )
        .bind(keep)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct RecentRow {
    id: i64,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    duration: i64,
    path: String,
    cover: Option<String>,
    file_size: Option<i64>,
    play_count: i64,
    last_played_at: Option<String>,
    created_at: String,
    updated_at: String,
    played_at: String,
}
