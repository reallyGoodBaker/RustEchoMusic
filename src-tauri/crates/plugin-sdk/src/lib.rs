pub mod abi;
pub mod command;
pub mod error;
pub mod event;
pub mod guest;
pub mod host;
pub mod ids;
pub mod manifest;
pub mod plugin;
pub mod service;

pub use command::CommandArgs;
pub use error::{ErrorCode, PluginError, PluginResult};
pub use event::{DispatchStats, EventPattern, EventSubscriptions, HostEvent};
pub use ids::{Capability, CommandId, ContributionPointId, EventType, PluginId, ServiceId, Version};
pub use manifest::{
    validate_manifest, ActivationSpec, DependencySpec, DiscoveredPlugin, IntegritySpec,
    Manifest, PluginSource, PluginTier, ResourceBudget,
};
pub use host::{
    EqualizerApi, HostEventsApi, LibraryApi, PlayerControlApi, PlayerStateApi, PluginSettingsApi,
    PluginStorageApi, QueueApi, RecentReadApi, RecentWriteApi, SettingsApi,
};
pub use plugin::{
    ActivationContext, ActivationScope, AudioProcessorHandle, AudioProcessorSpec, CommandSpec,
    Contribution, DeactivateReason, HealthStatus, InvokeContext, MenuSpec, NativeViewSpec,
    PluginDescriptor, PluginFactory, Plugin, ResolvedService, ServiceResolver, SettingSpec,
    SidebarSpec, TeardownContext, ViewSpec,
};
pub use service::{
    AnyService, ServiceBinding, ServiceDescriptor, ServiceHandle, ServiceRef, ServiceSlot,
    ServiceVisibility,
};

pub const ABI_VERSION: u32 = 2;

pub const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod capabilities {
    use super::Capability;

    pub fn player_read() -> Capability {
        Capability::new("player.read").expect("static capability is valid")
    }
    pub fn player_control() -> Capability {
        Capability::new("player.control").expect("static capability is valid")
    }
    pub fn queue_read() -> Capability {
        Capability::new("queue.read").expect("static capability is valid")
    }
    pub fn queue_write() -> Capability {
        Capability::new("queue.write").expect("static capability is valid")
    }
    pub fn library_read() -> Capability {
        Capability::new("library.read").expect("static capability is valid")
    }
    pub fn library_write() -> Capability {
        Capability::new("library.write").expect("static capability is valid")
    }
    pub fn settings_read() -> Capability {
        Capability::new("settings.read").expect("static capability is valid")
    }
    pub fn settings_write() -> Capability {
        Capability::new("settings.write").expect("static capability is valid")
    }
    pub fn audio_process() -> Capability {
        Capability::new("audio.process").expect("static capability is valid")
    }
    pub fn plugin_ui() -> Capability {
        Capability::new("plugin.ui").expect("static capability is valid")
    }
    pub fn storage_private() -> Capability {
        Capability::new("storage.private").expect("static capability is valid")
    }
    pub fn fs_user_granted() -> Capability {
        Capability::new("fs.userGranted").expect("static capability is valid")
    }
    pub fn network() -> Capability {
        Capability::new("net.fetch").expect("static capability is valid")
    }
    pub fn recent_read() -> Capability {
        Capability::new("recent.read").expect("static capability is valid")
    }
    pub fn recent_write() -> Capability {
        Capability::new("recent.write").expect("static capability is valid")
    }
}

pub mod services {
    use super::ServiceId;

    pub fn player() -> ServiceId {
        ServiceId::new("player.control").expect("static service id is valid")
    }
    pub fn player_state() -> ServiceId {
        ServiceId::new("player.state").expect("static service id is valid")
    }
    pub fn queue() -> ServiceId {
        ServiceId::new("queue.control").expect("static service id is valid")
    }
    pub fn library() -> ServiceId {
        ServiceId::new("library.read").expect("static service id is valid")
    }
    pub fn settings() -> ServiceId {
        ServiceId::new("settings.app").expect("static service id is valid")
    }
    pub fn equalizer() -> ServiceId {
        ServiceId::new("audio.equalizer").expect("static service id is valid")
    }
    pub fn audio_graph() -> ServiceId {
        ServiceId::new("audio.graph").expect("static service id is valid")
    }
    pub fn metadata() -> ServiceId {
        ServiceId::new("metadata.reader").expect("static service id is valid")
    }
    pub fn recent_read() -> ServiceId {
        ServiceId::new("recent.read").expect("static service id is valid")
    }
    pub fn recent_write() -> ServiceId {
        ServiceId::new("recent.write").expect("static service id is valid")
    }
    pub fn storage() -> ServiceId {
        ServiceId::new("storage.plugin").expect("static service id is valid")
    }
    pub fn plugin_settings() -> ServiceId {
        ServiceId::new("settings.plugin").expect("static service id is valid")
    }
    pub fn events() -> ServiceId {
        ServiceId::new("host.events").expect("static service id is valid")
    }
}
