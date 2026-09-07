use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use plugin_sdk::abi::AudioProcessorAbi;
use plugin_sdk::{
    validate_manifest, ActivationContext, ActivationScope, AudioProcessorSpec, Capability,
    CommandId, Contribution, DeactivateReason, DiscoveredPlugin, EventPattern, EventSubscriptions,
    EventType, HealthStatus, HostEvent, InvokeContext, Manifest, PluginDescriptor, PluginError,
    PluginId, PluginResult, PluginTier, Plugin, ServiceId, Version,
};
use serde::{Deserialize, Serialize};

pub mod container;
pub mod contributions;
pub mod deps;
pub mod discovery;
pub mod dynamic;
pub mod lifecycle;
pub mod loader;
pub mod supervisor;

pub use container::{ServiceContainer, ServiceInventoryItem};
pub use contributions::{
    ContributionRecord, ContributionRegistry, ContributionSnapshotItem, RichContribution,
};
pub use deps::DependencyGraph;
pub use discovery::{
    BuiltinLocator, CompositeLocator, DirectoryLocator, DiscoveryIssue, DiscoveryReport,
    PluginLocator,
};
pub use dynamic::{
    as_json_service, deactivate_reason_str, DynamicPlugin, JsonService, JsonServiceRegistry,
};
pub use lifecycle::{can_transition, LifecycleEvent, LifecycleTracker, PluginState, Transition};
pub use loader::{builtin_marker, candidate_paths, library_suffix, LoadedLibrary};
pub use supervisor::{
    combined_health, BreakerPolicy, BreakerState, CallOutcome, CallReport, CascadePolicy,
    CircuitBreaker, Supervisor,
};

pub type BuiltinFactory =
    Arc<dyn Fn() -> PluginResult<Box<dyn Plugin>> + Send + Sync>;

#[derive(Clone, Default)]
pub struct EventSink(Arc<Mutex<Vec<Box<dyn Fn(&HostEvent) + Send + Sync>>>>);

impl EventSink {
    pub fn notify(&self, event: &HostEvent) {
        if let Ok(sinks) = self.0.lock() {
            for sink in sinks.iter() {
                sink(event);
            }
        }
    }

    pub fn register(&self, sink: impl Fn(&HostEvent) + Send + Sync + 'static) {
        if let Ok(mut lock) = self.0.lock() {
            lock.push(Box::new(sink));
        }
    }
}

struct PluginEntry {
    manifest: Manifest,
    root: PathBuf,
    tracker: LifecycleTracker,
    descriptor: Option<PluginDescriptor>,
    instance: Option<Arc<dyn Plugin>>,
    lib: Option<Arc<LoadedLibrary>>,
    subscriptions: EventSubscriptions,
}

impl PluginEntry {
    fn new(discovered: DiscoveredPlugin) -> Self {
        Self {
            manifest: discovered.manifest,
            root: discovered.root,
            tracker: LifecycleTracker::new(PluginState::Discovered),
            descriptor: None,
            instance: None,
            lib: None,
            subscriptions: EventSubscriptions::new(),
        }
    }

