pub mod events;
pub mod host_services;
pub mod json_views;

use std::path::PathBuf;
use std::sync::Arc;

use plugin_runtime::{DirectoryLocator, PluginRuntime};
use plugin_sdk::{
    capabilities, services, ActivationContext, EventPattern, HostEvent, HostEventsApi,
    InvokeContext, LibraryApi as LibraryContract, Manifest, PluginId, PluginResult, Plugin,
    PluginSettingsApi, PluginStorageApi, ServiceDescriptor, ServiceHandle, Version,
};

use crate::services::playback_service::PlaybackService;
use crate::services::plugin::settings::settings_registry::SettingsRegistry;
use crate::services::settings_service::SettingsService;
use crate::services::track_service::TrackService;

use host_services::{
    HostAudio, HostLibrary, HostPlayerControl, HostPlayerState, HostPluginEvents,
    HostPluginSettings, HostPluginStorage, HostQueue, HostRecent, HostSettings,
};
use json_views::{
    EqualizerJson, LibraryJson, PlayerControlJson, PlayerStateJson, QueueJson, RecentReadJson,
    RecentWriteJson, SettingsJson,
};

use plugin_sdk::{
    EqualizerApi as EqualizerContract, PlayerControlApi, PlayerStateApi, QueueApi as QueueContract,
    RecentReadApi as RecentReadContract, RecentWriteApi as RecentWriteContract,
    SettingsApi as SettingsContract,
};

const HOST_SERVICE_VERSION: Version = Version::new(1, 0, 0);

pub struct HostDeps {
    pub playback: Arc<PlaybackService>,
    pub tracks: Arc<TrackService>,
    pub settings: Arc<SettingsService>,
    pub plugin_settings: Arc<SettingsRegistry>,
    pub app_data_dir: PathBuf,
    pub recent: Arc<crate::repositories::sqlite::SqliteRecentRepository>,
    pub packaged_dir: Option<PathBuf>,
}

pub fn build_runtime(deps: HostDeps) -> PluginRuntime {
    let mut runtime = PluginRuntime::new(host_version());

    register_host_services(&runtime, &deps);
    register_per_plugin_services(&runtime, &deps);

    // —— 核心插件：永远注册，永远激活 ——
    //
    // `recent` 是第一个被整体做成插件的核心域功能。它走与其它插件
    // 完全相同的生命周期、DI、贡献点与失败隔离，只是不可被用户禁用。
    seed_defaults(&deps.plugin_settings, &recent_plugin::manifest());
    runtime = runtime.with_builtin(recent_plugin::manifest(), || {
        Ok(Box::new(recent_plugin::RecentPlugin::new()))
    });

    runtime = register_directory_locators(runtime, &deps);

    if probe_enabled() {
        runtime = runtime.with_builtin(probe_manifest(), probe_factory);
    }
    seed_defaults(&deps.plugin_settings, &lyrics_plugin::manifest());
    runtime = runtime.with_builtin(lyrics_plugin::manifest(), || {
        Ok(Box::new(lyrics_plugin::LyricsPlugin::new()))
    });
    seed_defaults(&deps.plugin_settings, &eq_plugin::manifest());
    runtime = runtime.with_builtin(eq_plugin::manifest(), || {
        Ok(Box::new(eq_plugin::EqPlugin::new()))
    });

    for problem in verify_host_surface(&runtime) {
        eprintln!("[plugin] host surface: {problem}");
    }

    runtime
}

fn register_directory_locators(runtime: PluginRuntime, deps: &HostDeps) -> PluginRuntime {
    // 用户自装插件目录。**刻意不用 `app_data_dir/plugins`**：那个目录已经是
    // 插件私有存储的根（`<root>/<id>/{data,cache}`），让两者共用一棵树
    // 意味着扫描时会把每个插件的 data/cache 目录当成"待发现的插件"，
    // 每次启动都产生一堆 "missing plugin.json" 噪声。
    let user_dir = deps.app_data_dir.join("extensions");
    let mut runtime = runtime.with_locator(DirectoryLocator::new(
        user_dir,
        plugin_sdk::PluginSource::User,
    ));

    if let Some(packaged) = &deps.packaged_dir {
        runtime = runtime.with_locator(DirectoryLocator::new(
            packaged,
            plugin_sdk::PluginSource::Packaged,
        ));
    }
    runtime
}

fn host_version() -> Version {
    parse_version(env!("CARGO_PKG_VERSION")).unwrap_or(Version::new(0, 0, 0))
}

pub fn parse_version(raw: &str) -> Option<Version> {
    let mut parts = raw.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some(Version::new(major, minor, patch))
}

