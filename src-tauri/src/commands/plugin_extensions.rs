use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{command, State};

use crate::errors::AppError;
use crate::services::plugin::extension::menu_extension::MenuExtension;
use crate::services::plugin::extension::native_view_extension::NativeViewExtension;
use crate::services::plugin::extension::sidebar_extension::SidebarExtension;
use crate::services::plugin::extension::state::ExtensionState;
use crate::services::plugin::runtime::state::PluginLifecycleState;
use crate::services::plugin::settings::persistence::{JsonPluginStorage, PluginStorage};
use crate::services::plugin::settings::settings_registry::SettingsRegistry;
use crate::services::plugin::kernel::host_services::json_to_setting;
use plugin_runtime::PluginRuntime;
use crate::services::plugin::dto::MenuLocation;
use crate::services::plugin::dto::{
    ActivationEvent, PluginContribution, PluginSource, ResolvedPluginManifest,
};
use crate::services::plugin::settings::value::{PluginSettingMeta, SettingValue};
use plugin_sdk::{CommandArgs, Contribution, CommandId, DeactivateReason, Manifest, PluginId};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginViewResolution {
    pub plugin_id: String,
    pub view_id: String,
    pub title: String,
    pub entry_url: String,
}

fn state_for(rt: &Arc<PluginRuntime>, plugin: &PluginId) -> ExtensionState {
    if rt.is_active(plugin) {
        ExtensionState::Enabled
    } else {
        ExtensionState::Disabled
    }
}

fn sidebar_extensions(rt: &Arc<PluginRuntime>) -> Vec<SidebarExtension> {
    rt.contributions()
        .rich_snapshot()
        .into_iter()
        .filter(|c| c.point.as_str() == "ui.sidebar")
        .filter_map(|c| {
            let Ok(contrib) = serde_json::from_value::<Contribution>(c.payload.clone()) else {
                return None;
            };
            let Contribution::SidebarItem(spec) = contrib else {
                return None;
            };
            Some(SidebarExtension {
                id: spec.id.clone(),
                plugin_id: c.plugin.to_string(),
                title: spec.title.clone(),
                icon: spec.icon.clone(),
                route: spec.target.clone(),
                state: state_for(rt, &c.plugin),
            })
        })
        .collect()
}

fn native_view_extensions(rt: &Arc<PluginRuntime>) -> Vec<NativeViewExtension> {
    rt.contributions()
        .rich_snapshot()
        .into_iter()
        .filter(|c| c.point.as_str() == "ui.nativeView")
        .filter_map(|c| {
            let Ok(contrib) = serde_json::from_value::<Contribution>(c.payload.clone()) else {
                return None;
            };
            let Contribution::NativeView(spec) = contrib else {
                return None;
            };
            Some(NativeViewExtension {
                id: spec.id.clone(),
                plugin_id: c.plugin.to_string(),
                title: spec.title.clone(),
                token: spec.token.clone(),
                icon: spec.icon.clone(),
                state: state_for(rt, &c.plugin),
            })
        })
        .collect()
}

fn menu_location_from_str(value: &str) -> Option<MenuLocation> {
    match value {
        "track.context" | "TrackContextMenu" => Some(MenuLocation::TrackContextMenu),
        "sidebar.actions" | "SidebarActions" => Some(MenuLocation::SidebarActions),
        _ => None,
    }
}

fn menu_extensions(location: MenuLocation, rt: &Arc<PluginRuntime>) -> Vec<MenuExtension> {
    rt.contributions()
        .rich_snapshot()
        .into_iter()
        .filter(|c| c.point.as_str() == "ui.menuItem")
        .filter_map(|c| {
            let Ok(contrib) = serde_json::from_value::<Contribution>(c.payload.clone()) else {
                return None;
            };
            let id = contrib.key();
            let Contribution::MenuItem(spec) = contrib else {
                return None;
            };
            let Some(loc) = menu_location_from_str(&spec.location) else {
                return None;
            };
            if loc != location {
                return None;
            }
            Some(MenuExtension {
                id,
                plugin_id: c.plugin.to_string(),
                command: spec.command.to_string(),
                location,
                group: spec.group.clone(),
                state: state_for(rt, &c.plugin),
            })
        })
        .collect()
}

fn activation_event_from_str(value: &str) -> ActivationEvent {
    match serde_json::from_value::<ActivationEvent>(serde_json::Value::String(value.to_string())) {
        Ok(event) => event,
        Err(_) => ActivationEvent::OnCommand(value.to_string()),
    }
}

fn resolve_manifest(m: Manifest) -> ResolvedPluginManifest {
    let source = match m.source {
        plugin_sdk::manifest::PluginSource::Builtin => PluginSource::Builtin,
        plugin_sdk::manifest::PluginSource::Packaged => PluginSource::Packaged,
        plugin_sdk::manifest::PluginSource::User => PluginSource::User,
    };
    let settings = m
        .settings
        .into_iter()
        .map(|s| crate::services::plugin::dto::SettingDefinition {
            key: s.key,
            title: s.title,
            default_value: json_to_setting(s.default_value),
        })
        .collect();
    ResolvedPluginManifest {
        id: m.id.to_string(),
        source,
        route: format!("/plugins/view/{}", m.id),
        name: m.name,
        display_name: m.display_name,
        version: m.version.to_string(),
        author: m.author,
        description: m.description,
        entry: m.entry,
        min_app_version: m.min_host.to_string(),
        permissions: Vec::new(),
        activation_events: m
            .activation
            .events
            .iter()
            .map(|e| activation_event_from_str(e.as_str()))
            .collect(),
        contributes: PluginContribution::default(),
        settings,
    }
}

