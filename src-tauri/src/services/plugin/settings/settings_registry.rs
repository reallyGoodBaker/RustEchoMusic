use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::errors::AppError;
use crate::services::plugin::settings::value::{PluginSettings, SettingValue};

pub struct SettingsRegistry {
    settings: RwLock<HashMap<String, PluginSettings>>,
}

impl SettingsRegistry {
    pub fn new() -> Self {
        Self {
            settings: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_defaults(&self, plugin_id: &str, defaults: HashMap<String, SettingValue>) {
        let mut lock = match self.settings.write() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        let entry = lock
            .entry(plugin_id.to_string())
            .or_insert_with(|| PluginSettings {
                plugin_id: plugin_id.to_string(),
                settings: HashMap::new(),
            });

        for (key, value) in defaults {
            entry.settings.entry(key).or_insert(value);
        }
    }

    pub fn get_setting(&self, plugin_id: &str, key: &str) -> Option<SettingValue> {
        let lock = self.settings.read().ok()?;
        lock.get(plugin_id)?.settings.get(key).cloned()
    }

    pub fn get_all(&self, plugin_id: &str) -> HashMap<String, SettingValue> {
        let lock = match self.settings.read() {
            Ok(guard) => guard,
            Err(_) => return HashMap::new(),
        };
        lock.get(plugin_id)
            .map(|ps| ps.settings.clone())
            .unwrap_or_default()
    }

    pub fn update_setting(
        &self,
        plugin_id: &str,
        key: &str,
        value: SettingValue,
    ) -> Result<(), AppError> {
        let mut lock = self
            .settings
            .write()
            .map_err(|e| AppError::Plugin(e.to_string()))?;

        let entry = lock
            .entry(plugin_id.to_string())
            .or_insert_with(|| PluginSettings {
                plugin_id: plugin_id.to_string(),
                settings: HashMap::new(),
            });

        entry.settings.insert(key.to_string(), value);
        Ok(())
    }

    pub fn dump_all(&self) -> HashMap<String, PluginSettings> {
        let lock = match self.settings.read() {
            Ok(guard) => guard,
            Err(_) => return HashMap::new(),
        };
        lock.clone()
    }

    pub fn load_all(&self, data: HashMap<String, PluginSettings>) {
        let mut lock = match self.settings.write() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        *lock = data;
    }
}
