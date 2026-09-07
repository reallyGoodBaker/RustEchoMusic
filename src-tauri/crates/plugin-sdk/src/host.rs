use super::error::PluginResult;

pub trait PlayerControlApi: Send + Sync {
    fn play(&self) -> PluginResult<()>;
    fn pause(&self) -> PluginResult<()>;
    fn next(&self) -> PluginResult<()>;
    fn previous(&self) -> PluginResult<()>;
}

pub trait PlayerStateApi: Send + Sync {
    fn current_track_id(&self) -> PluginResult<Option<i64>>;
    fn current_time_secs(&self) -> PluginResult<f64>;
    fn is_playing(&self) -> PluginResult<bool>;
}

pub trait QueueApi: Send + Sync {
    fn current_queue(&self) -> PluginResult<Vec<i64>>;
    fn remove_track(&self, track_id: i64) -> PluginResult<()>;
    fn clear(&self) -> PluginResult<()>;
}

pub trait LibraryApi: Send + Sync {
    fn track_path(&self, track_id: i64) -> PluginResult<Option<String>>;
    fn track_exists(&self, track_id: i64) -> PluginResult<bool>;
}


pub trait RecentReadApi: Send + Sync {
    fn list(&self, limit: i64, offset: i64) -> PluginResult<serde_json::Value>;
    fn count(&self) -> PluginResult<i64>;
}

pub trait RecentWriteApi: Send + Sync {
    fn upsert(&self, track_id: i64, played_at: &str) -> PluginResult<()>;
    fn remove_oldest(&self, keep: i64) -> PluginResult<()>;
    fn clear(&self) -> PluginResult<()>;
}

pub trait SettingsApi: Send + Sync {
    fn theme(&self) -> PluginResult<String>;
    fn set_theme(&self, theme: String) -> PluginResult<()>;
}

pub trait HostEventsApi: Send + Sync {
    fn emit(&self, kind: super::EventType, payload: serde_json::Value) -> PluginResult<()>;
}

pub trait PluginSettingsApi: Send + Sync {
    fn get(&self, key: &str) -> Option<serde_json::Value>;
    fn set(&self, key: &str, value: serde_json::Value) -> PluginResult<()>;
    fn keys(&self) -> Vec<String>;
}

pub trait PluginStorageApi: Send + Sync {
    fn private_dir(&self) -> PluginResult<String>;
    fn cache_dir(&self) -> PluginResult<String>;
}

pub trait EqualizerApi: Send + Sync {
    fn set_band_gain(&self, band: usize, gain_db: f64) -> PluginResult<()>;
    fn apply_preset(&self, gains: [f64; 10]) -> PluginResult<()>;
    fn get_bands(&self) -> PluginResult<[f64; 10]>;
    fn set_enabled(&self, enabled: bool) -> PluginResult<()>;
    fn is_enabled(&self) -> PluginResult<bool>;
}
