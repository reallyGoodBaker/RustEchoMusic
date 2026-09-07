use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::SettingRow;

#[derive(Clone)]
pub struct SqliteSettingsRepository {
    pool: SqlitePool,
}

impl SqliteSettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self) -> Result<Option<SettingRow>, AppError> {
        let res = sqlx::query_as::<_, SettingRow>(
            "SELECT
                id,
                theme,
                volume,
                scan_on_startup,
                reduce_motion,
                library_dirs,
                use_album_artist_grouping,
                plugin_dirs,
                plugin_dev_mode,
                plugin_scan_on_startup,
                plugin_log_level,
                created_at,
                updated_at
             FROM settings
             WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn update(&self, row: SettingRow) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO settings (
                id,
                theme,
                volume,
                scan_on_startup,
                reduce_motion,
                library_dirs,
                use_album_artist_grouping,
                plugin_dirs,
                plugin_dev_mode,
                plugin_scan_on_startup,
                plugin_log_level,
                created_at,
                updated_at
             )
             VALUES (
                1,
                ?1,
                ?2,
                ?3,
                ?4,
                ?5,
                ?6,
                ?7,
                ?8,
                ?9,
                ?10,
                coalesce(nullif(?11, ''), datetime('now')),
                datetime('now')
             )
             ON CONFLICT(id) DO UPDATE SET
                theme = excluded.theme,
                volume = excluded.volume,
                scan_on_startup = excluded.scan_on_startup,
                reduce_motion = excluded.reduce_motion,
                library_dirs = excluded.library_dirs,
                use_album_artist_grouping = excluded.use_album_artist_grouping,
                plugin_dirs = excluded.plugin_dirs,
                plugin_dev_mode = excluded.plugin_dev_mode,
                plugin_scan_on_startup = excluded.plugin_scan_on_startup,
                plugin_log_level = excluded.plugin_log_level,
                updated_at = datetime('now')",
        )
        .bind(row.theme)
        .bind(row.volume)
        .bind(row.scan_on_startup)
        .bind(row.reduce_motion)
        .bind(row.library_dirs)
        .bind(row.use_album_artist_grouping)
        .bind(row.plugin_dirs)
        .bind(row.plugin_dev_mode)
        .bind(row.plugin_scan_on_startup)
        .bind(row.plugin_log_level)
        .bind(row.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
