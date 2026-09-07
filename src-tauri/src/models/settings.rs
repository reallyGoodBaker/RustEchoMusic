use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: ThemeMode,
    pub volume: u8,
    pub library_dirs: Vec<String>,
    pub scan_on_startup: bool,
    pub reduce_motion: bool,
    pub use_album_artist_grouping: bool,
    pub plugin_dirs: Vec<String>,
    pub plugin_dev_mode: bool,
    pub plugin_scan_on_startup: bool,
    pub plugin_log_level: PluginLogLevel,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Auto,
            volume: 80,
            library_dirs: Vec::new(),
            scan_on_startup: false,
            reduce_motion: false,
            use_album_artist_grouping: false,
            plugin_dirs: Vec::new(),
            plugin_dev_mode: false,
            plugin_scan_on_startup: true,
            plugin_log_level: PluginLogLevel::Warn,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    Auto,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Auto => "auto",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }
}

impl TryFrom<String> for ThemeMode {
    type Error = String;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        match value.as_str() {
            "auto" => Ok(ThemeMode::Auto),
            "light" => Ok(ThemeMode::Light),
            "dark" => Ok(ThemeMode::Dark),
            _ => Err(format!("无效主题: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginLogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
}

impl PluginLogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginLogLevel::Off => "off",
            PluginLogLevel::Error => "error",
            PluginLogLevel::Warn => "warn",
            PluginLogLevel::Info => "info",
            PluginLogLevel::Debug => "debug",
        }
    }
}

impl TryFrom<String> for PluginLogLevel {
    type Error = String;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        match value.as_str() {
            "off" => Ok(PluginLogLevel::Off),
            "error" => Ok(PluginLogLevel::Error),
            "warn" => Ok(PluginLogLevel::Warn),
            "info" => Ok(PluginLogLevel::Info),
            "debug" => Ok(PluginLogLevel::Debug),
            _ => Err(format!("无效插件日志级别: {}", value)),
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct SettingRow {
    pub id: i64,
    pub theme: String,
    pub volume: i64,
    pub scan_on_startup: i64,
    pub reduce_motion: i64,
    pub library_dirs: String,
    pub use_album_artist_grouping: i64,
    pub plugin_dirs: String,
    pub plugin_dev_mode: i64,
    pub plugin_scan_on_startup: i64,
    pub plugin_log_level: String,
    pub created_at: String,
    pub updated_at: String,
}
