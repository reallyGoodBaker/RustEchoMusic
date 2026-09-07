use std::sync::Arc;

use plugin_runtime::JsonService;
use plugin_sdk::{
    capabilities, EqualizerApi, LibraryApi as LibraryContract, PlayerControlApi, PlayerStateApi,
    PluginError, PluginResult, QueueApi as QueueContract, RecentReadApi as RecentReadContract,
    RecentWriteApi as RecentWriteContract, ServiceDescriptor, ServiceId,
    SettingsApi as SettingsContract, Version,
};
use serde_json::{json, Value};

use super::host_services::{
    HostAudio, HostLibrary, HostPlayerControl, HostPlayerState, HostQueue, HostRecent, HostSettings,
};

const HOST_SERVICE_VERSION: (u32, u32, u32) = (1, 0, 0);

fn version() -> Version {
    Version::new(HOST_SERVICE_VERSION.0, HOST_SERVICE_VERSION.1, HOST_SERVICE_VERSION.2)
}

// —— 参数提取。统一返回 `InvalidArgument`，让插件能区分"我用错了"与
//     "宿主挂了"。——

fn want_i64(args: &Value, key: &str) -> PluginResult<i64> {
    args.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| PluginError::invalid_argument(format!("'{key}' must be an integer")))
}

fn want_f64(args: &Value, key: &str) -> PluginResult<f64> {
    args.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| PluginError::invalid_argument(format!("'{key}' must be a number")))
}

fn want_str(args: &Value, key: &str) -> PluginResult<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| PluginError::invalid_argument(format!("'{key}' must be a string")))
}

fn want_bool(args: &Value, key: &str) -> PluginResult<bool> {
    args.get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| PluginError::invalid_argument(format!("'{key}' must be a boolean")))
}

// —— 播放控制 ——

pub struct PlayerControlJson(pub Arc<HostPlayerControl>);

impl JsonService for PlayerControlJson {
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::new(plugin_sdk::services::player(), version())
            .requiring(capabilities::player_control())
            .with_summary("播放控制：play / pause / next / previous")
    }

    fn call(&self, method: &str, _args: &Value) -> PluginResult<Value> {
        match method {
            "play" => self.0.play().map(|()| Value::Null),
            "pause" => self.0.pause().map(|()| Value::Null),
            "next" => self.0.next().map(|()| Value::Null),
            "previous" => self.0.previous().map(|()| Value::Null),
            other => Err(PluginError::not_found(format!(
                "player.control has no method '{other}'"
            ))),
        }
    }
}

// —— 播放状态 ——

pub struct PlayerStateJson;

impl JsonService for PlayerStateJson {
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::new(plugin_sdk::services::player_state(), version())
            .requiring(capabilities::player_read())
            .with_summary("播放状态：currentTrackId / currentTimeSecs / isPlaying")
    }

    fn call(&self, method: &str, _args: &Value) -> PluginResult<Value> {
        match method {
            "currentTrackId" => Ok(json!(HostPlayerState.current_track_id()?)),
            "currentTimeSecs" => Ok(json!(HostPlayerState.current_time_secs()?)),
            "isPlaying" => Ok(json!(HostPlayerState.is_playing()?)),
            other => Err(PluginError::not_found(format!(
                "player.state has no method '{other}'"
            ))),
        }
    }
}

// —— 播放队列 ——

pub struct QueueJson;

impl JsonService for QueueJson {
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::new(plugin_sdk::services::queue(), version())
            .requiring(capabilities::queue_read())
            .with_summary("播放队列：currentQueue / removeTrack / clear")
    }

    fn call(&self, method: &str, args: &Value) -> PluginResult<Value> {
        match method {
            "currentQueue" => Ok(json!(HostQueue.current_queue()?)),
            "removeTrack" => {
                let track_id = want_i64(args, "trackId")?;
                HostQueue.remove_track(track_id).map(|()| Value::Null)
            }
            "clear" => HostQueue.clear().map(|()| Value::Null),
            other => Err(PluginError::not_found(format!(
                "queue.control has no method '{other}'"
            ))),
        }
    }
}

// —— 曲库 ——

pub struct LibraryJson(pub Arc<HostLibrary>);

impl JsonService for LibraryJson {
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::new(plugin_sdk::services::library(), version())
            .requiring(capabilities::library_read())
            .with_summary("曲库：trackPath / trackExists")
    }

    fn call(&self, method: &str, args: &Value) -> PluginResult<Value> {
        match method {
            "trackPath" => {
                let track_id = want_i64(args, "trackId")?;
                Ok(json!(self.0.track_path(track_id)?))
            }
            "trackExists" => {
                let track_id = want_i64(args, "trackId")?;
                Ok(json!(self.0.track_exists(track_id)?))
            }
            other => Err(PluginError::not_found(format!(
                "library.read has no method '{other}'"
            ))),
        }
    }
}

// —— 应用设置 ——

pub struct SettingsJson(pub Arc<HostSettings>);

impl JsonService for SettingsJson {
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::new(plugin_sdk::services::settings(), version())
            .requiring(capabilities::settings_read())
            .with_summary("应用设置：theme / setTheme")
    }

    fn call(&self, method: &str, args: &Value) -> PluginResult<Value> {
        match method {
            "theme" => Ok(json!(self.0.theme()?)),
            "setTheme" => {
                let theme = want_str(args, "theme")?;
                self.0.set_theme(theme).map(|()| Value::Null)
            }
            other => Err(PluginError::not_found(format!(
                "settings.app has no method '{other}'"
            ))),
        }
    }
}

