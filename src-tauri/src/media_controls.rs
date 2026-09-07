use std::ffi::c_void;
use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use souvlaki::{MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use tauri::{AppHandle, Manager, WebviewWindow};

use crate::commands::playback::handle_media_control_event;
use crate::events::PlaybackStatePayload;
use crate::models::playback::NativeTrackMetadata;

#[derive(Clone)]
enum MediaControlMessage {
    Metadata(NativeTrackMetadata),
    Playback(PlaybackStatePayload),
}

static MEDIA_CONTROL_SENDER: OnceLock<Sender<MediaControlMessage>> = OnceLock::new();

#[cfg(target_os = "windows")]
fn window_handle(window: &WebviewWindow) -> Option<isize> {
    window.hwnd().ok().map(|handle| handle.0 as isize)
}

#[cfg(not(target_os = "windows"))]
fn window_handle(_window: &WebviewWindow) -> Option<isize> {
    None
}

pub fn init_media_controls(app_handle: AppHandle) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    let hwnd = window_handle(&window);
    let (tx, rx) = mpsc::channel::<MediaControlMessage>();
    let app_for_events = app_handle.clone();

    thread::spawn(move || {
        #[cfg(target_os = "windows")]
        if hwnd.is_none() {
            return;
        }

        let config = PlatformConfig {
            display_name: "RustEchoMusic",
            dbus_name: "rust_echo_music",
            hwnd: hwnd.map(|value| value as *mut c_void),
        };

        let Ok(mut controls) = MediaControls::new(config) else {
            return;
        };

        let _ = controls.attach(move |event| {
            let app = app_for_events.clone();
            tauri::async_runtime::spawn(async move {
                handle_media_control_event(app, event).await;
            });
        });

        while let Ok(message) = rx.recv() {
            match message {
                MediaControlMessage::Metadata(track) => {
                    let duration = track.duration.map(Duration::from_secs_f64);
                    let metadata = MediaMetadata {
                        title: Some(track.title.as_str()),
                        album: Some(track.album.as_str()),
                        artist: Some(track.artist.as_str()),
                        cover_url: None,
                        duration,
                    };
                    let _ = controls.set_metadata(metadata);
                }
                MediaControlMessage::Playback(state) => {
                    let progress = Some(MediaPosition(Duration::from_secs_f64(state.current_time)));
                    let playback = if state.playing {
                        MediaPlayback::Playing { progress }
                    } else {
                        MediaPlayback::Paused { progress }
                    };
                    let _ = controls.set_playback(playback);
                }
            }
        }
    });

    let _ = MEDIA_CONTROL_SENDER.set(tx);

    Ok(())
}

fn send(message: MediaControlMessage) {
    if let Some(sender) = MEDIA_CONTROL_SENDER.get() {
        let _ = sender.send(message);
    }
}

pub fn update_media_controls_metadata(track: NativeTrackMetadata) {
    send(MediaControlMessage::Metadata(track));
}

pub fn update_media_controls_playback(state: PlaybackStatePayload) {
    send(MediaControlMessage::Playback(state));
}
