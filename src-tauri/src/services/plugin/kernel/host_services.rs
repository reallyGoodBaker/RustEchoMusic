use std::sync::Arc;

use plugin_sdk::{
    EqualizerApi, HostEvent, LibraryApi as LibraryContract, PlayerControlApi, PlayerStateApi,
    PluginError, PluginId, PluginResult, QueueApi as QueueContract,
    RecentReadApi as RecentReadContract, RecentWriteApi as RecentWriteContract,
    SettingsApi as SettingsContract,
};
use tauri::async_runtime::block_on;

use serde_json::json;

use crate::errors::AppError;
use crate::repositories::sqlite::SqliteRecentRepository;
use crate::services::playback_service::PlaybackService;
use crate::services::plugin::settings::settings_registry::SettingsRegistry;
use crate::services::settings_service::SettingsService;
use crate::services::track_service::TrackService;
use crate::state::playback_state::{current_playback_snapshot, with_audio_state};

fn to_plugin_error(error: AppError) -> PluginError {
    match &error {
        AppError::Io(_) | AppError::Database(_) | AppError::Migration(_) => {
            PluginError::io(error.to_string())
        }
        AppError::PluginPermissionDenied { .. } => {
            PluginError::permission_denied(error.to_string())
        }
        _ => PluginError::plugin(error.to_string()),
    }
}

// —— 播放控制 ——

pub struct HostPlayerControl {
    playback: Arc<PlaybackService>,
}

impl HostPlayerControl {
    pub fn new(playback: Arc<PlaybackService>) -> Self {
        Self { playback }
    }
}

impl PlayerControlApi for HostPlayerControl {
    fn play(&self) -> PluginResult<()> {
        block_on(self.playback.resume()).map_err(to_plugin_error)
    }
    fn pause(&self) -> PluginResult<()> {
        block_on(self.playback.pause()).map_err(to_plugin_error)
    }
    fn next(&self) -> PluginResult<()> {
        block_on(self.playback.next()).map_err(to_plugin_error)
    }
    fn previous(&self) -> PluginResult<()> {
        block_on(self.playback.previous()).map_err(to_plugin_error)
    }
}

// —— 播放状态 ——

pub struct HostPlayerState;

impl PlayerStateApi for HostPlayerState {
    fn current_track_id(&self) -> PluginResult<Option<i64>> {
        with_audio_state(|state| state.current_track_id).map_err(to_plugin_error)
    }

    fn current_time_secs(&self) -> PluginResult<f64> {
        current_playback_snapshot()
            .map(|snapshot| snapshot.current_time)
            .map_err(to_plugin_error)
    }

    fn is_playing(&self) -> PluginResult<bool> {
        current_playback_snapshot()
            .map(|snapshot| snapshot.playing)
            .map_err(to_plugin_error)
    }
}

// —— 播放队列 ——

pub struct HostQueue;

impl QueueContract for HostQueue {
    fn current_queue(&self) -> PluginResult<Vec<i64>> {
        with_audio_state(|state| state.playback_queue.tracks.iter().map(|t| t.id).collect())
            .map_err(to_plugin_error)
    }

    fn remove_track(&self, track_id: i64) -> PluginResult<()> {
        with_audio_state(|state| state.playback_queue.remove_track(track_id))
            .map_err(to_plugin_error)?;
        Ok(())
    }

    fn clear(&self) -> PluginResult<()> {
        with_audio_state(|state| state.playback_queue.clear()).map_err(to_plugin_error)?;
        Ok(())
    }
}

// —— 曲库 ——

pub struct HostLibrary {
    tracks: Arc<TrackService>,
}

impl HostLibrary {
    pub fn new(tracks: Arc<TrackService>) -> Self {
        Self { tracks }
    }
}

impl LibraryContract for HostLibrary {
    fn track_path(&self, track_id: i64) -> PluginResult<Option<String>> {
        let track = block_on(self.tracks.get_track(track_id)).map_err(to_plugin_error)?;
        Ok(track.map(|track| track.path))
    }

    fn track_exists(&self, track_id: i64) -> PluginResult<bool> {
        let track = block_on(self.tracks.get_track(track_id)).map_err(to_plugin_error)?;
        Ok(track.is_some())
    }
}

// —— 最近播放（存储原语）——
//
// 宿主只提供**原语**，业务策略（保留多少条、怎么裁剪）在 `recent-plugin` 里。
// 读与写是两个服务、对应两个能力，因此"只想看最近播放"的插件
// 不会被迫申请写权限。

