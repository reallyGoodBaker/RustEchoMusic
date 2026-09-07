use std::ffi::c_void;

pub use serde_json;
use serde_json::{Value, json};

use crate::abi::{AbiResponse, ByteSlice, HostVTable, OutBuffer};
use crate::PluginError;

pub type Json = Value;
pub type GuestResult = crate::PluginResult<Value>;

pub struct Host {
    vtable: *const HostVTable,
    ctx: *mut c_void,
}

unsafe impl Send for Host {}

impl Host {
    pub fn new(vtable: *const HostVTable, ctx: *mut c_void) -> Self {
        Self { vtable, ctx }
    }

    pub fn resolve(&self, service_id: &str) -> std::result::Result<u64, PluginError> {
        let resp = self.invoke_raw(self.ctx, "resolve", &json!({ "serviceId": service_id }))?;
        resp.get("handle")
            .and_then(Value::as_u64)
            .ok_or_else(|| PluginError::plugin("resolve 未返回 handle"))
    }

    pub fn call(&self, handle: u64, method: &str, args: &Value) -> std::result::Result<Value, PluginError> {
        self.invoke_raw(handle as *mut c_void, method, args)
    }

    pub fn release(&self, handle: u64) {
        let _ = self.invoke_raw(self.ctx, "release", &json!({ "handle": handle }));
    }

    fn invoke_raw(&self, service: *mut c_void, method: &str, args: &Value) -> std::result::Result<Value, PluginError> {
        let vtable = unsafe { &*self.vtable };
        let method_bytes = method.as_bytes();
        let args_bytes = serde_json::to_vec(args).map_err(|e| PluginError::plugin(e.to_string()))?;
        let mut out = OutBuffer::default();
        let code = unsafe {
            (vtable.invoke_service)(
                service,
                ByteSlice { ptr: method_bytes.as_ptr(), len: method_bytes.len() },
                ByteSlice { ptr: args_bytes.as_ptr(), len: args_bytes.len() },
                &mut out,
            )
        };
        if code != 0 {
            return Err(PluginError::plugin(format!("invoke_service 返回非零码 {code}")));
        }
        let bytes = {
            let slice = unsafe { std::slice::from_raw_parts(out.ptr, out.len) };
            let copied = slice.to_vec();
            unsafe { (vtable.free)(&mut out); }
            copied
        };
        let response: AbiResponse = serde_json::from_slice(&bytes)
            .map_err(|e| PluginError::plugin(format!("无法解析 AbiResponse: {e}")))?;
        response.into_result().map(|v| v.unwrap_or(Value::Null))
    }

    pub fn log(&self, level: i32, message: &str) {
        let vtable = unsafe { &*self.vtable };
        unsafe { (vtable.log)(level, ByteSlice::from_slice(message.as_bytes())) };
    }
}

pub trait GuestPlugin: Default {
    fn descriptor(&self) -> Value;

    fn activate(&self, _host: &Host) -> std::result::Result<Value, PluginError> {
        Ok(Value::Null)
    }

    fn command(&self, _host: &Host, _command: &str, _args: &Value) -> std::result::Result<Value, PluginError>;

    fn health(&self) -> Value {
        json!("healthy")
    }

    fn deactivate(&self) -> std::result::Result<(), PluginError> {
        Ok(())
    }

    fn on_event(&self, _host: &Host, _event: &Value) -> std::result::Result<(), PluginError> {
        Ok(())
    }
}

pub fn write_json(out: *mut OutBuffer, payload: AbiResponse) {
    let mut bytes = serde_json::to_vec(&payload).unwrap_or_default();
    let ptr = bytes.as_mut_ptr();
    let len = bytes.len();
    let cap = bytes.capacity();
    std::mem::forget(bytes);
    unsafe {
        (*out).ptr = ptr;
        (*out).len = len;
        (*out).cap = cap;
    }
}