fn command_args_to_value(args: &CommandArgs) -> Value {
    match args {
        CommandArgs::None => Value::Null,
        CommandArgs::TrackId(id) => json!({ "songId": id }),
        CommandArgs::TrackIds(ids) => json!({ "songIds": ids }),
        CommandArgs::RawPayload(s) => {
            serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.clone()))
        }
        CommandArgs::LyricsSearch { title, artist } => {
            json!({ "title": title, "artist": artist })
        }
    }
}

#[command]
pub async fn get_menu_extensions(
    location: MenuLocation,
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<Vec<MenuExtension>, AppError> {
    Ok(menu_extensions(location, runtime.inner()))
}

#[command]
pub async fn get_sidebar_extensions(
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<Vec<SidebarExtension>, AppError> {
    Ok(sidebar_extensions(runtime.inner()))
}

#[command]
pub async fn get_all_sidebar_extensions(
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<Vec<SidebarExtension>, AppError> {
    Ok(sidebar_extensions(runtime.inner()))
}

#[command]
pub async fn get_native_view_extensions(
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<Vec<NativeViewExtension>, AppError> {
    Ok(native_view_extensions(runtime.inner()))
}

#[command]
pub async fn get_plugin_manifests(
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<Vec<ResolvedPluginManifest>, AppError> {
    Ok(runtime
        .inner()
        .plugin_manifests()
        .into_iter()
        .map(resolve_manifest)
        .collect())
}

#[command]
pub async fn get_plugin_view(
    plugin_id: String,
    view_id: String,
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<Option<PluginViewResolution>, AppError> {
    for c in runtime.inner().contributions().rich_snapshot() {
        if c.point.as_str() != "ui.view" {
            continue;
        }
        if c.plugin.to_string() != plugin_id {
            continue;
        }
        if c.key != view_id {
            continue;
        }
        if let Ok(Contribution::View(spec)) =
            serde_json::from_value::<Contribution>(c.payload.clone())
        {
            return Ok(Some(PluginViewResolution {
                plugin_id: plugin_id.clone(),
                view_id: view_id.clone(),
                title: spec.title,
                entry_url: format!("plugin://{}/{}", plugin_id, spec.entry),
            }));
        }
    }
    Ok(None)
}

#[command]
pub async fn get_plugin_settings(
    plugin_id: String,
    runtime: State<'_, Arc<PluginRuntime>>,
    settings: State<'_, Arc<SettingsRegistry>>,
) -> Result<Vec<PluginSettingMeta>, AppError> {
    let manifest = runtime
        .inner()
        .plugin_manifests()
        .into_iter()
        .find(|m| m.id.to_string() == plugin_id)
        .ok_or_else(|| AppError::Plugin(format!("plugin '{}' not found", plugin_id)))?;
    let stored = settings.get_all(&plugin_id);
    Ok(manifest
        .settings
        .into_iter()
        .map(|spec| {
            let default_value = json_to_setting(spec.default_value);
            let value = stored
                .get(&spec.key)
                .cloned()
                .unwrap_or_else(|| default_value.clone());
            PluginSettingMeta {
                key: spec.key,
                title: spec.title,
                value,
                default_value,
            }
        })
        .collect())
}

#[command]
pub async fn update_plugin_setting(
    plugin_id: String,
    key: String,
    value: SettingValue,
    settings: State<'_, Arc<SettingsRegistry>>,
    storage: State<'_, Arc<JsonPluginStorage>>,
) -> Result<(), AppError> {
    settings.update_setting(&plugin_id, &key, value)?;
    let all = settings.dump_all();
    let states: HashMap<String, PluginLifecycleState> = HashMap::new();
    if let Err(e) = storage.save_config(&states, &all) {
        eprintln!("[plugin] failed to persist plugin settings: {}", e);
    }
    Ok(())
}

#[command]
pub async fn enable_plugin_command(
    plugin_id: String,
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<(), AppError> {
    let id = PluginId::new(&plugin_id)
        .map_err(|e| AppError::Plugin(e.to_string()))?;
    runtime
        .inner()
        .activate(&id)
        .map_err(|e| AppError::Plugin(e.to_string()))
}

#[command]
pub async fn disable_plugin_command(
    plugin_id: String,
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<(), AppError> {
    let id = PluginId::new(&plugin_id)
        .map_err(|e| AppError::Plugin(e.to_string()))?;
    runtime
        .inner()
        .deactivate(&id, DeactivateReason::UserDisabled)
        .map_err(|e| AppError::Plugin(e.to_string()))
}

#[command]
pub async fn execute_plugin_command(
    command_id: String,
    args: CommandArgs,
    runtime: State<'_, Arc<PluginRuntime>>,
) -> Result<(), AppError> {
    let cid = CommandId::new(&command_id)
        .map_err(|e| AppError::Plugin(e.to_string()))?;
    let value = command_args_to_value(&args);
    runtime
        .inner()
        .execute(&cid, value)
        .map_err(|e| AppError::Plugin(e.to_string()))?;
    Ok(())
}
