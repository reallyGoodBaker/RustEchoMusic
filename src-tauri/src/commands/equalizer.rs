use std::sync::Arc;

use plugin_runtime::PluginRuntime;
use plugin_sdk::CommandId;
use serde_json::{json, Value};
use tauri::State;

use crate::errors::AppError;

#[derive(serde::Serialize)]
pub struct EqState {
    pub bands: [f64; 10],
    pub enabled: bool,
    pub preset_name: String,
}

fn call(runtime: &PluginRuntime, command: &str, args: Value) -> Result<Value, AppError> {
    let command =
        CommandId::new(command).map_err(|error| AppError::Command(error.to_string()))?;
    runtime.execute(&command, args).map_err(AppError::from)
}

#[tauri::command]
pub async fn get_eq_state(
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<EqState, AppError> {
    let value = call(&runtime, "eq.getState", Value::Null)?;
    let bands: [f64; 10] = serde_json::from_value(
        value.get("bands").cloned().unwrap_or(json!([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
    )
    .unwrap_or([0.0; 10]);
    let enabled = value.get("enabled").and_then(Value::as_bool).unwrap_or(false);
    let preset_name = value
        .get("presetName")
        .and_then(Value::as_str)
        .unwrap_or("Flat")
        .to_string();
    Ok(EqState {
        bands,
        enabled,
        preset_name,
    })
}

#[tauri::command]
pub async fn set_eq_band(
    band: usize,
    gain: f64,
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<(), AppError> {
    call(
        &runtime,
        "eq.setBand",
        json!({ "band": band, "gain": gain }),
    )
    .map(|_| ())
}

#[tauri::command]
pub async fn apply_eq_preset(
    preset_name: String,
    bands: [f64; 10],
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<(), AppError> {
    call(
        &runtime,
        "eq.applyPreset",
        json!({ "presetName": preset_name, "bands": bands }),
    )
    .map(|_| ())
}

#[tauri::command]
pub async fn set_eq_enabled(
    enabled: bool,
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<(), AppError> {
    call(&runtime, "eq.setEnabled", json!({ "enabled": enabled })).map(|_| ())
}
