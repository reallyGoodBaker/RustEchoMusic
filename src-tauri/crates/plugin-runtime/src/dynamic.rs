use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use plugin_sdk::abi::{
    AbiResponse, AudioProcessorAbi, ByteSlice, HostVTable, OutBuffer, PluginAbi,
};
use plugin_sdk::{
    ActivationContext, Contribution, DeactivateReason, EventPattern, HealthStatus, HostEvent,
    InvokeContext, PluginDescriptor, PluginError, PluginId, PluginResult, Plugin, ServiceId,
    ServiceSlot, TeardownContext,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::container::ServiceContainer;
use crate::loader::LoadedLibrary;

const fn encode_handle(raw: u64) -> usize {
    ((raw << 1) | 1) as usize
}

pub trait JsonService: Send + Sync {
    fn descriptor(&self) -> plugin_sdk::ServiceDescriptor;
    fn call(&self, method: &str, args: &Value) -> PluginResult<Value>;
}

#[derive(Default)]
pub struct JsonServiceRegistry {
    inner: RwLock<HashMap<ServiceId, Arc<dyn JsonService>>>,
}

impl JsonServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, service: Arc<dyn JsonService>) {
        if let Ok(mut lock) = self.inner.write() {
            lock.insert(service.descriptor().id.clone(), service);
        }
    }

    pub fn remove_provider(&self, provider: &PluginId) -> Vec<ServiceId> {
        let mut removed = Vec::new();
        if let Ok(mut lock) = self.inner.write() {
            lock.retain(|id, service| {
                let is_provider = service
                    .descriptor()
                    .id
                    .as_str()
                    .starts_with(provider.as_str());
                if is_provider {
                    removed.push(id.clone());
                    false
                } else {
                    true
                }
            });
        }
        removed
    }

    pub fn has(&self, id: &ServiceId) -> bool {
        self.inner
            .read()
            .map(|lock| lock.contains_key(id))
            .unwrap_or(false)
    }

    pub fn call(&self, id: &ServiceId, method: &str, args: &Value) -> PluginResult<Value> {
        let service = self
            .inner
            .read()
            .ok()
            .and_then(|lock| lock.get(id).cloned())
            .ok_or_else(|| {
                PluginError::service_unavailable(format!("service '{id}' has no JSON interface"))
            })?;
        service.call(method, args)
    }
}

struct HandleEntry {
    id: ServiceId,
    slot: Arc<ServiceSlot>,
    container: Arc<ServiceContainer>,
    json: Arc<JsonServiceRegistry>,
    plugin: PluginId,
}

static HANDLES: LazyLock<Mutex<HashMap<u64, Arc<HandleEntry>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

fn invoke_handle(handle: u64, method: &str, args: &Value) -> PluginResult<Value> {
    let entry = {
        let lock = HANDLES
            .lock()
            .map_err(|e| PluginError::io(format!("service handle table poisoned: {e}")))?;
        lock.get(&handle).cloned()
    }
    .ok_or_else(|| {
        PluginError::service_unavailable(format!("stale service handle #{handle}"))
    })?;

    if !entry.slot.is_alive() {
        return Err(PluginError::service_unavailable(format!(
            "service '{}' has been revoked",
            entry.id
        )));
    }
    // 每次调用都重新鉴权：授权可能已在停用 / 卸载时被撤销。
    entry.container.authorize(Some(&entry.plugin), &entry.id)?;
    entry.json.call(&entry.id, method, args)
}

struct HostBridge {
    container: Arc<ServiceContainer>,
    json: Arc<JsonServiceRegistry>,
    plugin: PluginId,
    issued: Mutex<Vec<u64>>,
}

impl HostBridge {
    fn new(
        container: Arc<ServiceContainer>,
        json: Arc<JsonServiceRegistry>,
        plugin: PluginId,
    ) -> Self {
        Self {
            container,
            json,
            plugin,
            issued: Mutex::new(Vec::new()),
        }
    }

