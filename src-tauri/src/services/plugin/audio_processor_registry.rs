use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, RwLock};

use plugin_sdk::abi::AudioProcessorAbi;

pub struct AudioProcessorRegistry {
    processors: RwLock<HashMap<String, AudioProcessorAbi>>,
    handles: Mutex<HashMap<String, *mut c_void>>,
    sample_rate: AtomicU32,
    channels: AtomicU32,
}

impl Default for AudioProcessorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioProcessorRegistry {
    pub fn new() -> Self {
        Self {
            processors: RwLock::new(HashMap::new()),
            handles: Mutex::new(HashMap::new()),
            sample_rate: AtomicU32::new(44100),
            channels: AtomicU32::new(2),
        }
    }

    pub fn set_audio_format(&self, sample_rate: u32, channels: u32) {
        self.sample_rate.store(sample_rate, Ordering::Relaxed);
        self.channels.store(channels, Ordering::Relaxed);
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Relaxed)
    }

    pub fn channels(&self) -> u32 {
        self.channels.load(Ordering::Relaxed)
    }

    pub fn register(&self, plugin_id: String, api: AudioProcessorAbi, handle: *mut c_void) {
        if let Ok(mut lock) = self.processors.write() {
            lock.insert(plugin_id.clone(), api);
        }
        if let Ok(mut lock) = self.handles.lock() {
            lock.insert(plugin_id, handle);
        }
    }

    pub fn unregister(&self, plugin_id: &str) -> Option<(AudioProcessorAbi, *mut c_void)> {
        let api = self
            .processors
            .write()
            .ok()
            .and_then(|mut lock| lock.remove(plugin_id));
        let handle = self
            .handles
            .lock()
            .ok()
            .and_then(|mut lock| lock.remove(plugin_id));
        match (api, handle) {
            (Some(api), Some(handle)) => Some((api, handle)),
            (Some(api), None) => Some((api, std::ptr::null_mut())),
            _ => None,
        }
    }

    pub fn iter_active(&self) -> Vec<(String, AudioProcessorAbi, *mut c_void)> {
        let processors = match self.processors.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        let handles = match self.handles.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        processors
            .iter()
            .map(|(id, api)| {
                let handle = handles.get(id).copied().unwrap_or(std::ptr::null_mut());
                (id.clone(), *api, handle)
            })
            .collect()
    }
}

unsafe impl Send for AudioProcessorRegistry {}
unsafe impl Sync for AudioProcessorRegistry {}