    fn instance(&self) -> PluginResult<Arc<dyn Plugin>> {
        self.instance.clone().ok_or_else(|| {
            PluginError::not_found(format!("plugin '{}' is not loaded", self.manifest.id))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationOutcome {
    pub plugin: PluginId,
    pub state: PluginState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub host_version: Version,
    pub plugins: Vec<PluginSnapshot>,
    pub contributions: Vec<ContributionSnapshotItem>,
    pub services: Vec<ServiceInventoryItem>,
    pub issues: Vec<DiscoveryIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginSnapshot {
    pub id: PluginId,
    pub version: Version,
    pub source: plugin_sdk::PluginSource,
    pub tier: PluginTier,
    pub user_disableable: bool,
    pub state: PluginState,
    pub health: HealthStatus,
    pub breaker: BreakerState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub capabilities: Vec<Capability>,
    pub depends_on: Vec<PluginId>,
    pub has_audio_processor: bool,
    pub transitions: Vec<Transition>,
}

pub struct AudioProcessorBinding {
    pub plugin: PluginId,
    pub spec: AudioProcessorSpec,
    pub abi: AudioProcessorAbi,
    pub instance: *mut std::ffi::c_void,
}

// SAFETY: `handle` 只在宿主音频线程上使用，且其有效性由运行时的生命周期
// 状态机保证（激活中才允许被取用，停用会先于卸载发生）。
unsafe impl Send for AudioProcessorBinding {}
unsafe impl Sync for AudioProcessorBinding {}

pub struct PluginRuntime {
    host_version: Version,
    locators: Mutex<Vec<Box<dyn PluginLocator>>>,
    builtins: Mutex<HashMap<PluginId, (Manifest, BuiltinFactory)>>,
    container: Arc<ServiceContainer>,
    json: Arc<JsonServiceRegistry>,
    contributions: Arc<ContributionRegistry>,
    supervisor: Arc<Supervisor>,
    entries: RwLock<HashMap<PluginId, PluginEntry>>,
    graph: RwLock<DependencyGraph>,
    issues: RwLock<Vec<DiscoveryIssue>>,
    sequence: AtomicU64,
    listeners: Mutex<Vec<Box<dyn Fn(&LifecycleEvent) + Send + Sync>>>,
    publishers: EventSink,
}

impl PluginRuntime {
    // —— 构建 ——

    pub fn new(host_version: Version) -> Self {
        Self {
            host_version,
            locators: Mutex::new(Vec::new()),
            builtins: Mutex::new(HashMap::new()),
            container: Arc::new(ServiceContainer::new()),
            json: Arc::new(JsonServiceRegistry::new()),
            contributions: Arc::new(ContributionRegistry::new()),
            supervisor: Arc::new(Supervisor::new(CascadePolicy::StopDependents)),
            entries: RwLock::new(HashMap::new()),
            graph: RwLock::new(DependencyGraph::new()),
            issues: RwLock::new(Vec::new()),
            sequence: AtomicU64::new(1),
            listeners: Mutex::new(Vec::new()),
            publishers: EventSink::default(),
        }
    }

    pub fn on_plugin_event(self, sink: impl Fn(&HostEvent) + Send + Sync + 'static) -> Self {
        self.publishers.register(sink);
        self
    }

    pub fn event_sink(&self) -> EventSink {
        self.publishers.clone()
    }

    pub fn publish(
        &self,
        source: &PluginId,
        kind: EventType,
        payload: serde_json::Value,
    ) -> plugin_sdk::DispatchStats {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        let event = HostEvent::from_plugin(kind, payload, sequence, source.clone());

        self.publishers.notify(&event);
        self.dispatch(&event)
    }

    pub fn with_cascade(mut self, policy: CascadePolicy) -> Self {
        self.supervisor = Arc::new(Supervisor::new(policy));
        self
    }

    pub fn with_locator(self, locator: impl PluginLocator + 'static) -> Self {
        self.locators.lock().ok().map(|mut lock| lock.push(Box::new(locator)));
        self
    }

    pub fn with_builtin(
        self,
        manifest: Manifest,
        factory: impl Fn() -> PluginResult<Box<dyn Plugin>> + Send + Sync + 'static,
    ) -> Self {
        if let Ok(mut lock) = self.builtins.lock() {
            lock.insert(manifest.id.clone(), (manifest, Arc::new(factory)));
        }
        self
    }

    pub fn on_lifecycle(self, listener: impl Fn(&LifecycleEvent) + Send + Sync + 'static) -> Self {
        if let Ok(mut lock) = self.listeners.lock() {
            lock.push(Box::new(listener));
        }
        self
    }

    // —— 子系统的只读句柄 ——

    pub fn services(&self) -> &Arc<ServiceContainer> {
        &self.container
    }
    pub fn json_services(&self) -> &Arc<JsonServiceRegistry> {
        &self.json
    }
    pub fn contributions(&self) -> &Arc<ContributionRegistry> {
        &self.contributions
    }
    pub fn supervisor(&self) -> &Arc<Supervisor> {
        &self.supervisor
    }
    pub fn host_version(&self) -> &Version {
        &self.host_version
    }

    pub fn provide_host_service<T: Send + Sync + 'static>(
        &self,
        descriptor: plugin_sdk::ServiceDescriptor,
        service: Arc<T>,
    ) -> PluginResult<()> {
        self.container.register_host(descriptor, service)
    }

    pub fn provide_host_json_service(&self, service: Arc<dyn JsonService>) {
        self.json.register(service)
    }

    pub fn provide_host_factory<T: Send + Sync + 'static>(
        &self,
        descriptor: plugin_sdk::ServiceDescriptor,
        build: impl Fn(&PluginId) -> Option<Arc<T>> + Send + Sync + 'static,
    ) -> PluginResult<()> {
        self.container.register_factory(
            descriptor,
            Arc::new(move |requester: &PluginId| {
                build(requester).map(|service| {
                    (service as Arc<dyn std::any::Any + Send + Sync>, std::any::TypeId::of::<T>())
                })
            }),
        )
    }

    // —— 阶段一：发现 ——

    pub fn discover(&self) -> DiscoveryReport {
        let mut report = DiscoveryReport::default();

        let locators = self.locators.lock();
        if let Ok(lock) = &locators {
            for locator in lock.iter() {
                report.merge(locator.locate(&self.host_version));
            }
        }
        drop(locators);

        // 内置插件由代码登记，不经过目录扫描；这里单独合入。
        if let Ok(builtins) = self.builtins.lock() {
            for (_, (manifest, _)) in builtins.iter() {
                report.found.push(DiscoveredPlugin {
                    manifest: manifest.clone(),
                    root: builtin_marker(&manifest.id),
                });
            }
        }

        // 去重。规则：**内置插件优先于任何目录里的同名插件**，其余按发现顺序
        // 先到先得。这条规则防的是"用户目录里塞一个 lyrics 就能顶掉内置实现"。
        let builtin_ids: Vec<PluginId> = self
            .builtins
            .lock()
            .map(|lock| lock.keys().cloned().collect())
            .unwrap_or_default();

        let found = std::mem::take(&mut report.found);
        let (builtin_items, mut rest): (Vec<DiscoveredPlugin>, Vec<DiscoveredPlugin>) =
            found.into_iter().partition(|item| {
                builtin_ids.iter().any(|id| *id == item.manifest.id)
            });

        let mut accepted: Vec<DiscoveredPlugin> = Vec::with_capacity(builtin_items.len());
        let mut seen: Vec<PluginId> = Vec::new();

        for item in builtin_items {
            if seen.iter().any(|id| *id == item.manifest.id) {
                report.issues.push(DiscoveryIssue {
                    path: item.root.clone(),
                    plugin: Some(item.manifest.id.to_string()),
                    message: format!(
                        "duplicate builtin plugin id '{}'; the first one wins",
                        item.manifest.id
                    ),
                });
                continue;
            }
            seen.push(item.manifest.id.clone());
            accepted.push(item);
        }
        // 目录项按 root 排序，保证"先发现者"是确定的，而不是依赖 readdir 顺序。
        rest.sort_by(|a, b| a.root.cmp(&b.root));
        for item in rest {
            if seen.iter().any(|id| *id == item.manifest.id) {
                report.issues.push(DiscoveryIssue {
                    path: item.root.clone(),
                    plugin: Some(item.manifest.id.to_string()),
                    message: format!(
                        "duplicate plugin id '{}'; the earlier one wins",
                        item.manifest.id
                    ),
                });
                continue;
            }
            seen.push(item.manifest.id.clone());
            accepted.push(item);
        }

        // 二次校验：定位器各自校验过，但合入后仍可能因宿主版本变化而失效。
        let mut entries = HashMap::new();
        let mut accepted_out = Vec::with_capacity(accepted.len());
        for item in accepted {
            let id = item.manifest.id.clone();
            if let Err(error) = validate_manifest(&item.manifest, &self.host_version) {
                report.issues.push(DiscoveryIssue {
                    path: item.root.clone(),
                    plugin: Some(id.to_string()),
                    message: error.to_string(),
                });
                continue;
            }
            entries.insert(id.clone(), PluginEntry::new(item.clone()));
            accepted_out.push(item);
        }
        // `report.found` 只保留**被接受**的插件，方便前端直接展示。
        report.found = accepted_out;

        {
            let Ok(mut lock) = self.entries.write() else {
                return report;
            };
            // 已激活的插件在重新发现时保留运行状态：只替换清单与根目录。
            for (id, entry) in entries {
                match lock.get_mut(&id) {
                    Some(existing)
                        if matches!(
                            existing.tracker.state(),
                            PluginState::Active | PluginState::Deactivating | PluginState::Activating
                        ) =>
                    {
                        existing.manifest = entry.manifest;
                        existing.root = entry.root;
                    }
                    _ => {
                        lock.insert(id, entry);
                    }
                }
            }
            // 已被卸载（磁盘上消失）的插件：标记为可重新解析。
            for existing in lock.values_mut() {
                if existing.tracker.state() == PluginState::Unloaded {
                    let _ = existing.tracker.transition(PluginState::Resolved, "rediscovered");
                }
            }
        }
        self.rebuild_graph();
        if let Ok(mut lock) = self.issues.write() {
            *lock = report.issues.clone();
        }
        report
    }

    fn rebuild_graph(&self) {
        let Ok(lock) = self.entries.read() else {
            return;
        };
        let discovered: Vec<DiscoveredPlugin> = lock
            .values()
            .map(|entry| DiscoveredPlugin {
                manifest: entry.manifest.clone(),
                root: entry.root.clone(),
            })
            .collect();
        drop(lock);
        if let Ok(mut graph) = self.graph.write() {
            *graph = DependencyGraph::build(discovered.iter());
        }
    }

    // —— 阶段二：依赖解析 + 加载 + 激活 ——

    pub fn activate_all(&self) -> Vec<ActivationOutcome> {
        let targets: Vec<PluginId> = {
            let Ok(lock) = self.entries.read() else {
                return Vec::new();
            };
            lock.values()
                .filter(|entry| entry.manifest.activation.eager)
                .map(|entry| entry.manifest.id.clone())
                .collect()
        };

        let order = {
            let Ok(graph) = self.graph.read() else {
                return Vec::new();
            };
            match graph.activation_order(&targets) {
                Ok(order) => order,
                Err(error) => {
                    // 拓扑不可解（缺依赖 / 版本冲突 / 有环）：整体拒绝，
                    // 但把每个目标都回报成失败，方便前端一次性展示。
                    return targets
                        .into_iter()
                        .map(|plugin| ActivationOutcome {
                            plugin,
                            state: PluginState::Failed,
                            error: Some(error.to_string()),
                        })
                        .collect();
                }
            }
        };

        order
            .iter()
            .map(|id| match self.activate(id) {
                Ok(()) => ActivationOutcome {
                    plugin: id.clone(),
                    state: self.state_of(id).unwrap_or(PluginState::Failed),
                    error: None,
                },
                Err(error) => ActivationOutcome {
                    plugin: id.clone(),
                    state: self.state_of(id).unwrap_or(PluginState::Failed),
                    error: Some(error.to_string()),
                },
            })
            .collect()
    }

    pub fn activate(&self, id: &PluginId) -> PluginResult<()> {
        self.ensure_resolved(id)?;
        self.ensure_loaded(id)?;
        self.activate_transaction(id)
    }

    fn ensure_resolved(&self, id: &PluginId) -> PluginResult<()> {
        let mut lock = self
            .entries
            .write()
            .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
        let entry = lock
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(format!("plugin '{id}' is not installed")))?;
        if entry.tracker.state() == PluginState::Discovered {
            entry.tracker.transition(PluginState::Resolved, "dependencies resolved")?;
        }
        Ok(())
    }

    fn ensure_loaded(&self, id: &PluginId) -> PluginResult<()> {
        let (state, manifest, root) = {
            let lock = self
                .entries
                .read()
                .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
            let entry = lock
                .get(id)
                .ok_or_else(|| PluginError::not_found(format!("plugin '{id}' is not installed")))?;
            (
                entry.tracker.state(),
                entry.manifest.clone(),
                entry.root.clone(),
            )
        };

        if state != PluginState::Resolved {
            // 已加载 / 已激活 → 直接跳过；其他状态由调用方负责。
            return Ok(());
        }

        let builtin = self
            .builtins
            .lock()
            .ok()
            .and_then(|lock| lock.get(id).map(|(_, factory)| Arc::clone(factory)));

        let (instance, lib): (Arc<dyn Plugin>, Option<Arc<LoadedLibrary>>) = match builtin {
            Some(factory) => {
                let boxed = factory()?;
                verify_descriptor(&manifest, boxed.descriptor())?;
                (Arc::from(boxed), None)
            }
            None => {
                let library = Arc::new(LoadedLibrary::open(id, &root)?);
                let plugin = DynamicPlugin::create(
                    Arc::clone(&library),
                    Arc::clone(&self.container),
                    Arc::clone(&self.json),
                    id.clone(),
                )?;
                verify_descriptor(&manifest, plugin.descriptor())?;
                (Arc::new(plugin), Some(library))
            }
        };

        let mut lock = self
            .entries
            .write()
            .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
        let entry = lock
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(format!("plugin '{id}' is not installed")))?;
        if entry.tracker.state() != PluginState::Resolved {
            // 竞态：别的线程已经加载过了，丢弃本次结果。
            drop(instance);
            drop(lib);
            return Ok(());
        }
        entry.descriptor = Some(instance.descriptor().clone());
        entry.instance = Some(instance);
        entry.lib = lib;
        entry
            .tracker
            .transition(PluginState::Loaded, "code loaded")?;
        self.supervisor.register(id, &manifest.budget);
        Ok(())
    }

    fn activate_transaction(&self, id: &PluginId) -> PluginResult<()> {
        let (instance, manifest, descriptor) = {
            let mut lock = self
                .entries
                .write()
                .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
            let entry = lock
                .get_mut(id)
                .ok_or_else(|| PluginError::not_found(format!("plugin '{id}' is not installed")))?;
            let state = entry.tracker.state();
            if !state.can_activate() {
                return Err(PluginError::invalid_argument(format!(
                    "plugin '{id}' cannot be activated from state '{}'",
                    state.label()
                )));
            }
            entry.tracker.transition(PluginState::Activating, "activate")?;
            (
                entry.instance()?,
                entry.manifest.clone(),
                entry.descriptor.clone(),
            )
        };

        // 授权与依赖声明是**覆盖式**的：每次激活前重置，避免残留上一次的授权。
        let capabilities: Vec<Capability> = manifest.capabilities.clone();
        self.container.grant(id, capabilities.iter().cloned());
        let providers: Vec<PluginId> = manifest
            .dependencies
            .iter()
            .map(|dependency| dependency.plugin.clone())
            .collect();
        self.container
            .declare_dependencies(id, providers.iter().cloned());

        // 运行期能力必须是清单静态声明的子集，否则清单就失去了意义。
        if let Some(descriptor) = &descriptor {
            for capability in &descriptor.capabilities {
                if !capabilities.contains(capability) {
                    let error = PluginError::permission_denied(format!(
                        "plugin '{id}' requested capability '{capability}' at runtime, \
                         which is not declared in its manifest"
                    ));
                    self.rollback_activation(id, &error);
                    return Err(error);
                }
            }
        }

        let timeout = Duration::from_millis(manifest.budget.call_timeout_ms.max(1));
        let container = Arc::clone(&self.container);
        let capabilities = capabilities.clone();
        let plugin_id = id.clone();
        let callee = Arc::clone(&instance);

        let outcome = self.supervisor.guard(id, timeout, move || {
            let mut scope = ActivationScope::new();
            {
                let mut ctx = ActivationContext::new(
                    &plugin_id,
                    &mut scope,
                    container.as_ref(),
                    &capabilities,
                );
                callee.activate(&mut ctx)?;
            }
            Ok(scope)
        });

        match outcome {
            Ok(mut scope) => {
                let services = scope.take_services();
                let mut registered: Vec<ServiceId> = Vec::new();
                let mut failure: Option<PluginError> = None;

                for binding in services {
                    let service_id = binding.descriptor().id.clone();
                    // 动态库插件提供的服务要**同时**登记到 JSON 通道，
                    // 否则别的 packaged 插件无法解析它。
                    if let Some(json) = as_json_service(&binding) {
                        self.json.register(json);
                    }
                    if let Err(error) = self.container.register_plugin(id, binding) {
                        failure = Some(error);
                        break;
                    }
                    registered.push(service_id);
                }

                if let Some(error) = failure {
                    // 已经注册成功的一部分也要撤掉，保证事务性。
                    let _ = self.container.revoke_provider(id);
                    self.json.remove_provider(id);
                    self.rollback_activation(id, &error);
                    return Err(error);
                }

                let contributions: Vec<Contribution> = scope.contributions().to_vec();
                let mut subscriptions = EventSubscriptions::new();
                for pattern in scope.subscriptions() {
                    subscriptions = subscriptions.add(pattern.clone());
                }

                {
                    let mut lock = self
                        .entries
                        .write()
                        .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
                    if let Some(entry) = lock.get_mut(id) {
                        entry.subscriptions = subscriptions;
                        self.contributions.apply(id, contributions);
                        entry.tracker.clear_error();
                        if let Err(error) = entry.tracker.transition(PluginState::Active, "activated") {
                            self.contributions.revoke(id);
                            let _ = self.container.revoke_provider(id);
                            self.json.remove_provider(id);
                            return Err(error);
                        }
                    }
                }
                self.emit_lifecycle(id, PluginState::Activating, PluginState::Active, "activated");
                Ok(())
            }
            Err(error) => {
                self.rollback_activation(id, &error);
                // 级联：熔断开启且策略允许时，停掉依赖它的插件。
                if self.supervisor.should_cascade()
                    && matches!(self.supervisor.state_of(id), BreakerState::Open { .. })
                {
                    self.cascade_stop(id);
                }
                Err(error)
            }
        }
    }

    fn rollback_activation(&self, id: &PluginId, error: &PluginError) {
        self.contributions.revoke(id);
        let _ = self.container.revoke_provider(id);
        self.json.remove_provider(id);
        self.container.revoke_grants(id);
        if let Ok(mut lock) = self.entries.write() {
            if let Some(entry) = lock.get_mut(id) {
                entry.subscriptions = EventSubscriptions::new();
                entry.tracker.mark_failed(error);
            }
        }
        self.emit_lifecycle(id, PluginState::Activating, PluginState::Failed, "activate failed");
    }

    // —— 阶段三：运行（事件 / 命令）——

    pub fn dispatch(&self, event: &HostEvent) -> plugin_sdk::DispatchStats {
        let mut stats = plugin_sdk::DispatchStats::default();

        // 先做一次全表剪枝：没有任何插件关心这个 kind 就直接返回。
        let targets: Vec<(PluginId, Arc<dyn Plugin>)> = {
            let Ok(lock) = self.entries.read() else {
                return stats;
            };
            let mut matched: Vec<(PluginId, Arc<dyn Plugin>)> = lock
                .values()
                .filter(|entry| entry.tracker.state().is_active())
                .filter(|entry| entry.subscriptions.may_match(&event.kind))
                .filter(|entry| entry.subscriptions.matches(event))
                .filter_map(|entry| {
                    entry
                        .instance
                        .as_ref()
                        .map(|instance| (entry.manifest.id.clone(), Arc::clone(instance)))
                })
                .collect();
            // 排序保证派发顺序稳定（不依赖 HashMap 迭代顺序）。
            matched.sort_by(|a, b| a.0.cmp(&b.0));
            matched
        };

        let skipped = {
            let Ok(lock) = self.entries.read() else {
                return stats;
            };
            lock.values()
                .filter(|entry| entry.tracker.state().is_active())
                .filter(|entry| !entry.subscriptions.may_match(&event.kind))
                .count() as u64
        };
        stats.skipped = skipped;

        for (id, instance) in targets {
            if event.source.as_ref() == Some(&id) {
                continue; // 不把事件回灌给产生它的插件。
            }
            *stats.delivered.entry(id.to_string()).or_insert(0) += 1;

            let timeout = self.call_timeout_of(&id);
            let container = Arc::clone(&self.container);
            let plugin_id = id.clone();
            let event = event.clone();
            let result = self.supervisor.guard(&id, timeout, move || {
                let ctx = InvokeContext::new(&plugin_id, container.as_ref());
                instance.on_event(&event, &ctx)
            });

            if let Err(error) = result {
                *stats.failed.entry(id.to_string()).or_insert(0) += 1;
                if let Ok(mut lock) = self.entries.write() {
                    if let Some(entry) = lock.get_mut(&id) {
                        entry
                            .tracker
                            .record_error(&error.to_string());
                    }
                }
                if self.supervisor.should_cascade()
                    && matches!(self.supervisor.state_of(&id), BreakerState::Open { .. })
                {
                    self.cascade_stop(&id);
                }
            }
        }
        stats
    }

    pub fn emit(&self, kind: EventType, payload: serde_json::Value) -> plugin_sdk::DispatchStats {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        self.dispatch(&HostEvent::host(kind, payload, sequence))
    }

    pub fn execute(
        &self,
        command: &CommandId,
        args: serde_json::Value,
    ) -> PluginResult<serde_json::Value> {
        let owner = self
            .contributions
            .owner_of_command(command)
            .ok_or_else(|| PluginError::not_found(format!("no plugin handles command '{command}'")))?;

        // 懒激活：命令被显式声明为激活触发器的插件在此刻加载。
        if !self.is_active(&owner) {
            self.activate(&owner)?;
        }

        let instance = {
            let Ok(lock) = self.entries.read() else {
                return Err(PluginError::io("plugin table poisoned"));
            };
            let entry = lock.get(&owner).ok_or_else(|| {
                PluginError::not_found(format!("plugin '{owner}' vanished before execution"))
            })?;
            entry.instance()?
        };

        let timeout = self.call_timeout_of(&owner);
        let container = Arc::clone(&self.container);
        let plugin_id = owner.clone();
        let command = command.clone();
        let result = self.supervisor.guard(&owner, timeout, move || {
            let ctx = InvokeContext::new(&plugin_id, container.as_ref());
            instance.execute(&command, args, &ctx)
        });

        if let Err(error) = &result {
            if let Ok(mut lock) = self.entries.write() {
                if let Some(entry) = lock.get_mut(&owner) {
                    entry
                        .tracker
                        .record_error(&error.to_string());
                }
            }
        }
        result
    }

    fn call_timeout_of(&self, id: &PluginId) -> Duration {
        let millis = self
            .entries
            .read()
            .ok()
            .and_then(|lock| {
                lock.get(id)
                    .map(|entry| entry.manifest.budget.call_timeout_ms)
            })
            .unwrap_or(2_000);
        Duration::from_millis(millis.max(1))
    }

    pub fn subscribe(&self, id: &PluginId, patterns: Vec<EventPattern>) -> PluginResult<()> {
        let mut lock = self
            .entries
            .write()
            .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
        let entry = lock
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(format!("plugin '{id}' is not installed")))?;
        for pattern in patterns {
            entry.subscriptions = std::mem::take(&mut entry.subscriptions).add(pattern);
        }
        Ok(())
    }