// —— 最近播放 ——
//
// 与强类型通道共用同一个 `HostRecent` 实例，只做转发、不含业务逻辑。

pub struct RecentReadJson(pub Arc<HostRecent>);

impl JsonService for RecentReadJson {
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::new(plugin_sdk::services::recent_read(), version())
            .requiring(capabilities::recent_read())
            .with_summary("最近播放（读）：list / count")
    }

    fn call(&self, method: &str, args: &Value) -> PluginResult<Value> {
        match method {
            "list" => {
                let limit = want_i64(args, "limit")?;
                let offset = want_i64(args, "offset")?;
                self.0.list(limit, offset)
            }
            "count" => Ok(json!(self.0.count()?)),
            other => Err(PluginError::not_found(format!(
                "recent.read has no method '{other}'"
            ))),
        }
    }
}

pub struct RecentWriteJson(pub Arc<HostRecent>);

impl JsonService for RecentWriteJson {
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::new(plugin_sdk::services::recent_write(), version())
            .requiring(capabilities::recent_write())
            .with_summary("最近播放（写）：upsert / removeOldest / clear")
    }

    fn call(&self, method: &str, args: &Value) -> PluginResult<Value> {
        match method {
            "upsert" => {
                let track_id = want_i64(args, "trackId")?;
                let played_at = want_str(args, "playedAt")?;
                self.0.upsert(track_id, &played_at).map(|()| Value::Null)
            }
            "removeOldest" => {
                let keep = want_i64(args, "keep")?;
                self.0.remove_oldest(keep).map(|()| Value::Null)
            }
            "clear" => self.0.clear().map(|()| Value::Null),
            other => Err(PluginError::not_found(format!(
                "recent.write has no method '{other}'"
            ))),
        }
    }
}

pub fn documented_services() -> Vec<(ServiceId, Vec<&'static str>)> {
    vec![
        (
            plugin_sdk::services::player(),
            vec!["play", "pause", "next", "previous"],
        ),
        (
            plugin_sdk::services::player_state(),
            vec!["currentTrackId", "currentTimeSecs", "isPlaying"],
        ),
        (
            plugin_sdk::services::queue(),
            vec!["currentQueue", "removeTrack", "clear"],
        ),
        (
            plugin_sdk::services::library(),
            vec!["trackPath", "trackExists"],
        ),
        (
            plugin_sdk::services::settings(),
            vec!["theme", "setTheme"],
        ),
        (
            plugin_sdk::services::recent_read(),
            vec!["list", "count"],
        ),
        (
            plugin_sdk::services::recent_write(),
            vec!["upsert", "removeOldest", "clear"],
        ),
        (
            plugin_sdk::services::equalizer(),
            vec!["setBandGain", "applyPreset", "getBands", "setEnabled", "isEnabled"],
        ),
    ]
}

// `want_f64` 目前只有阶段 2 的 EQ 服务会用到（增益是浮点数）。
// 提前定义并标注，避免到时候各写各的参数校验。
#[allow(dead_code)]
const _F64_HELPER_RESERVED_FOR_EQ: fn(&Value, &str) -> PluginResult<f64> = want_f64;

pub struct EqualizerJson;

impl JsonService for EqualizerJson {
    fn descriptor(&self) -> ServiceDescriptor {
        ServiceDescriptor::new(plugin_sdk::services::equalizer(), version())
            .requiring(capabilities::audio_process())
            .with_summary("10 段均衡器：setBandGain / applyPreset / getBands / setEnabled / isEnabled")
    }

    fn call(&self, method: &str, args: &Value) -> PluginResult<Value> {
        let audio = HostAudio::new();
        match method {
            "setBandGain" => {
                let band = want_usize(args, "band")?;
                let gain = want_f64(args, "gainDb")?;
                audio.set_band_gain(band, gain)?;
                Ok(Value::Null)
            }
            "applyPreset" => {
                let gains = want_bands(args, "gains")?;
                audio.apply_preset(gains)?;
                Ok(Value::Null)
            }
            "getBands" => Ok(json!(audio.get_bands()?)),
            "setEnabled" => {
                let enabled = want_bool(args, "enabled")?;
                audio.set_enabled(enabled)?;
                Ok(Value::Null)
            }
            "isEnabled" => Ok(json!(audio.is_enabled()?)),
            other => Err(PluginError::not_found(format!(
                "equalizer has no method '{other}'"
            ))),
        }
    }
}

fn want_usize(args: &Value, key: &str) -> PluginResult<usize> {
    let value = args
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| PluginError::invalid_argument(format!("missing integer '{key}'")))?;
    Ok(value as usize)
}

fn want_bands(args: &Value, key: &str) -> PluginResult<[f64; 10]> {
    let array = args
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| PluginError::invalid_argument(format!("missing array '{key}'")))?;
    if array.len() != 10 {
        return Err(PluginError::invalid_argument(format!(
            "'{key}' must have exactly 10 entries, got {}",
            array.len()
        )));
    }
    let mut bands = [0.0; 10];
    for (index, item) in array.iter().enumerate() {
        bands[index] = item
            .as_f64()
            .ok_or_else(|| PluginError::invalid_argument(format!("'{key}'[{index}] not a number")))?;
    }
    Ok(bands)
}
