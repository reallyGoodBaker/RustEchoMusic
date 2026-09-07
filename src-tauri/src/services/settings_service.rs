use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::errors::AppError;
use crate::models::{AppSettings, PluginLogLevel, SettingRow, ThemeMode};
use crate::repositories::sqlite::settings_repository::SqliteSettingsRepository;

#[derive(Clone)]
pub struct SettingsService {
    settings: Arc<SqliteSettingsRepository>,
}

impl SettingsService {
    pub fn new(settings: Arc<SqliteSettingsRepository>) -> Self {
        Self { settings }
    }

    pub async fn get_settings(&self) -> Result<AppSettings, AppError> {
        if let Some(row) = self.settings.get().await? {
            return row_to_settings(row);
        }

        let settings = AppSettings::default();
        self.update_settings(settings.clone()).await?;
        Ok(settings)
    }

    pub async fn update_settings(&self, settings: AppSettings) -> Result<AppSettings, AppError> {
        let row = settings_to_row(settings.clone())?;
        self.settings.update(row).await?;
        Ok(settings)
    }
}

fn row_to_settings(row: SettingRow) -> Result<AppSettings, AppError> {
    Ok(AppSettings {
        theme: ThemeMode::try_from(row.theme).map_err(AppError::Service)?,
        volume: row.volume.clamp(0, 100) as u8,
        scan_on_startup: row.scan_on_startup != 0,
        reduce_motion: row.reduce_motion != 0,
        library_dirs: serde_json::from_str(&row.library_dirs)
            .map_err(|error| AppError::Service(error.to_string()))?,
        use_album_artist_grouping: row.use_album_artist_grouping != 0,
        plugin_dirs: serde_json::from_str(&row.plugin_dirs)
            .map_err(|error| AppError::Service(error.to_string()))?,
        plugin_dev_mode: row.plugin_dev_mode != 0,
        plugin_scan_on_startup: row.plugin_scan_on_startup != 0,
        plugin_log_level: PluginLogLevel::try_from(row.plugin_log_level)
            .map_err(AppError::Service)?,
    })
}

fn settings_to_row(settings: AppSettings) -> Result<SettingRow, AppError> {
    Ok(SettingRow {
        id: 1,
        theme: settings.theme.as_str().to_string(),
        volume: settings.volume as i64,
        scan_on_startup: i64::from(settings.scan_on_startup),
        reduce_motion: i64::from(settings.reduce_motion),
        library_dirs: serde_json::to_string(&settings.library_dirs)
            .map_err(|error| AppError::Service(error.to_string()))?,
        use_album_artist_grouping: i64::from(settings.use_album_artist_grouping),
        plugin_dirs: serde_json::to_string(&settings.plugin_dirs)
            .map_err(|error| AppError::Service(error.to_string()))?,
        plugin_dev_mode: i64::from(settings.plugin_dev_mode),
        plugin_scan_on_startup: i64::from(settings.plugin_scan_on_startup),
        plugin_log_level: settings.plugin_log_level.as_str().to_string(),
        created_at: now_string(),
        updated_at: now_string(),
    })
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
