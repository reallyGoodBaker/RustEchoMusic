pub(crate) mod cache;
pub(crate) mod models;
pub(crate) mod service;

use std::sync::RwLock;

use plugin_sdk::{
    capabilities, services, ActivationContext, CommandId, CommandSpec, Contribution, EventPattern,
    EventType, HostEvent, HostEventsApi, InvokeContext, LibraryApi, Manifest, MenuSpec,
    NativeViewSpec, PluginDescriptor, PluginError, PluginId, PluginResult, PluginSettingsApi,
    PluginStorageApi, Plugin, ServiceHandle, SettingSpec, SidebarSpec, Version,
};
use serde_json::{json, Value};

use crate::cache::LyricsCacheService;
use crate::models::LyricDocument;
use crate::service::LyricsService;

pub const ID: &str = "lyrics";

const VERSION: (u32, u32, u32) = (1, 0, 0);

pub fn event_lyrics_loaded() -> EventType {
    EventType::new("lyrics.loaded").expect("static event type is valid")
}

fn event_track_changed() -> EventType {
    EventType::new("track.changed").expect("static event type is valid")
}

pub fn manifest() -> Manifest {
    let mut manifest = Manifest::builtin(
        PluginId::new(ID).expect("lyrics id is valid"),
        Version::new(VERSION.0, VERSION.1, VERSION.2),
        Version::new(0, 1, 0),
    );
    manifest.name = ID.into();
    manifest.display_name = "Lyrics".into();
    manifest.author = "RustEchoMusic".into();
    manifest.description = "歌词搜索、加载、缓存与同步显示".into();
    manifest.capabilities = vec![
        capabilities::player_read(),
        capabilities::library_read(),
        capabilities::storage_private(),
    ];
    manifest.settings = vec![
        SettingSpec {
            key: "provider".into(),
            title: "歌词来源".into(),
            default_value: json!("local"),
            control: "text".into(),
            options: None,
        },
        SettingSpec {
            key: "auto_search".into(),
            title: "自动搜索歌词".into(),
            default_value: json!(true),
            control: "bool".into(),
            options: None,
        },
        SettingSpec {
            key: "auto_scroll".into(),
            title: "自动滚动".into(),
            default_value: json!(true),
            control: "bool".into(),
            options: None,
        },
        SettingSpec {
            key: "cache_enabled".into(),
            title: "启用缓存".into(),
            default_value: json!(true),
            control: "bool".into(),
            options: None,
        },
    ];
    manifest
}

struct LyricsState {
    song_id: Option<i64>,
    lyrics: Option<LyricDocument>,
}

pub struct LyricsPlugin {
    descriptor: PluginDescriptor,
    cache: RwLock<Option<LyricsCacheService>>,
    state: RwLock<LyricsState>,
}

impl LyricsPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: PluginDescriptor::new(
                PluginId::new(ID).expect("lyrics id is valid"),
                Version::new(VERSION.0, VERSION.1, VERSION.2),
                Version::new(0, 1, 0),
            )
            .with_display_name("Lyrics"),
            cache: RwLock::new(None),
            state: RwLock::new(LyricsState {
                song_id: None,
                lyrics: None,
            }),
        }
    }

    fn init_cache(&self, storage: &dyn PluginStorageApi) {
        let mut cache_lock = match self.cache.write() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if cache_lock.is_some() {
            return;
        }
        let Ok(dir) = storage.cache_dir() else {
            return;
        };
        let service = LyricsCacheService::new(std::path::PathBuf::from(dir));
        if let Err(error) = service.init() {
            eprintln!("[lyrics] 缓存初始化失败: {error}");
        }
        *cache_lock = Some(service);
    }

    fn bool_setting(settings: &dyn PluginSettingsApi, key: &str) -> bool {
        settings
            .get(key)
            .and_then(|value| value.as_bool())
            .unwrap_or(true)
    }

    fn publish_loaded(
        &self,
        events: &dyn HostEventsApi,
        song_id: i64,
        document: &LyricDocument,
    ) -> PluginResult<()> {
        events.emit(
            event_lyrics_loaded(),
            json!({
                "songId": song_id,
                "lines": document
                    .lines
                    .iter()
                    .map(|line| json!({ "timestampMs": line.timestamp_ms, "text": line.text }))
                    .collect::<Vec<_>>(),
            }),
        )
    }

    fn handle_track_changed(
        &self,
        track_id: i64,
        settings: &dyn PluginSettingsApi,
        library: &dyn LibraryApi,
        events: &dyn HostEventsApi,
    ) {
        {
            let mut state = match self.state.write() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            if state.song_id == Some(track_id) {
                return;
            }
            state.song_id = Some(track_id);
            state.lyrics = None;
        }

        if !Self::bool_setting(settings, "auto_search") {
            return;
        }

        let cache_guard = match self.cache.read() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let Some(cache) = cache_guard.as_ref() else {
            return;
        };

        let Ok(Some(path)) = library.track_path(track_id) else {
            return;
        };

        if let Some(document) =
            LyricsService::load_or_fetch(cache, track_id, std::path::Path::new(&path))
        {
            if let Err(error) = self.publish_loaded(events, document.song_id, &document) {
                eprintln!("[lyrics] 广播歌词失败: {error}");
            }
            if let Ok(mut state) = self.state.write() {
                if state.song_id == Some(track_id) {
                    state.lyrics = Some(document);
                }
            }
        }
    }
}