    // —— 阶段四：热插拔 ——

    pub fn tier_of(&self, id: &PluginId) -> Option<PluginTier> {
        self.entries
            .read()
            .ok()
            .and_then(|lock| lock.get(id).map(|entry| entry.manifest.tier))
    }

    fn guard_user_action(&self, id: &PluginId, action: &str) -> PluginResult<()> {
        if self.tier_of(id).map(PluginTier::is_core).unwrap_or(false) {
            return Err(PluginError::permission_denied(format!(
                "plugin '{id}' is a core plugin and cannot be {action} by the user"
            )));
        }
        Ok(())
    }

    pub fn deactivate(&self, id: &PluginId, reason: DeactivateReason) -> PluginResult<()> {
        if matches!(reason, DeactivateReason::UserDisabled) {
            self.guard_user_action(id, "disabled")?;
        }
        let (instance, timeout, state) = {
            let mut lock = self
                .entries
                .write()
                .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
            let entry = lock
                .get_mut(id)
                .ok_or_else(|| PluginError::not_found(format!("plugin '{id}' is not installed")))?;
            let state = entry.tracker.state();
            if !state.can_deactivate() {
                // 已经停了就是幂等成功。
                if matches!(state, PluginState::Stopped | PluginState::Loaded) {
                    return Ok(());
                }
                return Err(PluginError::invalid_argument(format!(
                    "plugin '{id}' cannot be deactivated from state '{}'",
                    state.label()
                )));
            }
            entry.tracker.transition(PluginState::Deactivating, reason_label(reason))?;
            (
                entry.instance()?,
                Duration::from_millis(entry.manifest.budget.call_timeout_ms.max(1)),
                state,
            )
        };

        // 先撤销外部可见的东西，再通知插件：即使插件的 `deactivate` 卡住或
        self.contributions.revoke(id);
        let revoked_services = self.container.revoke_provider(id);
        self.json.remove_provider(id);
        self.container.revoke_grants(id);

        let teardown_owner = id.clone();
        let result = self.supervisor.guard(id, timeout, move || {
            let ctx = plugin_sdk::TeardownContext::new(teardown_owner, reason);
            instance.deactivate(&ctx)
        });

        let mut lock = self
            .entries
            .write()
            .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
        if let Some(entry) = lock.get_mut(id) {
            entry.subscriptions = EventSubscriptions::new();
            match &result {
                Ok(()) => {
                    entry.tracker.clear_error();
                    let _ = entry
                        .tracker
                        .transition(PluginState::Stopped, reason_label(reason));
                }
                Err(error) => entry.tracker.mark_failed(error),
            }
        }
        drop(lock);

        self.emit_lifecycle(id, state, PluginState::Stopped, reason_label(reason));
        if !revoked_services.is_empty() {
            // 通知依赖方"能力已消失"，让它们有机会降级而不是默默拿到错误。
            let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
            let _ = self.dispatch(&HostEvent::host(
                EventType::new("plugin.serviceRevoked").unwrap_or_else(|_| plugin_lifecycle_type()),
                serde_json::json!({
                    "plugin": id.to_string(),
                    "services": revoked_services.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                }),
                sequence,
            ));
        }
        result.map(|_| ())
    }