    fn resolve(&self, args: &Value) -> PluginResult<Value> {
        let raw_id = args
            .get("serviceId")
            .and_then(Value::as_str)
            .ok_or_else(|| PluginError::invalid_argument("resolve requires 'serviceId'"))?;
        let id = ServiceId::new(raw_id).map_err(|error| {
            PluginError::invalid_argument(format!("invalid service id '{raw_id}': {error}"))
        })?;
        if !self.json.has(&id) {
            return Err(PluginError::service_unavailable(format!(
                "service '{id}' is not exposed over the JSON channel"
            )));
        }
        // 能力校验走与内置插件完全相同的路径。
        self.container.authorize(Some(&self.plugin), &id)?;

        let raw = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        let entry = Arc::new(HandleEntry {
            id,
            slot: Arc::new(ServiceSlot::new(None)),
            container: Arc::clone(&self.container),
            json: Arc::clone(&self.json),
            plugin: self.plugin.clone(),
        });
        {
            let Ok(mut table) = HANDLES.lock() else {
                return Err(PluginError::io("service handle table poisoned"));
            };
            table.insert(raw, entry);
        }
        if let Ok(mut issued) = self.issued.lock() {
            issued.push(raw);
        }
        // 交给插件的是**编码后**的句柄（奇数），宿主据此与根上下文区分。
        Ok(serde_json::json!({ "handle": encode_handle(raw) }))
    }

    fn release(&self, args: &Value) {
        // 插件传回的是编码后的句柄；先解码再查表。
        let Some(encoded) = args.get("handle").and_then(Value::as_u64) else {
            return;
        };
        let raw = encoded >> 1;
        let entry = {
            let Ok(mut table) = HANDLES.lock() else {
                return;
            };
            table.remove(&raw)
        };
        if let Some(entry) = entry {
            entry.slot.revoke();
        }
        if let Ok(mut issued) = self.issued.lock() {
            issued.retain(|id| *id != raw);
        }
    }

    fn revoke_all(&self) {
        let issued: Vec<u64> = match self.issued.lock() {
            Ok(mut lock) => std::mem::take(&mut *lock),
            Err(_) => return,
        };
        let Ok(mut table) = HANDLES.lock() else {
            return;
        };
        for raw in issued {
            if let Some(entry) = table.remove(&raw) {
                entry.slot.revoke();
            }
        }
    }
}

// —— 宿主回调表的具体实现 ——

static HOST_VTABLE: HostVTable = HostVTable {
    size: std::mem::size_of::<HostVTable>(),
    invoke_service: host_invoke_service,
    free: host_free,
    log: host_log,
};

unsafe extern "C" fn host_invoke_service(
    service: *mut c_void,
    method: ByteSlice,
    args: ByteSlice,
    out: *mut OutBuffer,
) -> i32 {
    if service.is_null() || out.is_null() {
        return -1;
    }
    let method = match std::str::from_utf8(method.as_slice()) {
        Ok(text) => text,
        Err(_) => return -2,
    };
    let args: Value = if args.is_empty() {
        Value::Null
    } else {
        match serde_json::from_slice(args.as_slice()) {
            Ok(value) => value,
            Err(_) => return -3,
        }
    };

    let address = service as usize;
    // **必须在解引用之前**判别：奇数 = 服务句柄，偶数 = 宿主根上下文指针。
    // 若顺序颠倒，把句柄当 `HostBridge` 解引用就是立即 UB。
    let result = if address & 1 == 1 {
        invoke_handle((address >> 1) as u64, method, &args)
    } else {
        let bridge = &*(service as *const HostBridge);
        match method {
            "resolve" => bridge.resolve(&args),
            "release" => {
                bridge.release(&args);
                Ok(Value::Null)
            }
            _ => Err(PluginError::invalid_argument(format!(
                "unknown host method '{method}'"
            ))),
        }
    };

    write_out(out, &result);
    0
}

