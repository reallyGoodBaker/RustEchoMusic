use serde::{Deserialize, Serialize};

use crate::services::plugin::settings::value::SettingValue;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandContribution {
    pub id: String,
    pub title: String,
    pub category: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuLocation {
    TrackContextMenu,
    SidebarActions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MenuContribution {
    pub command: String,
    pub title: String,
    pub location: MenuLocation,
    pub group: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidebarContribution {
    pub id: String,
    pub title: String,
    pub icon: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewContribution {
    pub id: String,
    pub title: String,
    pub entry: String,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeViewContribution {
    pub id: String,
    pub title: String,
    pub token: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ActivationEvent {
    OnStartup,
    OnTrackChanged,
    OnPlaybackStateChanged,
    OnQueueChanged,
    OnSettingsChanged,
    OnCommand(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginSource {
    Builtin,
    Packaged,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginContribution {
    pub commands: Vec<CommandContribution>,
    pub menus: Vec<MenuContribution>,
    pub sidebars: Vec<SidebarContribution>,
    #[serde(default)]
    pub views: Vec<ViewContribution>,
    #[serde(default, rename = "nativeViews")]
    pub native_views: Vec<NativeViewContribution>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingDefinition {
    pub key: String,
    pub title: String,
    pub default_value: SettingValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPluginManifest {
    pub id: String,
    pub source: PluginSource,
    pub route: String,
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry: String,
    pub min_app_version: String,
    pub permissions: Vec<String>,
    pub activation_events: Vec<ActivationEvent>,
    pub contributes: PluginContribution,
    pub settings: Vec<SettingDefinition>,
}
