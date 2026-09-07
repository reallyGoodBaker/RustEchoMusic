use serde::{Deserialize, Serialize};

use crate::services::plugin::dto::MenuLocation;
use crate::services::plugin::extension::state::ExtensionState;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuExtension {
    pub id: String,
    pub plugin_id: String,
    pub command: String,
    pub location: MenuLocation,
    pub group: Option<String>,
    pub state: ExtensionState,
}