unsafe extern "C" fn host_free(buffer: *mut OutBuffer) {
    if buffer.is_null() {
        return;
    }
    let OutBuffer { ptr, len, cap } = *buffer;
    if !ptr.is_null() && cap > 0 {
        drop(Vec::from_raw_parts(ptr, len, cap));
    }
    (*buffer).ptr = std::ptr::null_mut();
    (*buffer).len = 0;
    (*buffer).cap = 0;
}

unsafe extern "C" fn host_log(level: i32, message: ByteSlice) {
    let level = match level {
        0 => "ERROR",
        1 => "WARN",
        2 => "INFO",
        _ => "DEBUG",
    };
    eprintln!("[plugin] {level} {}", String::from_utf8_lossy(message.as_slice()));
}

unsafe fn write_out(out: *mut OutBuffer, result: &PluginResult<Value>) {
    let payload = match result {
        Ok(value) => AbiResponse::success(Some(value.clone())),
        Err(error) => AbiResponse::failure(error),
    };
    let mut bytes = serde_json::to_vec(&payload).unwrap_or_else(|_| Vec::new());
    let ptr = bytes.as_mut_ptr();
    let len = bytes.len();
    let cap = bytes.capacity();
    std::mem::forget(bytes); // 所有权交给插件，由 host_free 回收
    (*out).ptr = ptr;
    (*out).len = len;
    (*out).cap = cap;
}

// —— 插件提供的服务的宿主侧代理 ——

#[derive(Clone, Copy)]
struct InstanceHandle(*mut c_void);

// SAFETY: 见类型文档；句柄只在 `DynamicPlugin` 存活期间使用。
unsafe impl Send for InstanceHandle {}
unsafe impl Sync for InstanceHandle {}

impl InstanceHandle {
    fn is_null(self) -> bool {
        self.0.is_null()
    }

    fn get(self) -> *mut c_void {
        self.0
    }
}

pub struct RemoteServiceProxy {
    descriptor: plugin_sdk::ServiceDescriptor,
    caller: Arc<dyn Fn(&str, &Value) -> PluginResult<Value> + Send + Sync>,
}

impl JsonService for RemoteServiceProxy {
    fn descriptor(&self) -> plugin_sdk::ServiceDescriptor {
        self.descriptor.clone()
    }
    fn call(&self, method: &str, args: &Value) -> PluginResult<Value> {
        (self.caller)(method, args)
    }
}

pub fn as_json_service(binding: &plugin_sdk::ServiceBinding) -> Option<Arc<dyn JsonService>> {
    let proxy = plugin_sdk::ServiceBinding::downcast::<RemoteServiceProxy>(binding.service())?;
    Some(proxy as Arc<dyn JsonService>)
}

// —— 动态插件实例 ——

pub struct DynamicPlugin {
    descriptor: PluginDescriptor,
    lib: Arc<LoadedLibrary>,
    bridge: Box<HostBridge>,
    handle: *mut c_void,
    processor: Option<AudioProcessorAbi>,
}

// SAFETY: `handle` 只在本实例的 `unsafe` 调用点使用，且 `bridge` 地址稳定；
// `lib` 的 `Arc` 保证动态库在实例存活期间不会被卸载。
unsafe impl Send for DynamicPlugin {}
unsafe impl Sync for DynamicPlugin {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationPayload {
    #[serde(default)]
    pub contributions: Vec<Contribution>,
    #[serde(default)]
    pub service_ids: Vec<String>,
    #[serde(default)]
    pub subscriptions: Vec<EventPattern>,
}

impl DynamicPlugin {
    pub fn create(
        lib: Arc<LoadedLibrary>,
        container: Arc<ServiceContainer>,
        json: Arc<JsonServiceRegistry>,
        plugin: PluginId,
    ) -> PluginResult<Self> {
        let abi = unsafe { lib.abi() };
        let bridge = Box::new(HostBridge::new(container, json, plugin));
        let mut handle: *mut c_void = std::ptr::null_mut();
        let code = unsafe {
            (abi.create)(
                &HOST_VTABLE,
                &*bridge as *const HostBridge as *mut c_void,
                &mut handle,
            )
        };
        if code != 0 || handle.is_null() {
            return Err(PluginError::plugin(format!(
                "plugin '{}' failed to create an instance (code {code})",
                lib.id()
            )));
        }

        let descriptor = Self::read_descriptor(lib.id(), handle, abi)?;
        let processor = if abi.audio_processor.is_null() {
            None
        } else {
            Some(unsafe { *abi.audio_processor })
        };

        Ok(Self {
            descriptor,
            lib,
            bridge,
            handle,
            processor,
        })
    }