    pub fn reload(&self, id: &PluginId) -> PluginResult<()> {
        let _ = self.deactivate(id, DeactivateReason::Reload);
        self.unload(id)?;
        self.activate(id)
    }

    pub fn uninstall(&self, id: &PluginId) -> PluginResult<Vec<PluginId>> {
        self.guard_user_action(id, "uninstalled")?;
        let impact = {
            let Ok(graph) = self.graph.read() else {
                return Err(PluginError::io("dependency graph poisoned"));
            };
            graph.unload_impact(id)
        };
        let order = {
            let Ok(graph) = self.graph.read() else {
                return Err(PluginError::io("dependency graph poisoned"));
            };
            graph.deactivation_order(&impact)
        };

        // 受影响闭包里若含核心插件，整体拒绝：不允许通过"卸载 A 会级联停 B"
        // 这条侧路把核心插件带下来。
        if let Some(core) = order
            .iter()
            .find(|target| self.tier_of(target).map(PluginTier::is_core).unwrap_or(false))
        {
            return Err(PluginError::permission_denied(format!(
                "uninstalling '{id}' would also stop core plugin '{core}'; refused"
            )));
        }

        for target in &order {
            let _ = self.deactivate(target, DeactivateReason::DependencyRemoved);
        }
        // 逆序卸载：被依赖者最后释放。
        for target in order.iter().rev() {
            self.unload(target)?;
        }
        Ok(order)
    }

