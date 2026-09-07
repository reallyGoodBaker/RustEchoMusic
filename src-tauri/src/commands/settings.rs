use crate::errors::AppError;
use crate::events::{AppEvent, EventBus};
use crate::models::AppSettings;
use crate::state::AppState;
use tauri::{command, AppHandle, State};

#[command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    state.settings.get_settings().await
}

#[command]
pub async fn update_settings(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, AppError> {
    let updated_settings = state.settings.update_settings(settings).await?;

    EventBus::emit(
        &app_handle,
        AppEvent::SettingsChanged(updated_settings.clone()),
    )?;

    Ok(updated_settings)
}

#[command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    state.settings.get_settings().await
}

#[command]
pub async fn save_settings(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, AppError> {
    let updated_settings = state.settings.update_settings(settings).await?;

    EventBus::emit(
        &app_handle,
        AppEvent::SettingsChanged(updated_settings.clone()),
    )?;

    Ok(updated_settings)
}