pub struct HostRecent {
    repository: Arc<SqliteRecentRepository>,
}

impl HostRecent {
    pub fn new(repository: Arc<SqliteRecentRepository>) -> Self {
        Self { repository }
    }
}

impl RecentReadContract for HostRecent {
    fn list(&self, limit: i64, offset: i64) -> PluginResult<serde_json::Value> {
        let rows = block_on(self.repository.list_with_tracks(limit, offset))
            .map_err(to_plugin_error)?;
        // 序列化成 JSON 而不是暴露 `Vec<RecentPlayedWithTrack>`：
        // 宿主模型不该出现在契约层，否则插件与宿主的数据结构被焊死。
        serde_json::to_value(rows).map_err(|error| PluginError::io(error.to_string()))
    }

    fn count(&self) -> PluginResult<i64> {
        block_on(self.repository.count()).map_err(to_plugin_error)
    }
}

impl RecentWriteContract for HostRecent {
    fn upsert(&self, track_id: i64, played_at: &str) -> PluginResult<()> {
        block_on(self.repository.upsert(track_id, played_at.to_string()))
            .map_err(to_plugin_error)?;
        Ok(())
    }

    fn remove_oldest(&self, keep: i64) -> PluginResult<()> {
        block_on(self.repository.remove_oldest(keep)).map_err(to_plugin_error)?;
        Ok(())
    }

    fn clear(&self) -> PluginResult<()> {
        block_on(self.repository.clear()).map_err(to_plugin_error)?;
        Ok(())
    }
}

// —— 应用设置 ——

pub struct HostSettings {
    settings: Arc<SettingsService>,
}

impl HostSettings {
    pub fn new(settings: Arc<SettingsService>) -> Self {
        Self { settings }
    }
}

impl SettingsContract for HostSettings {
    fn theme(&self) -> PluginResult<String> {
        let settings = block_on(self.settings.get_settings()).map_err(to_plugin_error)?;
        Ok(settings.theme.as_str().to_string())
    }

    fn set_theme(&self, theme: String) -> PluginResult<()> {
        let theme_mode = crate::models::ThemeMode::try_from(theme).map_err(|error| {
            PluginError::invalid_argument(format!("unknown theme '{error}'"))
        })?;
        let mut settings = block_on(self.settings.get_settings()).map_err(to_plugin_error)?;
        settings.theme = theme_mode;
        block_on(self.settings.update_settings(settings))
            .map(|_| ())
            .map_err(to_plugin_error)
    }
}

// —— 均衡器 ——
//
// 过去 `commands/equalizer.rs` 与 `eq` 插件都能直接改全局 `AudioState`，
// 现在两条写路径都收敛到这个服务（Tauri 命令转发到插件，插件再调用本服务）。
//
// 引擎（`web_audio::ExternalProcessorNode`）可能在播放尚未开始时不存在，
// 此时只更新 `AudioState` 上的缓存值，等引擎创建后由它读取——与改造前
// `commands/equalizer.rs` 的行为一致，因此前端无感。

pub struct HostAudio;

impl HostAudio {
    pub fn new() -> Self {
        Self
    }
}

impl EqualizerApi for HostAudio {
    fn set_band_gain(&self, band: usize, gain_db: f64) -> PluginResult<()> {
        if band >= 10 {
            return Err(PluginError::invalid_argument(format!(
                "EQ band index {band} out of range"
            )));
        }
        with_audio_state(|state| {
            state.eq_bands[band] = gain_db;
            if let Some(ref mut engine) = state.engine {
                engine.set_eq_band_gain(band, gain_db)?;
            }
            Ok::<(), AppError>(())
        })
        .and_then(|inner| inner)
        .map_err(to_plugin_error)
    }

    fn apply_preset(&self, gains: [f64; 10]) -> PluginResult<()> {
        with_audio_state(|state| {
            state.eq_bands = gains;
            if let Some(ref mut engine) = state.engine {
                engine.apply_eq_preset(&gains)?;
            }
            Ok::<(), AppError>(())
        })
        .and_then(|inner| inner)
        .map_err(to_plugin_error)
    }

    fn get_bands(&self) -> PluginResult<[f64; 10]> {
        with_audio_state(|state| Ok(state.eq_bands))
            .and_then(|inner| inner)
            .map_err(to_plugin_error)
    }

    fn set_enabled(&self, enabled: bool) -> PluginResult<()> {
        with_audio_state(|state| {
            state.eq_enabled = enabled;
            if let Some(ref mut engine) = state.engine {
                engine.set_eq_enabled(enabled)?;
            }
            Ok::<(), AppError>(())
        })
        .and_then(|inner| inner)
        .map_err(to_plugin_error)
    }

