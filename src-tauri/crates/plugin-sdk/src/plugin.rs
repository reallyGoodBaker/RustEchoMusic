use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::{PluginError, PluginResult};
use super::event::{EventPattern, HostEvent};
use super::ids::{Capability, CommandId, ContributionPointId, PluginId, ServiceId, Version};
use super::service::{
    AnyService, ServiceBinding, ServiceDescriptor, ServiceRef, ServiceSlot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginDescriptor {
    pub id: PluginId,
    pub version: Version,
    pub min_host: Version,
    pub abi: u32,
    pub display_name: String,
    pub summary: String,
    pub capabilities: Vec<Capability>,
    pub depends_on: Vec<PluginId>,
    pub optional_depends_on: Vec<PluginId>,
}

impl PluginDescriptor {
    pub fn new(id: PluginId, version: Version, min_host: Version) -> Self {
        Self {
            id,
            version,
            min_host,
            abi: super::ABI_VERSION,
            display_name: String::new(),
            summary: String::new(),
            capabilities: Vec::new(),
            depends_on: Vec::new(),
            optional_depends_on: Vec::new(),
        }
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    pub fn requiring(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn depends_on(mut self, plugin: PluginId) -> Self {
        self.depends_on.push(plugin);
        self
    }

    pub fn optionally_depends_on(mut self, plugin: PluginId) -> Self {
        self.optional_depends_on.push(plugin);
        self
    }
}

// —— 声明式贡献 ——

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSpec {
    pub id: CommandId,
    pub title: String,
    pub category: Option<String>,
    pub input_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSpec {
    pub command: CommandId,
    pub title: String,
    pub location: String,
    pub group: Option<String>,
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarSpec {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpec {
    pub id: String,
    pub title: String,
    pub entry: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeViewSpec {
    pub id: String,
    pub title: String,
    pub token: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioProcessorSpec {
    pub id: String,
    pub order: i32,
    pub budget_us: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingSpec {
    pub key: String,
    pub title: String,
    pub default_value: Value,
    pub control: String,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Contribution {
    Command(CommandSpec),
    MenuItem(MenuSpec),
    SidebarItem(SidebarSpec),
    View(ViewSpec),
    NativeView(NativeViewSpec),
    AudioProcessor(AudioProcessorSpec),
    Setting(SettingSpec),
    Extension {
        point: ContributionPointId,
        payload: Value,
    },
}

impl Contribution {
    // —— 类型化访问器，供宿主的贡献点消费者使用 ——
    pub fn as_command(&self) -> Option<&CommandSpec> {
        match self {
            Self::Command(spec) => Some(spec),
            _ => None,
        }
    }
    pub fn as_menu_item(&self) -> Option<&MenuSpec> {
        match self {
            Self::MenuItem(spec) => Some(spec),
            _ => None,
        }
    }
    pub fn as_sidebar_item(&self) -> Option<&SidebarSpec> {
        match self {
            Self::SidebarItem(spec) => Some(spec),
            _ => None,
        }
    }
    pub fn as_view(&self) -> Option<&ViewSpec> {
        match self {
            Self::View(spec) => Some(spec),
            _ => None,
        }
    }
    pub fn as_native_view(&self) -> Option<&NativeViewSpec> {
        match self {
            Self::NativeView(spec) => Some(spec),
            _ => None,
        }
    }
    pub fn as_audio_processor(&self) -> Option<&AudioProcessorSpec> {
        match self {
            Self::AudioProcessor(spec) => Some(spec),
            _ => None,
        }
    }
    pub fn as_setting(&self) -> Option<&SettingSpec> {
        match self {
            Self::Setting(spec) => Some(spec),
            _ => None,
        }
    }

    pub fn point(&self) -> ContributionPointId {
        // 常量在构造时已校验，unwrap 安全。
        let id = match self {
            Self::Command(_) => "ui.command",
            Self::MenuItem(_) => "ui.menuItem",
            Self::SidebarItem(_) => "ui.sidebar",
            Self::View(_) => "ui.view",
            Self::NativeView(_) => "ui.nativeView",
            Self::AudioProcessor(_) => "audio.processor",
            Self::Setting(_) => "config.setting",
            Self::Extension { point, .. } => return point.clone(),
        };
        ContributionPointId::new(id).expect("built-in contribution point id is valid")
    }

    pub fn key(&self) -> String {
        match self {
            Self::Command(spec) => spec.id.to_string(),
            Self::MenuItem(spec) => format!("{}@{}", spec.command, spec.location),
            Self::SidebarItem(spec) => spec.id.clone(),
            Self::View(spec) => spec.id.clone(),
            Self::NativeView(spec) => spec.id.clone(),
            Self::AudioProcessor(spec) => spec.id.clone(),
            Self::Setting(spec) => spec.key.clone(),
            Self::Extension { point, payload } => {
                format!("{}#{}", point, stable_key_of(payload))
            }
        }
    }
}

fn stable_key_of(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            keys.into_iter()
                .map(|k| format!("{}={}", k, stable_key_of(&map[k])))
                .collect::<Vec<_>>()
                .join("&")
        }
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

#[derive(Debug, Default)]
pub struct ActivationScope {
    contributions: Vec<Contribution>,
    services: Vec<ServiceBinding>,
    subscriptions: Vec<EventPattern>,
}

impl ActivationScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contribute(&mut self, contribution: Contribution) -> &mut Self {
        self.contributions.push(contribution);
        self
    }

    pub fn provide(&mut self, binding: ServiceBinding) -> &mut Self {
        self.services.push(binding);
        self
    }

    pub fn subscribe(&mut self, pattern: EventPattern) -> &mut Self {
        self.subscriptions.push(pattern);
        self
    }

    pub fn contributions(&self) -> &[Contribution] {
        &self.contributions
    }

    pub fn subscriptions(&self) -> &[EventPattern] {
        &self.subscriptions
    }

    pub fn is_empty(&self) -> bool {
        self.contributions.is_empty() && self.services.is_empty() && self.subscriptions.is_empty()
    }

    pub fn take_services(&mut self) -> Vec<ServiceBinding> {
        std::mem::take(&mut self.services)
    }
}

pub trait ServiceResolver: Send + Sync {
    fn resolve(
        &self,
        requester: &PluginId,
        id: &ServiceId,
        type_id: std::any::TypeId,
    ) -> PluginResult<ResolvedService>;
}

pub struct ResolvedService {
    pub slot: std::sync::Arc<ServiceSlot>,
    pub raw: std::sync::Arc<AnyService>,
}

impl ResolvedService {
    pub fn new(slot: std::sync::Arc<ServiceSlot>, raw: std::sync::Arc<AnyService>) -> Self {
        Self { slot, raw }
    }
}

pub struct ActivationContext<'a> {
    plugin: &'a PluginId,
    scope: &'a mut ActivationScope,
    resolver: &'a dyn ServiceResolver,
    granted: &'a [Capability],
}

impl<'a> ActivationContext<'a> {
    pub fn new(
        plugin: &'a PluginId,
        scope: &'a mut ActivationScope,
        resolver: &'a dyn ServiceResolver,
        granted: &'a [Capability],
    ) -> Self {
        Self {
            plugin,
            scope,
            resolver,
            granted,
        }
    }

    pub fn plugin_id(&self) -> &PluginId {
        self.plugin
    }

    pub fn require<T: Send + Sync + 'static>(&self, id: &ServiceId) -> PluginResult<ServiceRef<T>> {
        let resolved = self
            .resolver
            .resolve(self.plugin, id, std::any::TypeId::of::<T>())?;
        let value = ServiceBinding::downcast::<T>(&resolved.raw).ok_or_else(|| {
            PluginError::service_unavailable(format!(
                "service '{id}' exists but is not of the requested type"
            ))
        })?;
        Ok(ServiceRef::new(resolved.slot, value))
    }

    pub fn try_require<T: Send + Sync + 'static>(
        &self,
        id: &ServiceId,
    ) -> PluginResult<Option<ServiceRef<T>>> {
        match self.require::<T>(id) {
            Ok(reference) => Ok(Some(reference)),
            Err(error) if error.code().is_retriable() => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn has(&self, capability: &Capability) -> bool {
        self.granted.contains(capability)
    }

    pub fn contribute(&mut self, contribution: Contribution) -> &mut Self {
        self.scope.contribute(contribution);
        self
    }

    pub fn provide(&mut self, binding: ServiceBinding) -> &mut Self {
        self.scope.provide(binding);
        self
    }

    pub fn subscribe(&mut self, pattern: EventPattern) -> &mut Self {
        self.scope.subscribe(pattern);
        self
    }
}

impl fmt::Debug for ActivationContext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActivationContext")
            .field("plugin", self.plugin)
            .finish()
    }
}

pub struct InvokeContext<'a> {
    plugin: &'a PluginId,
    resolver: &'a dyn ServiceResolver,
}

impl<'a> InvokeContext<'a> {
    pub fn new(plugin: &'a PluginId, resolver: &'a dyn ServiceResolver) -> Self {
        Self { plugin, resolver }
    }

    pub fn plugin_id(&self) -> &PluginId {
        self.plugin
    }

    pub fn require<T: Send + Sync + 'static>(&self, id: &ServiceId) -> PluginResult<ServiceRef<T>> {
        let resolved = self
            .resolver
            .resolve(self.plugin, id, std::any::TypeId::of::<T>())?;
        let value = ServiceBinding::downcast::<T>(&resolved.raw).ok_or_else(|| {
            PluginError::service_unavailable(format!(
                "service '{id}' exists but is not of the requested type"
            ))
        })?;
        Ok(ServiceRef::new(resolved.slot, value))
    }
}

pub struct AudioProcessorHandle {
    pub abi: super::abi::AudioProcessorAbi,
    pub handle: *mut std::ffi::c_void,
}

// SAFETY: 见类型文档；宿主保证只在插件激活期间使用。
unsafe impl Send for AudioProcessorHandle {}
unsafe impl Sync for AudioProcessorHandle {}

pub struct TeardownContext {
    plugin: PluginId,
    reason: DeactivateReason,
}

impl TeardownContext {
    pub fn new(plugin: PluginId, reason: DeactivateReason) -> Self {
        Self { plugin, reason }
    }
    pub fn plugin_id(&self) -> &PluginId {
        &self.plugin
    }
    pub fn reason(&self) -> DeactivateReason {
        self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeactivateReason {
    UserDisabled,
    Uninstall,
    Reload,
    CircuitBroken,
    DependencyRemoved,
    Shutdown,
}

impl DeactivateReason {
    pub const fn preserves_state(self) -> bool {
        !matches!(self, DeactivateReason::Uninstall)
    }
}

pub trait Plugin: Send + Sync {
    fn descriptor(&self) -> &PluginDescriptor;

    fn activate(&self, ctx: &mut ActivationContext) -> PluginResult<()>;

    fn deactivate(&self, ctx: &TeardownContext) -> PluginResult<()>;

    fn on_event(&self, _event: &HostEvent, _ctx: &InvokeContext) -> PluginResult<()> {
        Ok(())
    }

    fn execute(
        &self,
        command: &CommandId,
        _args: Value,
        _ctx: &InvokeContext,
    ) -> PluginResult<Value> {
        Err(PluginError::invalid_argument(format!(
            "plugin '{}' does not handle command '{command}'",
            self.descriptor().id
        )))
    }

    fn health(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    fn audio_processor(&self) -> Option<AudioProcessorHandle> {
        None
    }
}

pub trait PluginFactory: Send + Sync {
    fn descriptor(&self) -> &PluginDescriptor;
    fn create(&self) -> PluginResult<Box<dyn Plugin>>;
}

pub fn summarize_services(bindings: &[ServiceBinding]) -> Vec<ServiceDescriptor> {
    bindings.iter().map(|b| b.descriptor().clone()).collect()
}