impl Default for LyricsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for LyricsPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn activate(&self, ctx: &mut ActivationContext) -> PluginResult<()> {
        // 依赖按需解析。缺任何一个都让激活失败
        let storage = ctx.require::<ServiceHandle<dyn PluginStorageApi>>(&services::storage())?;
        self.init_cache(storage.get()?.as_ref());

        ctx.contribute(Contribution::Command(CommandSpec {
            id: CommandId::new("lyrics.search").expect("static command id is valid"),
            title: "搜索歌词".into(),
            category: None,
            input_schema: None,
        }));
        ctx.contribute(Contribution::Command(CommandSpec {
            id: CommandId::new("lyrics.load").expect("static command id is valid"),
            title: "加载歌词".into(),
            category: None,
            input_schema: None,
        }));
        ctx.contribute(Contribution::Command(CommandSpec {
            id: CommandId::new("lyrics.clearCache").expect("static command id is valid"),
            title: "清除歌词缓存".into(),
            category: None,
            input_schema: None,
        }));
        ctx.contribute(Contribution::SidebarItem(SidebarSpec {
            id: "lyrics".into(),
            title: "歌词".into(),
            icon: "lyrics".into(),
            target: "lyrics-panel-view".into(),
        }));
        ctx.contribute(Contribution::NativeView(NativeViewSpec {
            id: "lyrics-panel-view".into(),
            title: "Lyrics".into(),
            token: "lyrics-panel".into(),
            icon: Some("lyrics".into()),
        }));
        ctx.contribute(Contribution::MenuItem(MenuSpec {
            command: CommandId::new("lyrics.search").expect("static command id is valid"),
            title: "搜索歌词".into(),
            location: "track.context".into(),
            group: None,
            order: Some(10),
        }));

        ctx.subscribe(EventPattern::kind(event_track_changed()));
        Ok(())
    }

    fn deactivate(&self, _ctx: &plugin_sdk::TeardownContext) -> PluginResult<()> {
        if let Ok(mut state) = self.state.write() {
            state.song_id = None;
            state.lyrics = None;
        }
        Ok(())
    }

    fn on_event(&self, event: &HostEvent, ctx: &InvokeContext) -> PluginResult<()> {
        if event.kind != event_track_changed() {
            return Ok(());
        }
        let track_id = match event.payload.get("trackId").and_then(Value::as_i64) {
            Some(id) => id,
            None => return Ok(()),
        };

        let settings =
            ctx.require::<ServiceHandle<dyn PluginSettingsApi>>(&services::plugin_settings())?;
        let library = ctx.require::<ServiceHandle<dyn LibraryApi>>(&services::library())?;
        let events = ctx.require::<ServiceHandle<dyn HostEventsApi>>(&services::events())?;

        self.handle_track_changed(
            track_id,
            settings.get()?.as_ref(),
            library.get()?.as_ref(),
            events.get()?.as_ref(),
        );
        Ok(())
    }

    fn execute(&self, command: &CommandId, args: Value, ctx: &InvokeContext) -> PluginResult<Value> {
        match command.as_str() {
            "lyrics.search" => {
                let title = args.get("title").and_then(Value::as_str).unwrap_or_default();
                let artist = args.get("artist").and_then(Value::as_str).unwrap_or_default();
                println!("[lyrics] search '{title} - {artist}' (local only)");
                Ok(Value::Null)
            }
            "lyrics.load" => {
                let song_id = args
                    .get("songId")
                    .and_then(Value::as_i64)
                    .ok_or_else(|| PluginError::invalid_argument("lyrics.load 需要 'songId'"))?;

                let library =
                    ctx.require::<ServiceHandle<dyn LibraryApi>>(&services::library())?;
                let events = ctx.require::<ServiceHandle<dyn HostEventsApi>>(&services::events())?;
                let path = library.get()?.track_path(song_id)?.ok_or_else(|| {
                    PluginError::not_found(format!("曲目 {song_id} 的路径不存在"))
                })?;

                let cache_guard = self
                    .cache
                    .read()
                    .map_err(|error| PluginError::plugin(error.to_string()))?;
                let cache = cache_guard
                    .as_ref()
                    .ok_or_else(|| PluginError::service_unavailable("歌词缓存尚未初始化"))?;

                let document =
                    LyricsService::load_or_fetch(cache, song_id, std::path::Path::new(&path))
                        .ok_or_else(|| {
                            PluginError::not_found(format!("未找到曲目 {song_id} 的歌词"))
                        })?;

                self.publish_loaded(events.get()?.as_ref(), document.song_id, &document)?;
                if let Ok(mut state) = self.state.write() {
                    state.song_id = Some(song_id);
                    state.lyrics = Some(document);
                }
                Ok(Value::Null)
            }
            "lyrics.clearCache" => {
                if let Ok(cache_guard) = self.cache.read() {
                    if let Some(cache) = cache_guard.as_ref() {
                        cache
                            .clear()
                            .map_err(|error| PluginError::plugin(error.to_string()))?;
                    }
                }
                Ok(Value::Null)
            }
            other => Err(PluginError::not_found(format!(
                "lyrics 不支持命令 '{other}'"
            ))),
        }
    }
}
