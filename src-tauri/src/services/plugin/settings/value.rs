use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SettingValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Text(String),
    List(Vec<String>),
    Json(serde_json::Value),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginSetting {
    pub key: String,
    pub title: String,
    pub value: SettingValue,
    pub default_value: SettingValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSettingMeta {
    pub key: String,
    pub title: String,
    pub value: SettingValue,
    pub default_value: SettingValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginSettings {
    pub plugin_id: String,
    pub settings: HashMap<String, SettingValue>,
}