    fn read_descriptor(
        id: &PluginId,
        handle: *mut c_void,
        abi: &PluginAbi,
    ) -> PluginResult<PluginDescriptor> {
        let value = unsafe { invoke_raw(handle, "descriptor", &Value::Null, abi) }?;
        let descriptor: PluginDescriptor = serde_json::from_value(value.unwrap_or(Value::Null))
            .map_err(|error| {
                PluginError::incompatible(format!(
                    "plugin '{id}' returned an unreadable descriptor: {error}"
                ))
            })?;
        if &descriptor.id != id {
            return Err(PluginError::incompatible(format!(
                "plugin '{id}' self-reports as '{}'; refusing to load",
                descriptor.id
            )));
        }
        Ok(descriptor)
    }

    pub fn audio_processor(&self) -> Option<&AudioProcessorAbi> {
        self.processor.as_ref()
    }

    pub fn handle(&self) -> *mut c_void {
        self.handle
    }

    fn call(&self, op: &str, args: &Value) -> PluginResult<Option<Value>> {
        let abi = unsafe { self.lib.abi() };
        unsafe { invoke_raw(self.handle, op, args, abi) }
    }

    fn invoke_self(&self, op: &str, args: &Value) -> PluginResult<Value> {
        self.call(op, args).map(|value| value.unwrap_or(Value::Null))
    }
}

impl Drop for DynamicPlugin {
    fn drop(&mut self) {
        self.bridge.revoke_all();
        if !self.handle.is_null() {
            let abi = unsafe { self.lib.abi() };
            unsafe { (abi.destroy)(self.handle) };
            self.handle = std::ptr::null_mut();
        }
    }
}

impl Plugin for DynamicPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn activate(&self, ctx: &mut ActivationContext) -> PluginResult<()> {
        let payload: ActivationPayload =
            serde_json::from_value(self.invoke_self(plugin_sdk::abi::ops::ACTIVATE, &Value::Null)?)
                .map_err(|error| {
                    PluginError::plugin(format!("plugin returned a malformed activation payload: {error}"))
                })?;

        for contribution in payload.contributions {
            ctx.contribute(contribution);
        }
        for pattern in payload.subscriptions {
            ctx.subscribe(pattern);
        }

        // 插件提供的服务：注册一个把调用转发回插件的宿主侧代理。
        for raw_id in payload.service_ids {
            let service_id = ServiceId::new(raw_id.clone()).map_err(|error| {
                PluginError::invalid_argument(format!(
                    "plugin '{}' published an invalid service id '{raw_id}': {error}",
                    self.descriptor.id
                ))
            })?;
            let caller = {
                let plugin_id = self.descriptor.id.clone();
                let service = service_id.clone();
                let lib = Arc::clone(&self.lib);
                let handle = InstanceHandle(self.handle);
                Arc::new(move |method: &str, args: &Value| -> PluginResult<Value> {
                    if handle.is_null() {
                        return Err(PluginError::service_unavailable(format!(
                            "service '{service}' from '{plugin_id}' is gone"
                        )));
                    }
                    let abi = unsafe { lib.abi() };
                    let request = serde_json::json!({
                        "serviceId": service.as_str(),
                        "method": method,
                        "args": args,
                    });
                    unsafe { invoke_raw(handle.get(), "service", &request, abi) }
                        .map(|value| value.unwrap_or(Value::Null))
                }) as Arc<dyn Fn(&str, &Value) -> PluginResult<Value> + Send + Sync>
            };

            ctx.provide(plugin_sdk::ServiceBinding::new(
                plugin_sdk::ServiceDescriptor::new(
                    service_id.clone(),
                    self.descriptor.version.clone(),
                )
                .with_summary(format!("provided by {}", self.descriptor.id)),
                // 代理本身就是一个 Rust 对象，注册到容器时按 `RemoteServiceProxy` 类型擦除，
                // 由 `JsonServiceRegistry` 侧按 `service_id` 分派。
                Arc::new(RemoteServiceProxy {
                    descriptor: plugin_sdk::ServiceDescriptor::new(
                        service_id.clone(),
                        self.descriptor.version.clone(),
                    ),
                    caller,
                }),
            ));
        }
        Ok(())
    }