    fn is_enabled(&self) -> PluginResult<bool> {
        with_audio_state(|state| Ok(state.eq_enabled))
            .and_then(|inner| inner)
            .map_err(to_plugin_error)
    }
}

// —— 每插件一份的服务 ——
//
// 这几个服务天生是"每个插件一份"：插件私有设置、私有目录、事件出口。
// 它们通过 `ServiceContainer::register_factory` 注册——身份由容器在解析时
// 注入，**不是**插件自报的，因此插件拿不到别人的那份。

use std::path::{Path, PathBuf};

use plugin_runtime::EventSink;
use plugin_sdk::{HostEventsApi, PluginSettingsApi, PluginStorageApi};

pub struct HostPluginSettings {
    owner: PluginId,
    registry: Arc<SettingsRegistry>,
}

impl HostPluginSettings {
    pub fn new(owner: PluginId, registry: Arc<SettingsRegistry>) -> Self {
        Self { owner, registry }
    }
}

impl PluginSettingsApi for HostPluginSettings {
    fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.registry
            .get_setting(self.owner.as_str(), key)
            .map(|value| setting_to_json(&value))
    }

    fn set(&self, key: &str, value: serde_json::Value) -> PluginResult<()> {
        self.registry
            .update_setting(self.owner.as_str(), key, json_to_setting(value))
            .map_err(to_plugin_error)
    }

    fn keys(&self) -> Vec<String> {
        self.registry
            .get_all(self.owner.as_str())
            .into_iter()
            .map(|(key, _)| key)
            .collect()
    }
}

pub struct HostPluginStorage {
    root: PathBuf,
}

impl HostPluginStorage {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn ensure(dir: &Path) -> PluginResult<()> {
        std::fs::create_dir_all(dir).map_err(|error| {
            PluginError::io(format!("cannot create plugin directory {}: {error}", dir.display()))
        })
    }
}

impl PluginStorageApi for HostPluginStorage {
    fn private_dir(&self) -> PluginResult<String> {
        let dir = self.root.join("data");
        Self::ensure(&dir)?;
        Ok(dir.to_string_lossy().to_string())
    }

    fn cache_dir(&self) -> PluginResult<String> {
        let dir = self.root.join("cache");
        Self::ensure(&dir)?;
        Ok(dir.to_string_lossy().to_string())
    }
}

pub struct HostPluginEvents {
    source: PluginId,
    sink: EventSink,
}

impl HostPluginEvents {
    pub fn new(source: PluginId, sink: EventSink) -> Self {
        Self { source, sink }
    }
}

impl HostEventsApi for HostPluginEvents {
    fn emit(&self, kind: plugin_sdk::EventType, payload: serde_json::Value) -> PluginResult<()> {
        self.sink.notify(&HostEvent::from_plugin(kind, payload, 0, self.source.clone()));
        Ok(())
    }
}

//
// 转换规则保持无损：JSON 的 bool / 整数 / 浮点 / 字符串 / 字符串数组
// 各自对应一个变体，其余形状兜底为 `Json`。

fn setting_to_json(value: &crate::services::plugin::settings::value::SettingValue) -> serde_json::Value {
    use crate::services::plugin::settings::value::SettingValue;
    match value {
        SettingValue::Bool(flag) => json!(flag),
        SettingValue::Integer(number) => json!(number),
        SettingValue::Float(number) => json!(number),
        SettingValue::Text(text) => json!(text),
        SettingValue::List(items) => json!(items),
        SettingValue::Json(value) => value.clone(),
    }
}

pub fn json_to_setting(value: serde_json::Value) -> crate::services::plugin::settings::value::SettingValue {
    use crate::services::plugin::settings::value::SettingValue;
    match value {
        serde_json::Value::Bool(flag) => SettingValue::Bool(flag),
        serde_json::Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                SettingValue::Integer(int)
            } else if let Some(float) = number.as_f64() {
                SettingValue::Float(float)
            } else {
                SettingValue::Json(serde_json::Value::Number(number))
            }
        }
        serde_json::Value::String(text) => SettingValue::Text(text),
        serde_json::Value::Array(items) => SettingValue::List(
            items
                .into_iter()
                .map(|item| match item {
                    serde_json::Value::String(text) => text,
                    other => other.to_string(),
                })
                .collect(),
        ),
        other => SettingValue::Json(other),
    }
}