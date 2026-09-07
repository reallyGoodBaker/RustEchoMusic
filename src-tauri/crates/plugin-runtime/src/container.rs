use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use plugin_sdk::{
    AnyService, Capability, PluginError, PluginId, PluginResult, ResolvedService, ServiceBinding,
    ServiceDescriptor, ServiceId, ServiceRef, ServiceResolver, ServiceSlot, ServiceVisibility,
    Version,
};

pub type ServiceFactory =
    Arc<dyn Fn(&PluginId) -> Option<(Arc<AnyService>, TypeId)> + Send + Sync>;

struct ServiceEntry {
    provider: Option<PluginId>,
    descriptor: ServiceDescriptor,
    service: Option<Arc<AnyService>>,
    type_id: TypeId,
    slot: Arc<ServiceSlot>,
    factory: Option<ServiceFactory>,
}

pub struct ServiceContainer {
    entries: RwLock<HashMap<ServiceId, Vec<ServiceEntry>>>,
    granted: RwLock<HashMap<PluginId, HashSet<Capability>>>,
    declared: RwLock<HashMap<PluginId, HashSet<PluginId>>>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            granted: RwLock::new(HashMap::new()),
            declared: RwLock::new(HashMap::new()),
        }
    }

    // —— 注册 ——

    pub fn register_host<T: Send + Sync + 'static>(
        &self,
        descriptor: ServiceDescriptor,
        service: Arc<T>,
    ) -> PluginResult<()> {
        self.insert(ServiceEntry {
            provider: None,
            descriptor,
            service: Some(service),
            type_id: TypeId::of::<T>(),
            slot: Arc::new(ServiceSlot::new(None)),
            factory: None,
        })
    }

    pub fn register_plugin(
        &self,
        provider: &PluginId,
        binding: ServiceBinding,
    ) -> PluginResult<()> {
        let (descriptor, service, type_id) = binding.into_parts();
        self.insert(ServiceEntry {
            provider: Some(provider.clone()),
            descriptor,
            service: Some(service),
            type_id,
            slot: Arc::new(ServiceSlot::new(Some(provider.clone()))),
            factory: None,
        })
    }

    pub fn register_factory(
        &self,
        descriptor: ServiceDescriptor,
        factory: ServiceFactory,
    ) -> PluginResult<()> {
        self.insert(ServiceEntry {
            provider: None,
            descriptor,
            service: None,
            // 工厂型条目的 `type_id` 由工厂自己返回，这里填一个占位值。
            type_id: TypeId::of::<()>(),
            slot: Arc::new(ServiceSlot::new(None)),
            factory: Some(factory),
        })
    }

    fn insert(&self, entry: ServiceEntry) -> PluginResult<()> {
        let mut lock = self
            .entries
            .write()
            .map_err(|e| PluginError::io(format!("service registry poisoned: {e}")))?;
        let bucket = lock.entry(entry.descriptor.id.clone()).or_default();
        // 同一提供方重复注册同一服务 → 覆盖（幂等，支持热重载）。
        bucket.retain(|existing| existing.provider != entry.provider);
        bucket.push(entry);
        Ok(())
    }

    // —— 授权 ——

    pub fn grant(
        &self,
        plugin: &PluginId,
        capabilities: impl IntoIterator<Item = Capability>,
    ) {
        if let Ok(mut lock) = self.granted.write() {
            lock.insert(plugin.clone(), capabilities.into_iter().collect());
        }
    }

    pub fn declare_dependencies(
        &self,
        dependent: &PluginId,
        providers: impl IntoIterator<Item = PluginId>,
    ) {
        if let Ok(mut lock) = self.declared.write() {
            lock.insert(dependent.clone(), providers.into_iter().collect());
        }
    }

    pub fn revoke_grants(&self, plugin: &PluginId) {
        if let Ok(mut lock) = self.granted.write() {
            lock.remove(plugin);
        }
        if let Ok(mut lock) = self.declared.write() {
            lock.remove(plugin);
        }
    }

    // —— 撤销 ——

    pub fn revoke_provider(&self, provider: &PluginId) -> Vec<ServiceId> {
        let mut revoked = Vec::new();
        if let Ok(mut lock) = self.entries.write() {
            for (id, bucket) in lock.iter_mut() {
                let before = bucket.len();
                for entry in bucket.iter() {
                    if entry.provider.as_ref() == Some(provider) {
                        entry.slot.revoke();
                    }
                }
                bucket.retain(|entry| entry.provider.as_ref() != Some(provider));
                if bucket.len() != before {
                    revoked.push(id.clone());
                }
            }
            lock.retain(|_, bucket| !bucket.is_empty());
        }
        revoked
    }

    // —— 解析 ——

    pub fn resolve_for(
        &self,
        requester: Option<&PluginId>,
        id: &ServiceId,
        type_id: TypeId,
    ) -> PluginResult<ResolvedService> {
        let lock = self
            .entries
            .read()
            .map_err(|e| PluginError::io(format!("service registry poisoned: {e}")))?;
        let bucket = lock
            .get(id)
            .filter(|bucket| !bucket.is_empty())
            .ok_or_else(|| {
                PluginError::service_unavailable(format!("service '{id}' is not registered"))
            })?;

        // 优先选择宿主提供的实现，其次按注册顺序取第一个插件实现。
        // 工厂型条目的 `type_id` 由工厂返回，不参与这里的匹配。
        let entry = bucket
            .iter()
            .find(|entry| {
                entry.provider.is_none()
                    && (entry.factory.is_some() || entry.type_id == type_id)
            })
            .or_else(|| {
                bucket
                    .iter()
                    .find(|entry| entry.factory.is_some() || entry.type_id == type_id)
            })
            .ok_or_else(|| {
                PluginError::service_unavailable(format!(
                    "service '{id}' is registered but not with the requested type"
                ))
            })?;

        self.check_visible(requester, entry)?;
        self.check_capabilities(requester, entry)?;

        if !entry.slot.is_alive() {
            return Err(PluginError::service_unavailable(format!(
                "service '{id}' has been revoked"
            )));
        }

        // 工厂型：为本次请求方现造一个实例。
        if let Some(factory) = &entry.factory {
            let Some(plugin) = requester else {
                // 宿主自身没有"自己的 id"可传，这类服务对它无意义。
                return Err(PluginError::invalid_argument(format!(
                    "service '{id}' is per-plugin and cannot be resolved by the host itself"
                )));
            };
            let (service, produced_type) = factory(plugin).ok_or_else(|| {
                PluginError::service_unavailable(format!(
                    "service '{id}' is unavailable for plugin '{plugin}'"
                ))
            })?;
            if produced_type != type_id {
                return Err(PluginError::service_unavailable(format!(
                    "service '{id}' is registered but not with the requested type"
                )));
            }
            // 工厂由宿主提供，租约永不失效（宿主不会先于插件消失）。
            return Ok(ResolvedService::new(
                Arc::new(ServiceSlot::new(None)),
                service,
            ));
        }

        let service = entry.service.as_ref().ok_or_else(|| {
            PluginError::service_unavailable(format!("service '{id}' has no live instance"))
        })?;
        Ok(ResolvedService::new(
            Arc::clone(&entry.slot),
            Arc::clone(service),
        ))
    }

    pub fn authorize(&self, requester: Option<&PluginId>, id: &ServiceId) -> PluginResult<()> {
        let lock = self
            .entries
            .read()
            .map_err(|e| PluginError::io(format!("service registry poisoned: {e}")))?;
        let bucket = lock
            .get(id)
            .filter(|bucket| !bucket.is_empty())
            .ok_or_else(|| {
                PluginError::service_unavailable(format!("service '{id}' is not registered"))
            })?;
        let entry = bucket
            .iter()
            .find(|entry| entry.provider.is_none())
            .or_else(|| bucket.first())
            .ok_or_else(|| {
                PluginError::service_unavailable(format!("service '{id}' has no live provider"))
            })?;

        self.check_visible(requester, entry)?;
        self.check_capabilities(requester, entry)?;
        if !entry.slot.is_alive() {
            return Err(PluginError::service_unavailable(format!(
                "service '{id}' has been revoked"
            )));
        }
        Ok(())
    }

    pub fn resolve_as<T: Send + Sync + 'static>(
        &self,
        requester: Option<&PluginId>,
        id: &ServiceId,
    ) -> PluginResult<ServiceRef<T>> {
        let resolved = self.resolve_for(requester, id, TypeId::of::<T>())?;
        let value = ServiceBinding::downcast::<T>(&resolved.raw).ok_or_else(|| {
            PluginError::service_unavailable(format!("service '{id}' type mismatch"))
        })?;
        Ok(ServiceRef::new(resolved.slot, value))
    }

    fn check_visible(&self, requester: Option<&PluginId>, entry: &ServiceEntry) -> PluginResult<()> {
        match entry.descriptor.visibility {
            ServiceVisibility::Public => Ok(()),
            ServiceVisibility::HostOnly => match requester {
                None => Ok(()),
                Some(plugin) => Err(PluginError::permission_denied(format!(
                    "plugin '{plugin}' may not resolve host-only service '{}'",
                    entry.descriptor.id
                ))),
            },
            ServiceVisibility::DeclaredDependentsOnly => match (requester, &entry.provider) {
                (None, _) => Ok(()),
                (Some(_), None) => Ok(()),
                (Some(requester), Some(provider)) => {
                    let declared = self.declared.read().ok();
                    let allowed = declared
                        .as_ref()
                        .and_then(|map| map.get(requester))
                        .map(|set| set.contains(provider))
                        .unwrap_or(false);
                    if allowed {
                        Ok(())
                    } else {
                        Err(PluginError::permission_denied(format!(
                            "plugin '{requester}' must declare '{provider}' as a dependency to resolve '{}'",
                            entry.descriptor.id
                        )))
                    }
                }
            },
        }
    }

    fn check_capabilities(
        &self,
        requester: Option<&PluginId>,
        entry: &ServiceEntry,
    ) -> PluginResult<()> {
        if entry.descriptor.requires.is_empty() {
            return Ok(());
        }
        let Some(plugin) = requester else {
            return Ok(()); // 宿主自身不受限
        };
        let granted = self
            .granted
            .read()
            .map_err(|e| PluginError::io(format!("grant table poisoned: {e}")))?;
        let owned = granted.get(plugin);
        for required in &entry.descriptor.requires {
            if !owned.map(|set| set.contains(required)).unwrap_or(false) {
                return Err(PluginError::permission_denied(format!(
                    "plugin '{plugin}' lacks capability '{required}' required by service '{}'",
                    entry.descriptor.id
                )));
            }
        }
        Ok(())
    }

    // —— 诊断 ——

    pub fn has(&self, id: &ServiceId) -> bool {
        self.entries
            .read()
            .map(|lock| lock.get(id).map(|b| !b.is_empty()).unwrap_or(false))
            .unwrap_or(false)
    }

    pub fn inventory(&self) -> Vec<ServiceInventoryItem> {
        let Ok(lock) = self.entries.read() else {
            return Vec::new();
        };
        let mut items: Vec<ServiceInventoryItem> = lock
            .iter()
            .flat_map(|(id, bucket)| {
                bucket.iter().map(move |entry| ServiceInventoryItem {
                    id: id.clone(),
                    provider: entry.provider.as_ref().map(|p| p.as_str().to_string()),
                    version: entry.descriptor.version.clone(),
                    visibility: entry.descriptor.visibility,
                    requires: entry.descriptor.requires.clone(),
                    alive: entry.slot.is_alive(),
                })
            })
            .collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));
        items
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInventoryItem {
    pub id: ServiceId,
    pub provider: Option<String>,
    pub version: Version,
    pub visibility: ServiceVisibility,
    pub requires: Vec<Capability>,
    pub alive: bool,
}

impl ServiceResolver for ServiceContainer {
    fn resolve(
        &self,
        requester: &PluginId,
        id: &ServiceId,
        type_id: TypeId,
    ) -> PluginResult<ResolvedService> {
        self.resolve_for(Some(requester), id, type_id)
    }
}