pub fn free_out_buffer(buf: *mut OutBuffer) {
    if buf.is_null() {
        return;
    }
    let OutBuffer { ptr, len, cap } = unsafe { *buf };
    if !ptr.is_null() && cap > 0 {
        drop(unsafe { Vec::from_raw_parts(ptr, len, cap) });
    }
    unsafe {
        (*buf).ptr = std::ptr::null_mut();
        (*buf).len = 0;
        (*buf).cap = 0;
    }
}

#[macro_export]
macro_rules! guest_plugin {
    ($ty:ty) => {
        $crate::guest_plugin!(@inner $ty, ::std::ptr::null::<$crate::abi::AudioProcessorAbi>());
    };
    ($ty:ty, audio_processor: $ap:expr) => {
        $crate::guest_plugin!(@inner $ty, $ap);
    };
    (@inner $ty:ty, $audio_processor:expr) => {
        static mut __GUEST_HOST: Option<$crate::guest::Host> = None;
        static mut __GUEST_INSTANCE: Option<*mut $ty> = None;

        unsafe extern "C" fn __guest_create(
            host: *const $crate::abi::HostVTable,
            host_ctx: *mut ::std::ffi::c_void,
            out: *mut *mut ::std::ffi::c_void,
        ) -> i32 {
            __GUEST_HOST = Some($crate::guest::Host::new(host, host_ctx));
            let plugin = ::std::boxed::Box::into_raw(::std::boxed::Box::new(<$ty>::default()));
            __GUEST_INSTANCE = Some(plugin);
            *out = plugin as *mut ::std::ffi::c_void;
            0
        }

        unsafe extern "C" fn __guest_destroy(handle: *mut ::std::ffi::c_void) {
            if !handle.is_null() {
                drop(::std::boxed::Box::from_raw(handle as *mut $ty));
            }
            __GUEST_INSTANCE = None;
            __GUEST_HOST = None;
        }

        unsafe extern "C" fn __guest_free(buf: *mut $crate::abi::OutBuffer) {
            $crate::guest::free_out_buffer(buf);
        }

        unsafe extern "C" fn __guest_invoke(
            handle: *mut ::std::ffi::c_void,
            op: $crate::abi::ByteSlice,
            args: $crate::abi::ByteSlice,
            out: *mut $crate::abi::OutBuffer,
        ) -> i32 {
            let op = match ::std::str::from_utf8(op.as_slice()) {
                Ok(s) => s,
                Err(_) => return -2,
            };
            let args: $crate::guest::Json = if args.is_empty() {
                $crate::guest::Json::Null
            } else {
                match $crate::guest::serde_json::from_slice(args.as_slice()) {
                    Ok(v) => v,
                    Err(_) => return -3,
                }
            };
            let plugin = &*(handle as *const $ty);
            let host = match &*::std::ptr::addr_of!(__GUEST_HOST) {
                Some(h) => h,
                None => return -1,
            };
            let result: $crate::guest::GuestResult = match op {
                "descriptor" => Ok(plugin.descriptor()),
                "activate" => plugin.activate(host),
                "command" => {
                    let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                    plugin.command(host, command, &args)
                }
                "health" => Ok(plugin.health()),
                "deactivate" => plugin.deactivate().map(|_| $crate::guest::Json::Null),
                "event" => plugin.on_event(host, &args).map(|_| $crate::guest::Json::Null),
                _ => return -1,
            };
            match result {
                Ok(v) => {
                    $crate::guest::write_json(out, $crate::abi::AbiResponse::success(Some(v)));
                    0
                }
                Err(e) => {
                    $crate::guest::write_json(out, $crate::abi::AbiResponse::failure(&e));
                    0
                }
            }
        }

        #[repr(transparent)]
        struct __SyncAbi($crate::abi::PluginAbi);
        unsafe impl ::std::marker::Sync for __SyncAbi {}

        #[no_mangle]
        pub static REM_PLUGIN_ABI: __SyncAbi = __SyncAbi($crate::abi::PluginAbi {
            abi_version: $crate::ABI_VERSION,
            size: ::std::mem::size_of::<$crate::abi::PluginAbi>(),
            create: __guest_create,
            destroy: __guest_destroy,
            invoke: __guest_invoke,
            free: __guest_free,
            audio_processor: $audio_processor,
        });
    };
}
