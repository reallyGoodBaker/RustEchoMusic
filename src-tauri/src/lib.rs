<<<<<<< HEAD
pub mod commands;
pub mod models;
use commands::audio::{self, PlayerState};
use commands::file;
use web_audio_api::context::AudioContext;
=======
mod audio;
mod commands;
mod db;
mod errors;
mod events;
mod media_controls;
mod metadata;
mod models;
mod repositories;
mod services;
mod state;

use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuBuilder, MenuItem, SubmenuBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

use commands::library::{
    delete_track_file, delete_track_files, execute_scan, get_track_cover, load_track_library,
    scan_track_directories, show_in_folder, trash_track_files,
};

use commands::playback::{
    current_time, get_current_status, insert_track_as_next, pause_track, play_next_track,
    play_previous_track, play_queue_track, play_track, remove_track_from_queue, resume_track,
    set_current_time, set_volume, stop_track, sync_playback_queue, toggle_track,
};
use services::playback_service::PlaybackService;

use commands::recent::{add_recently_played, clear_recently_played, load_recently_played};

use commands::settings::{get_settings, load_settings, save_settings, update_settings};

use commands::menu::show_context_menu;
use commands::playlists::{
    add_track_to_playlist, clear_playlist_tracks, create_playlist, delete_playlist, get_playlist,
    get_playlist_with_tracks, list_playlists, list_playlists_with_tracks,
    remove_track_from_playlist, rename_playlist, reorder_playlist_track,
};
use commands::tracks::{
    create_track, delete_track, delete_track_by_path, get_track, get_track_by_path, list_tracks,
    mark_track_played, search_tracks, update_track, upsert_track,
};
use db::DatabaseManager;
use media_controls::init_media_controls;
use repositories::sqlite::{
    SqlitePlaylistRepository, SqliteRecentRepository, SqliteSettingsRepository,
    SqliteTrackRepository,
};
use services::plugin::settings::persistence::{JsonPluginStorage, PluginStorage};
use services::plugin::settings::settings_registry::SettingsRegistry;
use services::{PlaylistService, SettingsService, TrackService};
use state::AppState;

use commands::playback_queue::{
    clear_queue, get_playback_queue, insert_tracks_as_next, replace_playlist_and_play,
    set_play_mode,
};

use commands::plugin_extensions::{
    disable_plugin_command, enable_plugin_command, execute_plugin_command,
    get_all_sidebar_extensions, get_menu_extensions, get_native_view_extensions,
    get_plugin_manifests, get_plugin_settings, get_plugin_view, get_sidebar_extensions,
    update_plugin_setting,
};

use commands::equalizer::{apply_eq_preset, get_eq_state, set_eq_band, set_eq_enabled};
use commands::plugin::{plugin_diagnostics, plugin_emit, plugin_detail};
use commands::plugin_kernel::{plugin_contributions_full, plugin_kernel_snapshot};

use audio::init_audio_state;

use crate::errors::AppError;

pub fn init_startup_scan(_app_handle: tauri::AppHandle) {}
>>>>>>> pr/4

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = AudioContext::default();
    let player_state = PlayerState::new();

    tauri::Builder::default()
<<<<<<< HEAD
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(context)
        .manage(player_state)
        .plugin(tauri_plugin_dialog::init())
        // .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            audio::play_music,
            audio::play_music_from_path,
            audio::pause_music,
            audio::resume_music,
            audio::stop_music,
            file::select_music_folder,
            file::delete_file,
            file::open_containing_folder
