use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    Plugin,
    PermissionDenied,
    NotFound,
    ServiceUnavailable,
    DependencyMissing,
    InvalidArgument,
    Timeout,
    Panic,
    Incompatible,
    Io,
}

impl ErrorCode {
    pub const fn is_retriable(self) -> bool {
        matches!(
            self,
            ErrorCode::ServiceUnavailable
                | ErrorCode::DependencyMissing
                | ErrorCode::Timeout
                | ErrorCode::Panic
                | ErrorCode::Io
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginError {
    code: ErrorCode,
    message: String,
    plugin: Option<String>,
}

impl PluginError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            plugin: None,
        }
    }

    pub fn with_plugin(mut self, plugin: impl Into<String>) -> Self {
        self.plugin = Some(plugin.into());
        self
    }

    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn plugin_id(&self) -> Option<&str> {
        self.plugin.as_deref()
    }

    pub const fn is_retriable(&self) -> bool {
        self.code.is_retriable()
    }

    // —— 常用构造器 ——
    pub fn plugin(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Plugin, message)
    }
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PermissionDenied, message)
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ServiceUnavailable, message)
    }
    pub fn dependency_missing(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::DependencyMissing, message)
    }
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidArgument, message)
    }
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Timeout, message)
    }
    pub fn incompatible(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Incompatible, message)
    }
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Io, message)
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.plugin {
            Some(plugin) => write!(f, "[{:?}] {} (plugin={})", self.code, self.message, plugin),
            None => write!(f, "[{:?}] {}", self.code, self.message),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<std::io::Error> for PluginError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(ErrorCode::Io, error.to_string())
    }
}

pub type PluginResult<T> = Result<T, PluginError>;
