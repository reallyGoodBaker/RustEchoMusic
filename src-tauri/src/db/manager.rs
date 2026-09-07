use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{ConnectOptions, SqlitePool};
use tauri::Manager;

use crate::errors::AppError;

#[derive(Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self, AppError> {
        let database_path = database_path(app_handle)?;

        if let Some(parent) = database_path.parent() {
            let exists = tokio::fs::try_exists(parent).await.unwrap_or(false);
            if !exists {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(AppError::from)?;
            }
        }

        let options = SqliteConnectOptions::new()
            .filename(&database_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .disable_statement_logging();

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(AppError::from)?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(AppError::from)?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn database_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let app_data_path = app_handle.path().app_data_dir().map_err(AppError::from)?;

    Ok(app_data_path.join("music.sqlite"))
}