=======
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .register_uri_scheme_protocol("plugin", crate::services::plugin::asset::scheme::handle)
        .setup(|app| {
            println!("[Timer] setup start");
            let app_handle = app.handle().clone();

            println!("[Timer] creating database...");
            let database = tauri::async_runtime::block_on(DatabaseManager::new(&app_handle))?;
            let pool = database.pool().clone();

            let track_repository = Arc::new(SqliteTrackRepository::new(pool.clone()));
            let playlist_repository = Arc::new(SqlitePlaylistRepository::new(pool.clone()));
            let setting_repository = Arc::new(SqliteSettingsRepository::new(pool.clone()));
            let recent_repository = Arc::new(SqliteRecentRepository::new(pool));

            println!("[Timer] loading settings...");
            let setting_service = Arc::new(SettingsService::new(setting_repository));
            tauri::async_runtime::block_on(setting_service.get_settings())?;

            println!("[Timer] creating services...");
            let track_service = Arc::new(TrackService::new(track_repository));
            let playlist_service = Arc::new(PlaylistService::new(playlist_repository));

            let settings_registry = Arc::new(SettingsRegistry::new());

            let audio_processor_registry = Arc::new(
                crate::services::plugin::AudioProcessorRegistry::new(),
            );

            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let storage = Arc::new(JsonPluginStorage::new(app_data_dir.clone()));

            match storage.load_config() {
                Ok((_, settings)) => settings_registry.load_all(settings),
                Err(e) => eprintln!("[startup] failed to load persisted plugin settings: {}", e),
            }

            // 默认不启用发现与激活，因此开启后行为与今天完全一致。
            // 必须在 `AppState::new` 之前构造：那一步会 move 走几个 service 的 Arc。
            // 出站：插件广播的事件 → 宿主应用总线 → 前端。
            let sink_handle = app.handle().clone();
            let plugin_runtime = Arc::new(
                services::plugin::kernel::build_runtime(services::plugin::kernel::HostDeps {
                    playback: Arc::new(PlaybackService::new(app.handle().clone())),
                    tracks: track_service.clone(),
                    settings: setting_service.clone(),
                    plugin_settings: Arc::clone(&settings_registry),
                    app_data_dir: app_data_dir.clone(),
                    recent: Arc::clone(&recent_repository),
                    packaged_dir: None,
                })
                .on_plugin_event(move |event| {
                    services::plugin::kernel::events::host_event_to_app(&sink_handle, event);
                }),
            );

            let report = plugin_runtime.discover();
            for issue in &report.issues {
                eprintln!(
                    "[plugin] discovery issue at {}: {}",
                    issue.path.display(),
                    issue.message
                );
            }
            for outcome in plugin_runtime.activate_all() {
                match &outcome.error {
                    None => eprintln!("[plugin] activated '{}'", outcome.plugin),
                    Some(error) => {
                        eprintln!("[plugin] failed to activate '{}': {error}", outcome.plugin)
                    }
                }
            }

            for binding in plugin_runtime.audio_processors() {
                audio_processor_registry.register(
                    binding.plugin.to_string(),
                    binding.abi,
                    binding.instance,
                );
            }

            // 入站：宿主应用事件 → 插件事件 → 插件订阅者。
            let bus_runtime = Arc::clone(&plugin_runtime);
            crate::events::EventBus::subscribe(Box::new(move |event| {
                if let Some((kind, payload)) =
                    services::plugin::kernel::events::app_event_to_host_event(event)
                {
                    let _ = bus_runtime.emit(kind, payload);
                }
            }));
            app.manage(Arc::clone(&plugin_runtime));

            let app_state = AppState::new(
                track_service,
                playlist_service,
                setting_service,
                Arc::clone(&plugin_runtime),
                audio_processor_registry,
            );

            app.manage(app_state);
            app.manage(Arc::clone(&settings_registry));
            app.manage(Arc::clone(&storage));

            init_audio_state(models::PlaybackQueue::default());

            println!("[Timer] init media controls...");
            let _ = init_media_controls(app.handle().clone());
            println!("[Timer] starting playback progress task...");
            PlaybackService::spawn_playback_progress_task(app.handle().clone());

            println!("[Timer] setup complete, WebView should render now");

            let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&settings_item, &quit_item])?;
            let file_menu = SubmenuBuilder::new(app, "文件")
                .text("quit", "退出")
                .build()?;

            let app_menu = MenuBuilder::new(app).items(&[&file_menu]).build()?;
            app.set_menu(app_menu)?;

            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or_else(|| AppError::from("Application default window icon is missing"))?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(
                    |app_handle: &tauri::AppHandle, event| match event.id.as_ref() {
                        "settings" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.emit("tray:navigate", "settings");
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app_handle.exit(0);
                        }
                        _ => {}
                    },
                )
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                if let Ok(settings) = state.settings.get_settings().await {
                    if settings.scan_on_startup {
                        let library_dirs = settings.library_dirs;
                        let scanned_result = tauri::async_runtime::spawn_blocking(move || {
                            execute_scan(library_dirs)
                        })
                        .await;

                        match scanned_result {
                            Ok(Ok(scanned)) => {
                                let mut tracks = Vec::with_capacity(scanned.len());
                                for track in scanned {
                                    if let Ok(saved) = state.tracks.upsert_track(track).await {
                                        tracks.push(saved);
                                    }
                                }
                                let _ = handle.emit("library:refreshed", tracks);
                            }
                            Ok(Err(error)) => {
                                eprintln!("{}", error);
                            }
                            Err(error) => {
                                eprintln!("{}", error);
                            }
                        }
                    }
                }
            });

            app.on_menu_event(move |app_handle: &tauri::AppHandle, event| {
                match event.id().0.as_str() {
                    "quit" => {
                        app_handle.exit(0);
                    }
                    "play" => {
                        let handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = PlaybackService::new(handle).toggle().await;
                        });
                    }
                    "next" => {
                        let handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = PlaybackService::new(handle).next().await;
                        });
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_context_menu,
            create_track,
            upsert_track,
            update_track,
            delete_track,
            delete_track_by_path,
            get_track,
            get_track_by_path,
            list_tracks,
            search_tracks,
            mark_track_played,
            create_playlist,
            rename_playlist,
            delete_playlist,
            get_playlist,
            list_playlists,
            list_playlists_with_tracks,
            get_playlist_with_tracks,
            add_track_to_playlist,
            remove_track_from_playlist,
            clear_playlist_tracks,
            reorder_playlist_track,
            play_track,
            play_queue_track,
            insert_track_as_next,
            remove_track_from_queue,
            play_next_track,
            play_previous_track,
            stop_track,
            sync_playback_queue,
            resume_track,
            pause_track,
            toggle_track,
            current_time,
            get_current_status,
            set_current_time,
            set_volume,
            scan_track_directories,
            show_in_folder,
            delete_track_file,
            delete_track_files,
            trash_track_files,
            load_track_library,
            get_track_cover,
            load_recently_played,
            add_recently_played,
            clear_recently_played,
            get_settings,
            update_settings,
            load_settings,
            save_settings,
            get_playback_queue,
            replace_playlist_and_play,
            insert_tracks_as_next,
            set_play_mode,
            clear_queue,
            get_menu_extensions,
            get_sidebar_extensions,
            get_all_sidebar_extensions,
            get_native_view_extensions,
            get_plugin_manifests,
            get_plugin_settings,
            update_plugin_setting,
            enable_plugin_command,
            disable_plugin_command,
            execute_plugin_command,
            get_plugin_view,
            get_eq_state,
            set_eq_band,
            apply_eq_preset,
            set_eq_enabled,
            plugin_diagnostics,
            plugin_emit,
            plugin_detail,
            plugin_kernel_snapshot,
            plugin_contributions_full,
>>>>>>> pr/4
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
