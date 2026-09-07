use serde::{Deserialize, Serialize};

use super::error::PluginError;
use super::ids::{Capability, CommandId, EventType, PluginId, Version};
use super::plugin::SettingSpec;

pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginSource {
    Builtin,
    Packaged,
    User,
}

impl PluginSource {
    pub const fn is_removable(self) -> bool {
        !matches!(self, PluginSource::Builtin)
    }
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::Packaged => "packaged",
            Self::User => "user",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginTier {
    #[default]
    Feature,
    Core,
}

impl PluginTier {
    pub const fn is_core(self) -> bool {
        matches!(self, PluginTier::Core)
    }
    pub const fn is_user_disableable(self) -> bool {
        !self.is_core()
    }
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Feature => "feature",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencySpec {
    pub plugin: PluginId,
    pub min_version: Option<Version>,
    pub optional: bool,
}

impl DependencySpec {
    pub fn required(plugin: PluginId) -> Self {
        Self {
            plugin,
            min_version: None,
            optional: false,
        }
    }
    pub fn optional(plugin: PluginId) -> Self {
        Self {
            plugin,
            min_version: None,
            optional: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationSpec {
    pub eager: bool,
    pub events: Vec<EventType>,
    pub commands: Vec<CommandId>,
}

impl ActivationSpec {
    pub fn eager() -> Self {
        Self {
            eager: true,
            events: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub fn lazy() -> Self {
        Self::default()
    }

    pub fn on_event(mut self, kind: EventType) -> Self {
        self.events.push(kind);
        self
    }

    pub fn on_command(mut self, command: CommandId) -> Self {
        self.commands.push(command);
        self
    }

    pub fn triggered_by_event(&self, kind: &EventType) -> bool {
        self.eager || self.events.contains(kind)
    }

    pub fn triggered_by_command(&self, command: &CommandId) -> bool {
        self.eager || self.commands.contains(command)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegritySpec {
    pub algorithm: String,
    pub digest: String,
}

impl IntegritySpec {
    pub fn sha256(digest: impl Into<String>) -> Self {
        Self {
            algorithm: "sha256".into(),
            digest: digest.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceBudget {
    pub call_timeout_ms: u64,
    pub max_consecutive_failures: u32,
    pub cooldown_ms: u64,
    pub audio_budget_us: Option<u32>,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            call_timeout_ms: 2_000,
            max_consecutive_failures: 3,
            cooldown_ms: 30_000,
            audio_budget_us: None,
        }
    }
}

impl ResourceBudget {
    pub fn realtime() -> Self {
        Self {
            call_timeout_ms: 200,
            max_consecutive_failures: 5,
            cooldown_ms: 5_000,
            audio_budget_us: Some(500),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema: u32,
    pub id: PluginId,
    pub name: String,
    pub display_name: String,
    pub version: Version,
    pub min_host: Version,
    pub abi: u32,
    pub author: String,
    pub description: String,
    pub entry: String,
    pub source: PluginSource,
    #[serde(default)]
    pub tier: PluginTier,
    pub capabilities: Vec<Capability>,
    #[serde(default)]
    pub activation: ActivationSpec,
    #[serde(default)]
    pub dependencies: Vec<DependencySpec>,
    #[serde(default)]
    pub settings: Vec<SettingSpec>,
    pub integrity: Option<IntegritySpec>,
    #[serde(default)]
    pub budget: ResourceBudget,
}

impl Manifest {
    pub fn builtin(id: PluginId, version: Version, min_host: Version) -> Self {
        Self {
            schema: SCHEMA_VERSION,
            id,
            name: String::new(),
            display_name: String::new(),
            version,
            min_host,
            abi: super::ABI_VERSION,
            author: String::new(),
            description: String::new(),
            entry: String::new(),
            source: PluginSource::Builtin,
            tier: PluginTier::Feature,
            capabilities: Vec::new(),
            activation: ActivationSpec::eager(),
            dependencies: Vec::new(),
            settings: Vec::new(),
            integrity: None,
            budget: ResourceBudget::default(),
        }
    }

    pub fn core(id: PluginId, version: Version, min_host: Version) -> Self {
        let mut manifest = Self::builtin(id, version, min_host);
        manifest.tier = PluginTier::Core;
        manifest
    }

    pub fn is_user_disableable(&self) -> bool {
        self.tier.is_user_disableable()
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredPlugin {
    pub manifest: Manifest,
    pub root: std::path::PathBuf,
}

pub fn validate_manifest(manifest: &Manifest, host_version: &Version) -> Result<(), PluginError> {
    if manifest.schema != SCHEMA_VERSION {
        return Err(PluginError::incompatible(format!(
            "plugin '{}' declares schema {} but host understands {}",
            manifest.id, manifest.schema, SCHEMA_VERSION
        )));
    }
    if manifest.abi != super::ABI_VERSION {
        return Err(PluginError::incompatible(format!(
            "plugin '{}' was built against ABI {} but host is ABI {}",
            manifest.id,
            manifest.abi,
            super::ABI_VERSION
        )));
    }
    // 兼容性判定：插件要求的宿主版本必须与宿主主版本一致且不高于宿主。
    if !manifest.min_host.is_compatible_with(host_version) {
        return Err(PluginError::incompatible(format!(
            "plugin '{}' requires host >= {} but host is {}",
            manifest.id, manifest.min_host, host_version
        )));
    }
    // `tier = core` 意味着"用户不能禁用/卸载"。如果磁盘上的清单也能声明它，
    // 任何人只要往用户插件目录扔一个 `"tier": "core"` 的 plugin.json，
    // 就能得到一个**赖着不走**的插件——这是一条提权路径。
    // 因此核心层级只承认编译期内置的插件（`DirectoryLocator` 会把磁盘来源
    // 强制改写为 packaged/user，无法伪造成 builtin）。
    if manifest.tier.is_core() && manifest.source != PluginSource::Builtin {
        return Err(PluginError::permission_denied(format!(
            "plugin '{}' declares tier=core but is loaded from '{}'; \
             only builtin plugins may be core",
            manifest.id,
            manifest.source.as_str()
        )));
    }
    // `entry` 只对需要动态加载的插件有意义：内置插件的代码由宿主直接链接，
    // 加载路径根本不经过 `entry`。强制内置插件填一个假 entry 只会制造
    // 无意义的样板，因此这里按来源区分。
    if manifest.entry.is_empty() && manifest.source != PluginSource::Builtin {
        return Err(PluginError::invalid_argument(format!(
            "plugin '{}' has an empty 'entry'; dynamic plugins need a library base name",
            manifest.id
        )));
    }
    // 注意：`min_version` 约束的是**被依赖插件**的版本，而非宿主版本，
    // 因此不在这里比对，交给依赖解析阶段（见 plugin-runtime 的 resolver）。
    for dependency in &manifest.dependencies {
        if dependency.plugin == manifest.id {
            return Err(PluginError::invalid_argument(format!(
                "plugin '{}' depends on itself",
                manifest.id
            )));
        }
    }
    let mut seen = std::collections::HashSet::new();
    for capability in &manifest.capabilities {
        if !seen.insert(capability.clone()) {
            return Err(PluginError::invalid_argument(format!(
                "plugin '{}' declares duplicate capability '{capability}'",
                manifest.id
            )));
        }
    }
    Ok(())
}