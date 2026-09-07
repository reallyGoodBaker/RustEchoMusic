use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginLifecycleState {
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginState {
    pub plugin_id: String,
    pub state: PluginLifecycleState,
}