    fn unload(&self, id: &PluginId) -> PluginResult<()> {
        let mut lock = self
            .entries
            .write()
            .map_err(|e| PluginError::io(format!("plugin table poisoned: {e}")))?;
        let entry = lock
            .get_mut(id)
            .ok_or_else(|| PluginError::not_found(format!("plugin '{id}' is not installed")))?;
        let state = entry.tracker.state();
        if !state.can_unload() {
            return Err(PluginError::invalid_argument(format!(
                "plugin '{id}' cannot be unloaded from state '{}'; deactivate it first",
                state.label()
            )));
        }

        // 丢弃实例与库句柄。**顺序重要**：实例析构会调用插件的 `destroy`，
        // 此时动态库必须还在内存里，因此 `lib` 放在最后 drop。
        entry.instance = None;
        entry.descriptor = None;
        entry.subscriptions = EventSubscriptions::new();
        entry.lib = None;
        self.supervisor.unregister(id);
        self.container.revoke_grants(id);
        let _ = self.container.revoke_provider(id);
        self.json.remove_provider(id);
        self.contributions.revoke(id);
        entry.tracker.transition(PluginState::Unloaded, "code released")?;
        self.emit_lifecycle(id, state, PluginState::Unloaded, "code released");
        Ok(())
    }

    fn cascade_stop(&self, id: &PluginId) {
        let dependents = {
            let Ok(graph) = self.graph.read() else {
                return;
            };
            graph.dependents_of(id)
        };
        for dependent in dependents {
            if self.is_active(&dependent) {
                let _ = self.deactivate(&dependent, DeactivateReason::DependencyRemoved);
            }
        }
    }

