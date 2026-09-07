use std::collections::VecDeque;

use plugin_sdk::{PluginError, PluginId, PluginResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PluginState {
    Discovered,
    Resolved,
    Loaded,
    Activating,
    Active,
    Deactivating,
    Stopped,
    Failed,
    Unloaded,
}

impl PluginState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Resolved => "resolved",
            Self::Loaded => "loaded",
            Self::Activating => "activating",
            Self::Active => "active",
            Self::Deactivating => "deactivating",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
            Self::Unloaded => "unloaded",
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Unloaded)
    }

    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    pub const fn can_activate(self) -> bool {
        matches!(self, Self::Loaded | Self::Stopped | Self::Failed)
    }

    pub const fn can_deactivate(self) -> bool {
        matches!(self, Self::Active)
    }

    pub const fn can_unload(self) -> bool {
        matches!(self, Self::Loaded | Self::Stopped | Self::Failed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    pub from: PluginState,
    pub to: PluginState,
    pub at_ms: u64,
    pub reason: String,
}

pub fn can_transition(from: PluginState, to: PluginState) -> bool {
    use PluginState as S;
    matches!(
        (from, to),
        (S::Discovered, S::Resolved | S::Unloaded)
            | (S::Resolved, S::Loaded | S::Failed | S::Unloaded)
            | (S::Loaded, S::Activating | S::Stopped | S::Unloaded)
            | (S::Activating, S::Active | S::Failed)
            | (S::Active, S::Deactivating | S::Failed)
            | (S::Deactivating, S::Stopped | S::Failed)
            | (S::Stopped, S::Activating | S::Loaded | S::Unloaded)
            | (S::Failed, S::Activating | S::Loaded | S::Unloaded)
            | (S::Unloaded, S::Resolved)
    )
}

#[derive(Debug, Clone)]
pub struct LifecycleTracker {
    state: PluginState,
    history: VecDeque<Transition>,
    last_error: Option<String>,
}

const HISTORY_LIMIT: usize = 32;

impl LifecycleTracker {
    pub fn new(state: PluginState) -> Self {
        let mut history = VecDeque::with_capacity(HISTORY_LIMIT);
        history.push_back(Transition {
            from: state,
            to: state,
            at_ms: now_ms(),
            reason: "initialized".into(),
        });
        Self {
            state,
            history,
            last_error: None,
        }
    }

    pub fn state(&self) -> PluginState {
        self.state
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn history(&self) -> &VecDeque<Transition> {
        &self.history
    }

    pub fn transition(&mut self, to: PluginState, reason: impl Into<String>) -> PluginResult<PluginState> {
        if !can_transition(self.state, to) {
            return Err(PluginError::invalid_argument(format!(
                "illegal plugin lifecycle transition: {} -> {}",
                self.state.label(),
                to.label()
            )));
        }
        let record = Transition {
            from: self.state,
            to,
            at_ms: now_ms(),
            reason: reason.into(),
        };
        self.state = to;
        if self.history.len() >= HISTORY_LIMIT {
            self.history.pop_front();
        }
        self.history.push_back(record);
        Ok(to)
    }

    pub fn mark_failed(&mut self, error: &PluginError) {
        self.last_error = Some(error.to_string());
        let _ = self.transition(PluginState::Failed, "failed");
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    pub fn record_error(&mut self, message: impl Into<String>) {
        self.last_error = Some(message.into());
    }
}

pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleEvent {
    pub plugin: PluginId,
    pub from: PluginState,
    pub to: PluginState,
    pub reason: String,
    pub at_ms: u64,
}