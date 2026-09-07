use serde::{Deserialize, Serialize};

use crate::services::plugin::extension::state::ExtensionState;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeViewExtension {
    pub id: String,
    pub plugin_id: String,
    pub title: String,
    pub token: String,
    pub icon: Option<String>,
    pub state: ExtensionState,
}