    fn deactivate(&self, ctx: &TeardownContext) -> PluginResult<()> {
        self.bridge.revoke_all();
        let args = serde_json::json!({ "reason": deactivate_reason_str(ctx.reason()) });
        self.invoke_self(plugin_sdk::abi::ops::DEACTIVATE, &args)?;
        Ok(())
    }

    fn on_event(&self, event: &HostEvent, _ctx: &InvokeContext) -> PluginResult<()> {
        let payload = serde_json::to_value(event)?;
        self.invoke_self(plugin_sdk::abi::ops::EVENT, &payload)?;
        Ok(())
    }

    fn execute(
        &self,
        command: &plugin_sdk::CommandId,
        args: Value,
        _ctx: &InvokeContext,
    ) -> PluginResult<Value> {
        let request = serde_json::json!({ "command": command.as_str(), "args": args });
        self.invoke_self(plugin_sdk::abi::ops::COMMAND, &request)
    }

    fn health(&self) -> HealthStatus {
        match self.call(plugin_sdk::abi::ops::HEALTH, &Value::Null) {
            Ok(Some(value)) => serde_json::from_value(value).unwrap_or(HealthStatus::Degraded),
            Ok(None) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Degraded,
        }
    }

    fn audio_processor(&self) -> Option<plugin_sdk::AudioProcessorHandle> {
        self.processor.map(|abi| plugin_sdk::AudioProcessorHandle {
            abi,
            handle: self.handle,
        })
    }
}

pub fn deactivate_reason_str(reason: DeactivateReason) -> &'static str {
    match reason {
        DeactivateReason::UserDisabled => "userDisabled",
        DeactivateReason::Uninstall => "uninstall",
        DeactivateReason::Reload => "reload",
        DeactivateReason::CircuitBroken => "circuitBroken",
        DeactivateReason::DependencyRemoved => "dependencyRemoved",
        DeactivateReason::Shutdown => "shutdown",
    }
}

pub(crate) unsafe fn invoke_raw(
    handle: *mut c_void,
    op: &str,
    args: &Value,
    abi: &PluginAbi,
) -> PluginResult<Option<Value>> {
    let op_bytes = op.as_bytes();
    let args_bytes = serde_json::to_vec(args)?;

    let mut out = OutBuffer::default();
    let code = (abi.invoke)(
        handle,
        ByteSlice::from_slice(op_bytes),
        ByteSlice::from_slice(&args_bytes),
        &mut out,
    );

    let decoded = if out.is_empty() {
        None
    } else {
        let slice = out.as_slice();
        serde_json::from_slice::<AbiResponse>(slice).ok()
    };
    // 无论成功失败，缓冲区都交给插件自己的 free 释放。
    if !out.is_empty() {
        (abi.free)(&mut out);
    }

    if code != 0 {
        return Err(PluginError::plugin(format!(
            "abi invoke '{op}' failed with code {code}"
        )));
    }
    match decoded {
        Some(response) => response.into_result(),
        None => Err(PluginError::io(format!(
            "plugin returned an unreadable response for '{op}'"
        ))),
    }
}