#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    Plugin(String),
    PermissionDenied {
        plugin_id: String,
        permission: String,
    },
    Io(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Plugin(msg) => write!(f, "Plugin error: {}", msg),
            PluginError::PermissionDenied {
                plugin_id,
                permission,
            } => write!(
                f,
                "Plugin '{}' denied permission {}",
                plugin_id, permission
            ),
            PluginError::Io(msg) => write!(f, "Plugin IO error: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(e: std::io::Error) -> Self {
        PluginError::Io(e.to_string())
    }
}

impl From<String> for PluginError {
    fn from(s: String) -> Self {
        PluginError::Plugin(s)
    }
}

impl From<&str> for PluginError {
    fn from(s: &str) -> Self {
        PluginError::Plugin(s.to_string())
    }
}
