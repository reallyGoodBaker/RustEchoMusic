use std::sync::Arc;

use plugin_runtime::PluginRuntime;
use serde_json::Value;

use crate::errors::AppError;

#[tauri::command]
pub fn plugin_diagnostics(runtime: tauri::State<'_, Arc<PluginRuntime>>) -> Result<Value, AppError> {
    let snapshot = runtime.diagnostics();
    serde_json::to_value(snapshot).map_err(|error| {
        AppError::from(format!("cannot serialize plugin diagnostics: {error}"))
    })
}

#[tauri::command]
pub fn plugin_emit(
    runtime: tauri::State<'_, Arc<PluginRuntime>>,
    kind: String,
    payload: Option<Value>,
) -> Result<Value, AppError> {
    let event_type = plugin_sdk::EventType::new(kind)
        .map_err(|error| AppError::from(format!("invalid event kind: {error}")))?;
    let stats = runtime.emit(event_type, payload.unwrap_or(Value::Null));
    serde_json::to_value(stats)
        .map_err(|error| AppError::from(format!("cannot serialize dispatch stats: {error}")))
}

#[tauri::command]
pub fn plugin_detail(runtime: tauri::State<'_, Arc<PluginRuntime>>, id: String) -> Result<Value, AppError> {
    let plugin_id = plugin_sdk::PluginId::new(id.clone())
        .map_err(|error| AppError::from(format!("invalid plugin id: {error}")))?;
    let snapshot = runtime
        .diagnostics()
        .plugins
        .into_iter()
        .find(|plugin| plugin.id == plugin_id)
        .ok_or_else(|| AppError::from(format!("plugin '{id}' is not installed")))?;
    serde_json::to_value(snapshot)
        .map_err(|error| AppError::from(format!("cannot serialize plugin snapshot: {error}")))
}
