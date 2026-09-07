use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::errors::AppError;
use crate::services::plugin::runtime::state::PluginLifecycleState;
use crate::services::plugin::settings::value::PluginSettings;

pub trait PluginStorage: Send + Sync {
    fn load_config(
        &self,
    ) -> Result<
        (
            HashMap<String, PluginLifecycleState>,
            HashMap<String, PluginSettings>,
        ),
        AppError,
    >;
    fn save_config(
        &self,
        states: &HashMap<String, PluginLifecycleState>,
        settings: &HashMap<String, PluginSettings>,
    ) -> Result<(), AppError>;
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct PluginConfigFile {
    states: HashMap<String, PluginLifecycleState>,
    settings: HashMap<String, PluginSettings>,
}

pub struct JsonPluginStorage {
    path: PathBuf,
}

impl JsonPluginStorage {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let dir = app_data_dir.join("plugins");
        Self {
            path: dir.join("plugins.config.json"),
        }
    }
}

impl PluginStorage for JsonPluginStorage {
    fn load_config(
        &self,
    ) -> Result<
        (
            HashMap<String, PluginLifecycleState>,
            HashMap<String, PluginSettings>,
        ),
        AppError,
    > {
        if !self.path.exists() {
            return Ok((HashMap::new(), HashMap::new()));
        }

        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| AppError::Io(format!("Failed to read plugin config: {}", e)))?;

        let file: PluginConfigFile = serde_json::from_str(&content)
            .map_err(|e| AppError::Plugin(format!("Failed to parse plugin config: {}", e)))?;

        Ok((file.states, file.settings))
    }

    fn save_config(
        &self,
        states: &HashMap<String, PluginLifecycleState>,
        settings: &HashMap<String, PluginSettings>,
    ) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Io(format!("Failed to create plugin config directory: {}", e))
            })?;
        }

        let file = PluginConfigFile {
            states: states.clone(),
            settings: settings.clone(),
        };

        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| AppError::Plugin(format!("Failed to serialize plugin config: {}", e)))?;

        std::fs::write(&self.path, content)
            .map_err(|e| AppError::Io(format!("Failed to write plugin config: {}", e)))?;

        Ok(())
    }
}
