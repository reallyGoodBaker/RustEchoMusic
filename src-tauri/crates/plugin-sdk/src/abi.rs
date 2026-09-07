use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ByteSlice {
    pub ptr: *const u8,
    pub len: usize,
}

impl ByteSlice {
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self {
            ptr: if bytes.is_empty() {
                std::ptr::null()
            } else {
                bytes.as_ptr()
            },
            len: bytes.len(),
        }
    }

    pub unsafe fn as_slice<'a>(&self) -> &'a [u8] {
        if self.ptr.is_null() || self.len == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0 || self.ptr.is_null()
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OutBuffer {
    pub ptr: *mut u8,
    pub len: usize,
    pub cap: usize,
}

impl Default for OutBuffer {
    fn default() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }
}

impl OutBuffer {
    pub fn is_empty(&self) -> bool {
        self.ptr.is_null() || self.len == 0
    }

    pub unsafe fn as_slice<'a>(&self) -> &'a [u8] {
        if self.is_empty() {
            &[]
        } else {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

#[repr(C)]
pub struct HostVTable {
    pub size: usize,
    pub invoke_service: unsafe extern "C" fn(
        service: *mut c_void,
        method: ByteSlice,
        args: ByteSlice,
        out: *mut OutBuffer,
    ) -> i32,
    pub free: unsafe extern "C" fn(*mut OutBuffer),
    pub log: unsafe extern "C" fn(level: i32, message: ByteSlice),
}

#[repr(C)]
pub struct PluginAbi {
    pub abi_version: u32,
    pub size: usize,

    pub create: unsafe extern "C" fn(
        host: *const HostVTable,
        host_ctx: *mut c_void,
        out: *mut *mut c_void,
    ) -> i32,

    pub destroy: unsafe extern "C" fn(handle: *mut c_void),

    pub invoke: unsafe extern "C" fn(
        handle: *mut c_void,
        op: ByteSlice,
        args: ByteSlice,
        out: *mut OutBuffer,
    ) -> i32,

    pub free: unsafe extern "C" fn(*mut OutBuffer),

    pub audio_processor: *const AudioProcessorAbi,
}

impl PluginAbi {
    pub fn is_compatible(&self) -> Result<(), super::PluginError> {
        if self.abi_version != super::ABI_VERSION {
            return Err(super::PluginError::incompatible(format!(
                "plugin abi {} != host abi {}",
                self.abi_version,
                super::ABI_VERSION
            )));
        }
        let expected = std::mem::size_of::<PluginAbi>();
        if self.size != expected {
            return Err(super::PluginError::incompatible(format!(
                "plugin abi struct size {} != host size {}; rebuild the plugin",
                self.size, expected
            )));
        }
        Ok(())
    }
}

pub mod ops {
    pub const DESCRIPTOR: &str = "descriptor";
    pub const ACTIVATE: &str = "activate";
    pub const DEACTIVATE: &str = "deactivate";
    pub const EVENT: &str = "event";
    pub const COMMAND: &str = "command";
    pub const HEALTH: &str = "health";
    pub const RELEASE_SERVICE: &str = "releaseService";
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiResponse {
    pub ok: bool,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<AbiErrorBody>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbiErrorBody {
    pub code: String,
    pub message: String,
}

impl AbiResponse {
    pub fn success(value: Option<serde_json::Value>) -> Self {
        Self {
            ok: true,
            value,
            error: None,
        }
    }

    pub fn failure(error: &super::PluginError) -> Self {
        Self {
            ok: false,
            value: None,
            error: Some(AbiErrorBody {
                // 走 serde 的 `rename_all`，保证与 `ErrorCode` 的线上表示一致
                // （SCREAMING_SNAKE_CASE），而不是 Rust 的 `Debug` 变体名。
                code: serde_json::to_value(error.code())
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_else(|| "PLUGIN".to_string()),
                message: error.message().to_string(),
            }),
        }
    }

    pub fn into_result(self) -> super::PluginResult<Option<serde_json::Value>> {
        if self.ok {
            Ok(self.value)
        } else {
            let body = self
                .error
                .unwrap_or_else(|| AbiErrorBody {
                    code: "PLUGIN".into(),
                    message: "unknown plugin error".into(),
                });
            let code = serde_json::from_value::<super::ErrorCode>(
                serde_json::Value::String(body.code),
            )
            .unwrap_or(super::ErrorCode::Plugin);
            Err(super::PluginError::new(code, body.message))
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioProcessorAbi {
    pub init: Option<unsafe extern "C" fn(handle: *mut c_void, sample_rate: u32, channels: u32) -> i32>,
    pub process: Option<
        unsafe extern "C" fn(
            handle: *mut c_void,
            input: *const f32,
            output: *mut f32,
            frames: usize,
            channels: usize,
        ) -> i32,
    >,
    pub reset: Option<unsafe extern "C" fn(handle: *mut c_void) -> i32>,
}