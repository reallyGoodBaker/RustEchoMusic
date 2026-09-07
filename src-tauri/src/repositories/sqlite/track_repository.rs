use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::errors::AppError;
use crate::models::{NewTrack, SortDirection, Track, TrackSearchQuery, TrackSortBy, UpdateTrack};

#[derive(Clone)]
pub struct SqliteTrackRepository {
    pool: SqlitePool,
}

impl SqliteTrackRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, track: NewTrack) -> Result<Track, AppError> {
        let res = sqlx::query_as::<_, Track>(
            "INSERT INTO tracks (title, artist, album, duration, path, cover, file_size, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), datetime('now'))
             RETURNING id, title, artist, album, duration, path, cover, file_size, play_count, last_played_at, created_at, updated_at",
        )
        .bind(track.title)
        .bind(track.artist)
        .bind(track.album)
        .bind(track.duration)
        .bind(track.path)
        .bind(track.cover)
        .bind(track.file_size)
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn upsert_by_path(&self, track: NewTrack) -> Result<Track, AppError> {
        let res = sqlx::query_as::<_, Track>(
            "INSERT INTO tracks (title, artist, album, duration, path, cover, file_size, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), datetime('now'))
             ON CONFLICT(path) DO UPDATE SET
                title = excluded.title,
                artist = excluded.artist,
                album = excluded.album,
                duration = excluded.duration,
                cover = excluded.cover,
                file_size = excluded.file_size,
                updated_at = datetime('now')
             RETURNING id, title, artist, album, duration, path, cover, file_size, play_count, last_played_at, created_at, updated_at",
        )
        .bind(track.title)
        .bind(track.artist)
        .bind(track.album)
        .bind(track.duration)
        .bind(track.path)
        .bind(track.cover)
        .bind(track.file_size)
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn upsert_tracks(&self, tracks: Vec<NewTrack>) -> Result<Vec<Track>, AppError> {
        let mut tx = self.pool.begin().await?;

        let mut saved = Vec::with_capacity(tracks.len());

        for track in tracks {
            let res = sqlx::query_as::<_, Track>(
                "
                INSERT INTO tracks (
                    title,
                    artist,
                    album,
                    duration,
                    path,
                    cover,
                    file_size,
                    created_at,
                    updated_at
                )
                VALUES (
                    ?1, ?2, ?3,
                    ?4, ?5, ?6,
                    ?7,
                    datetime('now'),
                    datetime('now')
                )

                ON CONFLICT(path)
                DO UPDATE SET
                    title=excluded.title,
                    artist=excluded.artist,
                    album=excluded.album,
                    duration=excluded.duration,
                    cover=excluded.cover,
                    file_size=excluded.file_size,
                    updated_at=datetime('now')

                RETURNING
                    id,
                    title,
                    artist,
                    album,
                    duration,
                    path,
                    cover,
                    file_size,
                    play_count,
                    last_played_at,
                    created_at,
                    updated_at
                ",
            )
            .bind(track.title)
            .bind(track.artist)
            .bind(track.album)
            .bind(track.duration)
            .bind(track.path)
            .bind(track.cover)
            .bind(track.file_size)
            .fetch_one(&mut *tx)
            .await?;

            saved.push(res);
        }

        tx.commit().await?;

        Ok(saved)
    }

    pub async fn update(&self, track: UpdateTrack) -> Result<Track, AppError> {
        let res = sqlx::query_as::<_, Track>(
            "UPDATE tracks SET
                title = ?1,
                artist = ?2,
                album = ?3,
                duration = ?4,
                path = ?5,
                cover = ?6,
                file_size = ?7,
                updated_at = datetime('now')
             WHERE id = ?8
             RETURNING id, title, artist, album, duration, path, cover, file_size, play_count, last_played_at, created_at, updated_at",
        )
        .bind(track.title)
        .bind(track.artist)
        .bind(track.album)
        .bind(track.duration)
        .bind(track.path)
        .bind(track.cover)
        .bind(track.file_size)
        .bind(track.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        sqlx::query("DELETE FROM tracks WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_by_path(&self, path: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM tracks WHERE path = ?1")
            .bind(path)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Track>, AppError> {
        let res = sqlx::query_as::<_, Track>(
            "SELECT id, title, artist, album, duration, path, cover, file_size, play_count, last_played_at, created_at, updated_at
             FROM tracks WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn find_by_path(&self, path: &str) -> Result<Option<Track>, AppError> {
        let res = sqlx::query_as::<_, Track>(
            "SELECT id, title, artist, album, duration, path, cover, file_size, play_count, last_played_at, created_at, updated_at
             FROM tracks WHERE path = ?1",
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn list_all(&self) -> Result<Vec<Track>, AppError> {
        let rows = sqlx::query_as::<_, Track>(
            "SELECT id, title, artist, album, duration, path, cover, file_size, play_count, last_played_at, created_at, updated_at
             FROM tracks ORDER BY title COLLATE NOCASE ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn search(&self, query: TrackSearchQuery) -> Result<Vec<Track>, AppError> {
        let mut builder = QueryBuilder::<Sqlite>::new(
            "SELECT id, title, artist, album, duration, path, cover, file_size, play_count, last_played_at, created_at, updated_at FROM tracks",
        );

        if let Some(keyword) = query
            .keyword
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            let pattern = format!("%{}%", keyword.to_lowercase());
            builder.push(" WHERE lower(title) LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR lower(coalesce(artist, '')) LIKE ");
            builder.push_bind(pattern.clone());
            builder.push(" OR lower(coalesce(album, '')) LIKE ");
            builder.push_bind(pattern);
        }

        builder.push(" ORDER BY ");
        match query.sort_by.unwrap_or(TrackSortBy::Title) {
            TrackSortBy::Title => builder.push("title COLLATE NOCASE"),
            TrackSortBy::Artist => builder.push("artist COLLATE NOCASE"),
            TrackSortBy::Album => builder.push("album COLLATE NOCASE"),
            TrackSortBy::Duration => builder.push("duration"),
            TrackSortBy::CreatedAt => builder.push("created_at"),
            TrackSortBy::UpdatedAt => builder.push("updated_at"),
            TrackSortBy::PlayCount => builder.push("play_count"),
            TrackSortBy::LastPlayedAt => builder.push("last_played_at"),
        };

        match query.sort_direction.unwrap_or(SortDirection::Asc) {
            SortDirection::Asc => builder.push(" ASC"),
            SortDirection::Desc => builder.push(" DESC"),
        };

        builder.push(" LIMIT ");
        builder.push_bind(query.limit.unwrap_or(100).clamp(1, 500));
        builder.push(" OFFSET ");
        builder.push_bind(query.offset.unwrap_or(0).max(0));

        let rows = builder
            .build_query_as::<Track>()
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

    pub async fn increment_play_count(&self, id: i64, played_at: String) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE tracks SET play_count = play_count + 1, last_played_at = ?1, updated_at = datetime('now') WHERE id = ?2",
        )
        .bind(played_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
