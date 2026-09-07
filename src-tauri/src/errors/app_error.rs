use serde::Serialize;

use crate::errors::plugin_error::PluginError;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "message")]
pub enum AppError {
    Database(String),
    Io(String),
    Migration(String),
    Service(String),
    Command(String),
    Platform(String),
    Domain(String),
    Plugin(String),
    PluginPermissionDenied {
        plugin_id: String,
        permission: String,
    },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(message)
            | AppError::Io(message)
            | AppError::Migration(message)
            | AppError::Service(message)
            | AppError::Command(message)
            | AppError::Platform(message)
            | AppError::Domain(message)
            | AppError::Plugin(message) => formatter.write_str(message),
            AppError::PluginPermissionDenied {
                plugin_id,
                permission,
            } => {
                write!(
                    formatter,
                    "Plugin '{}' denied permission {}",
                    plugin_id, permission
                )
            }
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::Database(error.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(error: sqlx::migrate::MigrateError) -> Self {
        AppError::Migration(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Io(error.to_string())
    }
}

impl From<String> for AppError {
    fn from(error: String) -> Self {
        AppError::Service(error)
    }
}

impl From<&str> for AppError {
    fn from(error: &str) -> Self {
        AppError::Service(error.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        AppError::Platform(error.to_string())
    }
}

impl From<PluginError> for AppError {
    fn from(error: PluginError) -> Self {
        match error {
            PluginError::Plugin(msg) => AppError::Plugin(msg),
            PluginError::PermissionDenied { plugin_id, permission } => {
                AppError::PluginPermissionDenied { plugin_id, permission }
            }
            PluginError::Io(msg) => AppError::Io(msg),
        }
    }
}

impl From<plugin_sdk::PluginError> for AppError {
    fn from(error: plugin_sdk::PluginError) -> Self {
        use plugin_sdk::ErrorCode;
        match error.code() {
            ErrorCode::PermissionDenied => AppError::PluginPermissionDenied {
                plugin_id: error.plugin_id().unwrap_or("<host>").to_string(),
                permission: error.message().to_string(),
            },
            ErrorCode::Io | ErrorCode::Timeout | ErrorCode::ServiceUnavailable => {
                AppError::Io(error.to_string())
            }
            ErrorCode::NotFound | ErrorCode::DependencyMissing | ErrorCode::Incompatible => {
                AppError::Command(error.to_string())
            }
            _ => AppError::Plugin(error.to_string()),
        }
    }
}