    // —— 查询与诊断 ——

    pub fn state_of(&self, id: &PluginId) -> Option<PluginState> {
        self.entries
            .read()
            .ok()
            .and_then(|lock| lock.get(id).map(|entry| entry.tracker.state()))
    }

    pub fn is_active(&self, id: &PluginId) -> bool {
        self.state_of(id).map(PluginState::is_active).unwrap_or(false)
    }

    pub fn installed(&self) -> Vec<PluginId> {
        let mut ids: Vec<PluginId> = self
            .entries
            .read()
            .map(|lock| lock.keys().cloned().collect())
            .unwrap_or_default();
        ids.sort();
        ids
    }

    pub fn plugin_manifests(&self) -> Vec<Manifest> {
        let mut manifests: Vec<Manifest> = self
            .entries
            .read()
            .map(|lock| lock.values().map(|entry| entry.manifest.clone()).collect())
            .unwrap_or_default();
        manifests.sort_by(|a, b| a.id.cmp(&b.id));
        manifests
    }

    pub fn health_of(&self, id: &PluginId) -> HealthStatus {
        let reported = self
            .entries
            .read()
            .ok()
            .and_then(|lock| {
                lock.get(id).and_then(|entry| entry.instance.clone())
            })
            .map(|instance| {
                // 健康自检也受超时保护：自检卡住不能拖住诊断面板。
                self.supervisor
                    .guard(id, Duration::from_millis(200), move || Ok(instance.health()))
                    .unwrap_or(HealthStatus::Degraded)
            })
            .unwrap_or(HealthStatus::Degraded);
        combined_health(reported, self.supervisor.state_of(id))
    }

