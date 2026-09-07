use std::any::Any;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::error::{PluginError, PluginResult};
use super::ids::{Capability, PluginId, ServiceId, Version};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceVisibility {
    Public,
    DeclaredDependentsOnly,
    HostOnly,
}

impl Default for ServiceVisibility {
    fn default() -> Self {
        Self::Public
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDescriptor {
    pub id: ServiceId,
    pub version: Version,
    pub requires: Vec<Capability>,
    pub visibility: ServiceVisibility,
    pub summary: String,
}

impl ServiceDescriptor {
    pub fn new(id: ServiceId, version: Version) -> Self {
        Self {
            id,
            version,
            requires: Vec::new(),
            visibility: ServiceVisibility::Public,
            summary: String::new(),
        }
    }

    pub fn requiring(mut self, capability: Capability) -> Self {
        self.requires.push(capability);
        self
    }

    pub fn with_visibility(mut self, visibility: ServiceVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }
}

pub type AnyService = dyn Any + Send + Sync;

#[derive(Debug)]
pub struct ServiceSlot {
    alive: AtomicBool,
    owner: Option<PluginId>,
}

impl ServiceSlot {
    pub fn new(owner: Option<PluginId>) -> Self {
        Self {
            alive: AtomicBool::new(true),
            owner,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    pub fn revoke(&self) {
        self.alive.store(false, Ordering::Release);
    }

    pub fn owner(&self) -> Option<&PluginId> {
        self.owner.as_ref()
    }
}

impl Default for ServiceSlot {
    fn default() -> Self {
        Self::new(None)
    }
}

pub struct ServiceRef<T: ?Sized> {
    slot: Arc<ServiceSlot>,
    value: Arc<T>,
}

impl<T: ?Sized> ServiceRef<T> {
    pub fn new(slot: Arc<ServiceSlot>, value: Arc<T>) -> Self {
        Self { slot, value }
    }

    pub fn get(&self) -> PluginResult<&T> {
        if self.slot.is_alive() {
            Ok(&self.value)
        } else {
            Err(PluginError::service_unavailable(format!(
                "service owned by '{}' has been unloaded",
                owner_label(self.slot.owner())
            )))
        }
    }

    pub fn try_value(&self) -> Option<Arc<T>> {
        self.slot.is_alive().then(|| Arc::clone(&self.value))
    }

    pub fn is_alive(&self) -> bool {
        self.slot.is_alive()
    }

    pub fn slot(&self) -> &Arc<ServiceSlot> {
        &self.slot
    }
}

impl<T: ?Sized> Clone for ServiceRef<T> {
    fn clone(&self) -> Self {
        Self {
            slot: Arc::clone(&self.slot),
            value: Arc::clone(&self.value),
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for ServiceRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceRef")
            .field("alive", &self.slot.is_alive())
            .field("value", &self.value)
            .finish()
    }
}

fn owner_label(owner: Option<&PluginId>) -> String {
    owner.map(|p| p.as_str().to_string()).unwrap_or_else(|| "host".to_string())
}

pub struct ServiceBinding {
    descriptor: ServiceDescriptor,
    service: Arc<AnyService>,
    type_id: std::any::TypeId,
}

impl ServiceBinding {
    pub fn new<T: Send + Sync + 'static>(
        descriptor: ServiceDescriptor,
        service: Arc<T>,
    ) -> Self {
        Self {
            descriptor,
            service,
            type_id: std::any::TypeId::of::<T>(),
        }
    }

    pub fn descriptor(&self) -> &ServiceDescriptor {
        &self.descriptor
    }

    pub fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    pub fn service(&self) -> &Arc<AnyService> {
        &self.service
    }

    pub fn into_parts(self) -> (ServiceDescriptor, Arc<AnyService>, std::any::TypeId) {
        (self.descriptor, self.service, self.type_id)
    }

    pub fn downcast<T: Send + Sync + 'static>(service: &Arc<AnyService>) -> Option<Arc<T>> {
        Arc::clone(service).downcast::<T>().ok()
    }
}

impl fmt::Debug for ServiceBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceBinding")
            .field("id", &self.descriptor.id)
            .field("version", &self.descriptor.version)
            .finish()
    }
}

pub struct ServiceHandle<T: ?Sized>(pub Arc<T>);

impl<T: ?Sized> ServiceHandle<T> {
    pub fn new(value: Arc<T>) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T: ?Sized> Clone for ServiceHandle<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T: ?Sized> std::ops::Deref for ServiceHandle<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Arc<T> {
        &self.0
    }
}

impl<T: ?Sized> fmt::Debug for ServiceHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceHandle").finish()
    }
}