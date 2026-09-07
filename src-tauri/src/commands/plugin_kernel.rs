use std::sync::Arc;

use plugin_runtime::PluginRuntime;
use plugin_runtime::contributions::RichContribution;
use plugin_sdk::{CommandId, ContributionPointId, DeactivateReason, PluginId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionDto {
    pub point: String,
    pub plugin: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDto {
    pub id: String,
    pub version: String,
    pub source: String,
    pub tier: String,
    pub user_disableable: bool,
    pub state: String,
    pub health: String,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KernelSnapshotDto {
    pub host_version: String,
    pub plugins: Vec<PluginDto>,
    pub contributions: Vec<ContributionDto>,
    pub issues: Vec<String>,
}

#[tauri::command]
pub fn plugin_invoke(
    runtime: State<'_, Arc<PluginRuntime>>,
    command: String,
    args: Option<Value>,
) -> Result<Value, AppError> {
    let command =
        CommandId::new(command).map_err(|error| AppError::Command(error.to_string()))?;
    runtime
        .execute(&command, args.unwrap_or(Value::Null))
        .map_err(AppError::from)
}

#[tauri::command]
pub fn plugin_contributions(
    runtime: State<'_, Arc<PluginRuntime>>,
    point: Option<String>,
) -> Result<Vec<ContributionDto>, AppError> {
    let snapshot = runtime.diagnostics();
    let items = match point {
        Some(point) => {
            let point = ContributionPointId::new(point)
                .map_err(|error| AppError::Command(error.to_string()))?;
            snapshot
                .contributions
                .into_iter()
                .filter(|item| item.point == point)
                .collect()
        }
        None => snapshot.contributions,
    };
    Ok(items
        .into_iter()
        .map(|item| ContributionDto {
            point: item.point.to_string(),
            plugin: item.plugin.to_string(),
            key: item.key,
        })
        .collect())
}

#[tauri::command]
pub fn plugin_kernel_snapshot(
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<KernelSnapshotDto, AppError> {
    let snapshot = runtime.diagnostics();
    let plugins = snapshot
        .plugins
        .iter()
        .map(|plugin| PluginDto {
            id: plugin.id.to_string(),
            version: plugin.version.to_string(),
            source: plugin.source.as_str().to_string(),
            tier: plugin.tier.as_str().to_string(),
            user_disableable: plugin.user_disableable,
            state: plugin.state.label().to_string(),
            health: format!("{:?}", plugin.health),
            active: runtime.is_active(&plugin.id),
            last_error: plugin.last_error.clone(),
        })
        .collect();
    Ok(KernelSnapshotDto {
        host_version: snapshot.host_version.to_string(),
        plugins,
        contributions: snapshot
            .contributions
            .into_iter()
            .map(|item| ContributionDto {
                point: item.point.to_string(),
                plugin: item.plugin.to_string(),
                key: item.key,
            })
            .collect(),
        issues: snapshot
            .issues
            .into_iter()
            .map(|issue| format!("{}: {}", issue.path.display(), issue.message))
            .collect(),
    })
}

#[tauri::command]
pub fn plugin_contributions_full(
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<Vec<RichContribution>, AppError> {
    Ok(runtime.contributions().rich_snapshot())
}

#[tauri::command]
pub fn plugin_set_enabled(
    runtime: State<'_, Arc<PluginRuntime>>,
    plugin: String,
    enabled: bool,
) -> Result<(), AppError> {
    let id = PluginId::new(plugin).map_err(|error| AppError::Command(error.to_string()))?;
    if enabled {
        runtime.activate(&id).map_err(AppError::from)
    } else {
        runtime
            .deactivate(&id, DeactivateReason::UserDisabled)
            .map_err(AppError::from)
    }
}

#[tauri::command]
pub fn plugin_reload(
    runtime: State<'_, Arc<PluginRuntime>>,
    plugin: String,
) -> Result<(), AppError> {
    let id = PluginId::new(plugin).map_err(|error| AppError::Command(error.to_string()))?;
    runtime.reload(&id).map_err(AppError::from)
}

#[tauri::command]
pub fn plugin_uninstall(
    runtime: State<'_, Arc<PluginRuntime>>,
    plugin: String,
) -> Result<Vec<String>, AppError> {
    let id = PluginId::new(plugin).map_err(|error| AppError::Command(error.to_string()))?;
    let affected = runtime.uninstall(&id).map_err(AppError::from)?;
    Ok(affected.into_iter().map(|id| id.to_string()).collect())
}

#[tauri::command]
pub fn plugin_discover(runtime: State<'_, Arc<PluginRuntime>>) -> Result<KernelSnapshotDto, AppError> {
    let report = runtime.discover();
    for issue in &report.issues {
        eprintln!(
            "[plugin-kernel] discovery issue at {}: {}",
            issue.path.display(),
            issue.message
        );
    }
    runtime.activate_all();
    plugin_kernel_snapshot(runtime)
}
