use serde::{Deserialize, Serialize};

use crate::services::plugin::extension::state::ExtensionState;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarExtension {
    pub id: String,
    pub plugin_id: String,
    pub title: String,
    pub icon: String,
    pub route: String,
    pub state: ExtensionState,
}
