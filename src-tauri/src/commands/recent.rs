use std::sync::Arc;

use plugin_runtime::PluginRuntime;
use plugin_sdk::CommandId;
use serde_json::{json, Value};
use tauri::State;

use crate::errors::AppError;

fn call(runtime: &PluginRuntime, command: &str, args: Value) -> Result<Value, AppError> {
    let command = CommandId::new(command).map_err(|error| AppError::Command(error.to_string()))?;
    runtime.execute(&command, args).map_err(AppError::from)
}

#[tauri::command]
pub async fn load_recently_played(
    runtime: State<'_, Arc<PluginRuntime>>,
    limit: i64,
    offset: i64,
) -> Result<Value, AppError> {
    call(
        &runtime,
        "recent.list",
        json!({ "limit": limit, "offset": offset }),
    )
}

#[tauri::command]
pub async fn add_recently_played(
    runtime: State<'_, Arc<PluginRuntime>>,
    track_id: i64,
    played_at: String,
) -> Result<(), AppError> {
    call(
        &runtime,
        "recent.add",
        json!({ "trackId": track_id, "playedAt": played_at }),
    )
    .map(|_| ())
}

#[tauri::command]
pub async fn clear_recently_played(
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<(), AppError> {
    call(&runtime, "recent.clear", Value::Null).map(|_| ())
}