    pub fn audio_processors(&self) -> Vec<AudioProcessorBinding> {
        let specs = self.contributions.audio_processors();
        // 先取句柄再释放锁：下面要把它们交给音频线程，不能在持锁状态下
        // 做别的事（未来若音频线程反向查询状态，就是死锁）。
        let bindings: Vec<AudioProcessorBinding> = {
            let Ok(lock) = self.entries.read() else {
                return Vec::new();
            };
            specs
                .into_iter()
                .filter(|(plugin, _)| {
                    lock.get(plugin)
                        .map(|entry| entry.tracker.state().is_active())
                        .unwrap_or(false)
                })
                .filter_map(|(plugin, spec)| {
                    let handle = lock.get(&plugin)?.instance.as_ref()?.audio_processor()?;
                    Some(AudioProcessorBinding {
                        plugin,
                        spec,
                        abi: handle.abi,
                        instance: handle.handle,
                    })
                })
                .collect()
        };
        bindings
    }

    pub fn diagnostics(&self) -> RuntimeSnapshot {
        let audio_owners: Vec<PluginId> = self
            .contributions
            .audio_processors()
            .into_iter()
            .map(|(plugin, _)| plugin)
            .collect();

        let mut plugins: Vec<PluginSnapshot> = {
            let Ok(lock) = self.entries.read() else {
                return RuntimeSnapshot {
                    host_version: self.host_version.clone(),
                    plugins: Vec::new(),
                    contributions: Vec::new(),
                    services: Vec::new(),
                    issues: self.issues.read().map(|l| l.clone()).unwrap_or_default(),
                };
            };
            lock.values()
                .map(|entry| PluginSnapshot {
                    id: entry.manifest.id.clone(),
                    version: entry.manifest.version.clone(),
                    source: entry.manifest.source,
                    tier: entry.manifest.tier,
                    user_disableable: entry.manifest.is_user_disableable(),
                    state: entry.tracker.state(),
                    // 先占位，锁外再填。
                    health: HealthStatus::Degraded,
                    breaker: self.supervisor.state_of(&entry.manifest.id),
                    last_error: entry.tracker.last_error().map(str::to_string),
                    capabilities: entry.manifest.capabilities.clone(),
                    depends_on: entry
                        .manifest
                        .dependencies
                        .iter()
                        .map(|d| d.plugin.clone())
                        .collect(),
                    has_audio_processor: audio_owners.contains(&entry.manifest.id),
                    transitions: entry.tracker.history().iter().cloned().collect(),
                })
                .collect()
        };

        for plugin in &mut plugins {
            plugin.health = self.health_of(&plugin.id);
        }
        plugins.sort_by(|a, b| a.id.cmp(&b.id));

        RuntimeSnapshot {
            host_version: self.host_version.clone(),
            plugins,
            contributions: self.contributions.snapshot(),
            services: self.container.inventory(),
            issues: self.issues.read().map(|l| l.clone()).unwrap_or_default(),
        }
    }