fn register_host_services(runtime: &PluginRuntime, deps: &HostDeps) {
    let player = Arc::new(HostPlayerControl::new(Arc::clone(&deps.playback)));
    let library = Arc::new(HostLibrary::new(Arc::clone(&deps.tracks)));
    let app_settings = Arc::new(HostSettings::new(Arc::clone(&deps.settings)));
    let recent = Arc::new(HostRecent::new(Arc::clone(&deps.recent)));
    let audio = Arc::new(HostAudio::new());

    let registered = [
        runtime.provide_host_service(
            ServiceDescriptor::new(services::player(), HOST_SERVICE_VERSION)
                .requiring(capabilities::player_control())
                .with_summary("播放控制"),
            Arc::new(ServiceHandle::new(
                Arc::clone(&player) as Arc<dyn PlayerControlApi>
            )),
        ),
        runtime.provide_host_service(
            ServiceDescriptor::new(services::player_state(), HOST_SERVICE_VERSION)
                .requiring(capabilities::player_read())
                .with_summary("播放状态（只读）"),
            Arc::new(ServiceHandle::new(
                Arc::new(HostPlayerState) as Arc<dyn PlayerStateApi>
            )),
        ),
        runtime.provide_host_service(
            ServiceDescriptor::new(services::queue(), HOST_SERVICE_VERSION)
                .requiring(capabilities::queue_read())
                .with_summary("播放队列"),
            Arc::new(ServiceHandle::new(Arc::new(HostQueue) as Arc<dyn QueueContract>)),
        ),
        runtime.provide_host_service(
            ServiceDescriptor::new(services::library(), HOST_SERVICE_VERSION)
                .requiring(capabilities::library_read())
                .with_summary("曲库查询"),
            Arc::new(ServiceHandle::new(
                Arc::clone(&library) as Arc<dyn LibraryContract>
            )),
        ),
        runtime.provide_host_service(
            ServiceDescriptor::new(services::settings(), HOST_SERVICE_VERSION)
                .requiring(capabilities::settings_read())
                .with_summary("应用设置"),
            Arc::new(ServiceHandle::new(
                Arc::clone(&app_settings) as Arc<dyn SettingsContract>
            )),
        ),
        runtime.provide_host_service(
            ServiceDescriptor::new(services::recent_read(), HOST_SERVICE_VERSION)
                .requiring(capabilities::recent_read())
                .with_summary("最近播放（读原语）"),
            Arc::new(ServiceHandle::new(
                Arc::clone(&recent) as Arc<dyn RecentReadContract>
            )),
        ),
        runtime.provide_host_service(
            ServiceDescriptor::new(services::recent_write(), HOST_SERVICE_VERSION)
                .requiring(capabilities::recent_write())
                .with_summary("最近播放（写原语）"),
            Arc::new(ServiceHandle::new(
                Arc::clone(&recent) as Arc<dyn RecentWriteContract>
            )),
        ),
        runtime.provide_host_service(
            ServiceDescriptor::new(services::equalizer(), HOST_SERVICE_VERSION)
                .requiring(capabilities::audio_process())
                .with_summary("10 段均衡器"),
            Arc::new(ServiceHandle::new(
                Arc::clone(&audio) as Arc<dyn EqualizerContract>
            )),
        ),
    ];
    for result in registered {
        if let Err(error) = result {
            eprintln!("[plugin] 注册宿主服务失败: {error}");
        }
    }

    runtime.provide_host_json_service(Arc::new(PlayerControlJson(Arc::clone(&player))));
    runtime.provide_host_json_service(Arc::new(PlayerStateJson));
    runtime.provide_host_json_service(Arc::new(QueueJson));
    runtime.provide_host_json_service(Arc::new(LibraryJson(Arc::clone(&library))));
    runtime.provide_host_json_service(Arc::new(SettingsJson(Arc::clone(&app_settings))));
    runtime.provide_host_json_service(Arc::new(RecentReadJson(Arc::clone(&recent))));
    runtime.provide_host_json_service(Arc::new(RecentWriteJson(Arc::clone(&recent))));
    runtime.provide_host_json_service(Arc::new(EqualizerJson));
}

