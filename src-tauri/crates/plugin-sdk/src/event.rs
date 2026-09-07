use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::PluginError;
use super::ids::{EventType, PluginId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostEvent {
    pub kind: EventType,
    pub payload: Value,
    pub source: Option<PluginId>,
    pub sequence: u64,
}

impl HostEvent {
    pub fn host(kind: EventType, payload: Value, sequence: u64) -> Self {
        Self {
            kind,
            payload,
            source: None,
            sequence,
        }
    }

    pub fn from_plugin(
        kind: EventType,
        payload: Value,
        sequence: u64,
        source: PluginId,
    ) -> Self {
        Self {
            kind,
            payload,
            source: Some(source),
            sequence,
        }
    }

    pub fn decode<T: for<'de> Deserialize<'de>>(&self) -> Result<T, PluginError> {
        serde_json::from_value(self.payload.clone()).map_err(|e| {
            PluginError::invalid_argument(format!(
                "event '{}' payload does not match expected shape: {}",
                self.kind, e
            ))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventPattern {
    All,
    Kinds(Vec<EventType>),
    Prefix(String),
}

impl EventPattern {
    pub fn kind(kind: EventType) -> Self {
        Self::Kinds(vec![kind])
    }

    pub fn prefix(prefix: impl Into<String>) -> Self {
        Self::Prefix(prefix.into())
    }

    pub fn matches(&self, event: &HostEvent) -> bool {
        match self {
            Self::All => true,
            Self::Kinds(kinds) => kinds.contains(&event.kind),
            Self::Prefix(prefix) => event.kind.as_str().starts_with(prefix.as_str()),
        }
    }

    pub fn may_match(&self, kind: &EventType) -> bool {
        match self {
            Self::All => true,
            Self::Kinds(kinds) => kinds.contains(kind),
            Self::Prefix(prefix) => kind.as_str().starts_with(prefix.as_str()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventSubscriptions(Vec<EventPattern>);

impl EventSubscriptions {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(mut self, pattern: EventPattern) -> Self {
        self.0.push(pattern);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn matches(&self, event: &HostEvent) -> bool {
        self.0.iter().any(|p| p.matches(event))
    }

    pub fn may_match(&self, kind: &EventType) -> bool {
        self.0.iter().any(|p| p.may_match(kind))
    }
}

pub mod kinds {
    use super::EventType;

    pub fn track_changed() -> EventType {
        EventType::new("track.changed").expect("static event type is valid")
    }
    pub fn playback_state() -> EventType {
        EventType::new("playback.state").expect("static event type is valid")
    }
    pub fn queue_changed() -> EventType {
        EventType::new("queue.changed").expect("static event type is valid")
    }
    pub fn settings_changed() -> EventType {
        EventType::new("settings.changed").expect("static event type is valid")
    }
    pub fn library_changed() -> EventType {
        EventType::new("library.changed").expect("static event type is valid")
    }
    pub fn plugin_lifecycle() -> EventType {
        EventType::new("plugin.lifecycle").expect("static event type is valid")
    }
    pub fn audio_format() -> EventType {
        EventType::new("audio.format").expect("static event type is valid")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchStats {
    pub delivered: BTreeMap<String, u64>,
    pub failed: BTreeMap<String, u64>,
    pub skipped: u64,
}