    pub fn shutdown(&self) {
        let order = {
            let Ok(graph) = self.graph.read() else {
                return;
            };
            graph.deactivation_order(&self.installed())
        };
        for id in order {
            if self.is_active(&id) {
                let _ = self.deactivate(&id, DeactivateReason::Shutdown);
            }
        }
    }

    fn emit_lifecycle(&self, id: &PluginId, from: PluginState, to: PluginState, reason: &str) {
        let event = LifecycleEvent {
            plugin: id.clone(),
            from,
            to,
            reason: reason.to_string(),
            at_ms: lifecycle::now_ms(),
        };
        if let Ok(listeners) = self.listeners.lock() {
            for listener in listeners.iter() {
                listener(&event);
            }
        }
    }
}

// —— 辅助函数 ——

fn verify_descriptor(manifest: &Manifest, descriptor: &PluginDescriptor) -> PluginResult<()> {
    if descriptor.id != manifest.id {
        return Err(PluginError::incompatible(format!(
            "plugin '{}' self-reports as '{}'",
            manifest.id, descriptor.id
        )));
    }
    if descriptor.abi != manifest.abi || descriptor.abi != plugin_sdk::ABI_VERSION {
        return Err(PluginError::incompatible(format!(
            "plugin '{}' was built against ABI {}, host is ABI {}",
            manifest.id,
            descriptor.abi,
            plugin_sdk::ABI_VERSION
        )));
    }
    for capability in &descriptor.capabilities {
        if !manifest.capabilities.contains(capability) {
            return Err(PluginError::permission_denied(format!(
                "plugin '{}' declares capability '{capability}' at runtime \
                 but not in its manifest",
                manifest.id
            )));
        }
    }
    Ok(())
}

fn reason_label(reason: DeactivateReason) -> &'static str {
    deactivate_reason_str(reason)
}

fn plugin_lifecycle_type() -> EventType {
    plugin_sdk::event::kinds::plugin_lifecycle()
}