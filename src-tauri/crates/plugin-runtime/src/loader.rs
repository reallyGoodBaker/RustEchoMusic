use std::path::{Path, PathBuf};
use std::sync::Arc;

use plugin_sdk::{PluginError, PluginId, PluginResult};

pub const ABI_SYMBOL: &[u8] = b"REM_PLUGIN_ABI";

pub fn library_suffix() -> &'static str {
    if cfg!(target_os = "windows") {
        ".dll"
    } else if cfg!(target_os = "macos") {
        ".dylib"
    } else {
        ".so"
    }
}

pub fn candidate_paths(root: &Path, id: &str) -> Vec<PathBuf> {
    let suffix = library_suffix();
    [
        format!("{id}{suffix}"),
        format!("lib{id}{suffix}"),
        id.to_string(),
        format!("lib{id}"),
    ]
    .into_iter()
    .map(|name| root.join(name))
    .collect()
}

pub struct LoadedLibrary {
    id: PluginId,
    path: PathBuf,
    lib: Arc<libloading::Library>,
    abi: *const plugin_sdk::abi::PluginAbi,
}

// SAFETY: `lib` 是内部同步的 `Arc<Library>`；`abi` 指向库内的 `static`，
// 只读且生命周期被 `lib` 覆盖。二者都不会随线程逃逸出可变别名。
unsafe impl Send for LoadedLibrary {}
unsafe impl Sync for LoadedLibrary {}

impl LoadedLibrary {
    pub fn open(id: &PluginId, root: &Path) -> PluginResult<Self> {
        let path = candidate_paths(root, id.as_str())
            .into_iter()
            .find(|candidate| candidate.exists())
            .ok_or_else(|| {
                PluginError::io(format!(
                    "no shared library found for plugin '{id}' under {}",
                    root.display()
                ))
            })?;

        Self::open_path(id, path)
    }

    pub fn open_path(id: &PluginId, path: PathBuf) -> PluginResult<Self> {
        // 先把文件读进内存再 dlopen，这样在 Windows 上不会长期占用文件句柄，
        // 用户可以在插件运行时替换磁盘上的文件（热更新前置条件）。
        let library = unsafe { libloading::Library::new(&path) }.map_err(|error| {
            PluginError::io(format!(
                "cannot load '{}': {error}",
                path.display()
            ))
        })?;

        let abi: *const plugin_sdk::abi::PluginAbi = unsafe {
            let symbol: libloading::Symbol<*const plugin_sdk::abi::PluginAbi> =
                library.get(ABI_SYMBOL).map_err(|error| {
                    PluginError::incompatible(format!(
                        "library '{}' does not export '{}': {error}",
                        path.display(),
                        String::from_utf8_lossy(ABI_SYMBOL)
                    ))
                })?;
            let struct_ptr = symbol.into_raw().as_raw_ptr() as *const plugin_sdk::abi::PluginAbi;
            let owned: plugin_sdk::abi::PluginAbi = std::ptr::read(struct_ptr);
            Box::into_raw(Box::new(owned))
        };
        if abi.is_null() {
            return Err(PluginError::incompatible(format!(
                "plugin '{id}' exported a null ABI pointer"
            )));
        }

        // 校验必须在**任何**调用之前完成。
        unsafe { &*abi }.is_compatible().map_err(|error| {
            PluginError::incompatible(format!("plugin '{id}': {error}"))
        })?;

        Ok(Self {
            id: id.clone(),
            path,
            lib: Arc::new(library),
            abi,
        })
    }

    pub fn id(&self) -> &PluginId {
        &self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub unsafe fn abi(&self) -> &plugin_sdk::abi::PluginAbi {
        &*self.abi
    }

    pub fn external_refs(&self) -> usize {
        Arc::strong_count(&self.lib) - 1
    }
}

impl std::fmt::Debug for LoadedLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedLibrary")
            .field("id", &self.id)
            .field("path", &self.path)
            .finish()
    }
}

pub fn builtin_marker(id: &PluginId) -> PathBuf {
    PathBuf::from(format!("builtin://{}", id.as_str()))
}