fn register_per_plugin_services(runtime: &PluginRuntime, deps: &HostDeps) {
    let registry = Arc::clone(&deps.plugin_settings);
    let plugins_root = deps.app_data_dir.join("plugins");
    let sink = runtime.event_sink();

    let registered = [
        {
            let registry = Arc::clone(&registry);
            runtime.provide_host_factory::<ServiceHandle<dyn PluginSettingsApi>>(
                ServiceDescriptor::new(services::plugin_settings(), HOST_SERVICE_VERSION)
                    .with_summary("插件私有设置（每插件一份）"),
                move |plugin| {
                    Some(Arc::new(ServiceHandle::new(
                        Arc::new(HostPluginSettings::new(
                            plugin.clone(),
                            Arc::clone(&registry),
                        )) as Arc<dyn PluginSettingsApi>
                    )))
                },
            )
        },
        {
            let root = plugins_root.clone();
            runtime.provide_host_factory::<ServiceHandle<dyn PluginStorageApi>>(
                ServiceDescriptor::new(services::storage(), HOST_SERVICE_VERSION)
                    .requiring(capabilities::storage_private())
                    .with_summary("插件私有目录（每插件一份）"),
                move |plugin| {
                    Some(Arc::new(ServiceHandle::new(
                        Arc::new(HostPluginStorage::new(root.join(plugin.as_str())))
                            as Arc<dyn PluginStorageApi>
                    )))
                },
            )
        },
        {
            let sink = sink.clone();
            runtime.provide_host_factory::<ServiceHandle<dyn HostEventsApi>>(
                ServiceDescriptor::new(services::events(), HOST_SERVICE_VERSION)
                    .with_summary("插件事件出口（事件来源由容器注入）"),
                move |plugin| {
                    Some(Arc::new(ServiceHandle::new(
                        Arc::new(HostPluginEvents::new(plugin.clone(), sink.clone()))
                            as Arc<dyn HostEventsApi>
                    )))
                },
            )
        },
    ];
    for result in registered {
        if let Err(error) = result {
            eprintln!("[plugin] 注册 per-plugin 服务失败: {error}");
        }
    }
}

pub fn seed_defaults(registry: &SettingsRegistry, manifest: &Manifest) {
    let mut defaults = std::collections::HashMap::new();
    for setting in &manifest.settings {
        defaults.insert(
            setting.key.clone(),
            host_services::json_to_setting(setting.default_value.clone()),
        );
    }
    if !defaults.is_empty() {
        registry.register_defaults(manifest.id.as_str(), defaults);
    }
}

pub fn verify_host_surface(runtime: &PluginRuntime) -> Vec<String> {
    let mut problems = Vec::new();
    for (id, methods) in json_views::documented_services() {
        if methods.is_empty() {
            problems.push(format!("service '{id}' documents no methods"));
        }
        if !runtime.services().has(&id) {
            problems.push(format!("service '{id}' has no typed implementation"));
        }
        if !runtime.json_services().has(&id) {
            problems.push(format!(
                "service '{id}' has no JSON view; packaged plugins cannot call it"
            ));
        }
    }
    // per-plugin 服务只验强类型通道：动态库插件通过句柄调用，不需要 JSON 视图。
    for id in [
        services::plugin_settings(),
        services::storage(),
        services::events(),
    ] {
        if !runtime.services().has(&id) {
            problems.push(format!("per-plugin service '{id}' is not registered"));
        }
    }
    problems
}

pub fn feature_plugins_enabled() -> bool {
    flag("REM_PLUGIN")
}

pub fn probe_enabled() -> bool {
    flag("REM_PLUGIN_PROBE")
}

fn flag(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1" | "true" | "TRUE" | "on")
    )
}

// —— 自检插件 ——

const PROBE_ID: &str = "host.probe";

struct ProbePlugin {
    descriptor: plugin_sdk::PluginDescriptor,
}

impl Plugin for ProbePlugin {
    fn descriptor(&self) -> &plugin_sdk::PluginDescriptor {
        &self.descriptor
    }

    fn activate(&self, ctx: &mut ActivationContext) -> PluginResult<()> {
        let state = ctx.require::<ServiceHandle<dyn PlayerStateApi>>(&services::player_state())?;
        let _track = state.get()?.current_track_id()?;
        let _playing = state.get()?.is_playing()?;
        ctx.subscribe(EventPattern::All);
        eprintln!("[plugin] probe activated: host services reachable");
        Ok(())
    }

    fn deactivate(&self, _ctx: &plugin_sdk::TeardownContext) -> PluginResult<()> {
        eprintln!("[plugin] probe deactivated");
        Ok(())
    }

    fn on_event(&self, event: &HostEvent, _ctx: &InvokeContext) -> PluginResult<()> {
        eprintln!(
            "[plugin] probe saw event '{}' (#{})",
            event.kind, event.sequence
        );
        Ok(())
    }
}

fn probe_manifest() -> Manifest {
    let mut manifest = Manifest::builtin(
        PluginId::new(PROBE_ID).expect("probe id is valid"),
        Version::new(0, 1, 0),
        host_version(),
    );
    manifest.name = PROBE_ID.into();
    manifest.display_name = "宿主自检探针".into();
    manifest.capabilities = vec![capabilities::player_read()];
    manifest
}

fn probe_factory() -> PluginResult<Box<dyn Plugin>> {
    Ok(Box::new(ProbePlugin {
        descriptor: plugin_sdk::PluginDescriptor::new(
            PluginId::new(PROBE_ID).expect("probe id is valid"),
            Version::new(0, 1, 0),
            host_version(),
        )
        .with_display_name("宿主自检探针"),
    }))
}