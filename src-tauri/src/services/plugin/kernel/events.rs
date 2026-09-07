use plugin_sdk::{EventType, HostEvent};
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::events::payloads::{AppEvent, LyricLinePayload, LyricsLoadedPayload};
use crate::events::EventBus;

pub fn app_event_to_host_event(event: &AppEvent) -> Option<(EventType, Value)> {
    let kind = match event {
        AppEvent::TrackStarted(_) => track_changed(),
        AppEvent::PlaybackStateChanged(_) => playback_state(),
        AppEvent::QueueChanged(_) => queue_changed(),
        AppEvent::SettingsChanged(_) => settings_changed(),
        // 音量与播放进度是高频事件（进度每秒多次），投送给插件收益低、
        // 不回灌，避免插件 A 的事件触发插件 A。
        AppEvent::VolumeChanged(_)
        | AppEvent::PlaybackProgress(_)
        | AppEvent::LyricsLoaded(_) => return None,
    };

    let payload = match event {
        AppEvent::TrackStarted(payload) => json!({
            "trackId": payload.track.id,
            "title": payload.track.title,
            "artist": payload.track.artist,
            "album": payload.track.album,
            "index": payload.index,
        }),
        AppEvent::PlaybackStateChanged(payload) => json!({
            "playing": payload.playing,
            "currentTime": payload.current_time,
        }),
        AppEvent::QueueChanged(queue) => json!({
            "trackIds": queue.tracks.iter().map(|t| t.id).collect::<Vec<_>>(),
        }),
        AppEvent::SettingsChanged(settings) => json!({
            "theme": settings.theme.as_str(),
        }),
        _ => Value::Null,
    };

    Some((kind, payload))
}

pub fn host_event_to_app(app: &AppHandle, event: &HostEvent) {
    if event.kind == lyrics_loaded() {
        let Some(song_id) = event.payload.get("songId").and_then(Value::as_i64) else {
            eprintln!("[plugin] lyrics.loaded 缺少 songId，已丢弃");
            return;
        };
        let lines = event
            .payload
            .get("lines")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| {
                        Some(LyricLinePayload {
                            timestamp_ms: entry.get("timestampMs")?.as_u64()?,
                            text: entry.get("text")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        if let Err(error) = EventBus::emit_lyrics_loaded(
            app,
            LyricsLoadedPayload { song_id, lines },
        ) {
            eprintln!("[plugin] 转发 lyrics.loaded 失败: {error}");
        }
    }
}

// —— 事件类型常量 ——
//
// 用 `expect` 是安全的：这些都是编译期常量，非法即意味着代码写错了。

pub fn track_changed() -> EventType {
    EventType::new("track.changed").expect("static event type is valid")
}
pub fn playback_state() -> EventType {
    EventType::new("playback.state").expect("static event type is valid")
}
pub fn queue_changed() -> EventType {
    EventType::new("queue.changed").expect("static event type is valid")
}
pub fn settings_changed() -> EventType {
    EventType::new("settings.changed").expect("static event type is valid")
}
pub fn lyrics_loaded() -> EventType {
    EventType::new("lyrics.loaded").expect("static event type is valid")
}