use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{command, Emitter, Manager, State};
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

pub struct PlayerState {
    pub current_source: Mutex<Option<web_audio_api::node::AudioBufferSourceNode>>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            current_source: Mutex::new(None),
        }
    }

    pub fn stop_current(&self) {
        if let Ok(mut guard) = self.current_source.lock() {
            if let Some(ref mut source) = *guard {
                let _ = source.stop();
            }
            *guard = None;
        }
    }

    pub fn set_source(&self, source: web_audio_api::node::AudioBufferSourceNode) {
        if let Ok(mut guard) = self.current_source.lock() {
            *guard = Some(source);
        }
    }
}

#[command]
pub(crate) async fn play_music(
    name: &str,
    app_handle: tauri::AppHandle,
    context: State<'_, AudioContext>,
    player: State<'_, PlayerState>,
) -> Result<String, String> {
    use tauri::path::BaseDirectory;
    let resource_path = app_handle
        .path()
        .resolve(name, BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    internal_play(resource_path, &context, &player, &app_handle).await?;

    Ok(format!("Playing music: {}", name))
}

#[command]
pub(crate) async fn play_music_from_path(
    path: &str,
    app_handle: tauri::AppHandle,
    context: State<'_, AudioContext>,
    player: State<'_, PlayerState>,
) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    internal_play(path_buf, &context, &player, &app_handle).await?;
    Ok(format!("Playing music: {}", path))
}

#[command]
pub(crate) async fn pause_music(context: State<'_, AudioContext>) -> Result<(), String> {
    context.suspend().await;
    Ok(())
}

#[command]
pub(crate) async fn resume_music(context: State<'_, AudioContext>) -> Result<(), String> {
    context.resume().await;
    Ok(())
}

#[command]
pub(crate) async fn stop_music(player: State<'_, PlayerState>) -> Result<(), String> {
    player.stop_current();
    Ok(())
}

async fn internal_play(
    path: PathBuf,
    context: &AudioContext,
    player: &PlayerState,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    player.stop_current();

    let _ = app_handle.emit("audio:buffering", true);

    let file =
        File::open(&path).map_err(|e| format!("Failed to open file at {:?}: {}", path, e))?;

    let _file_size = file.metadata().map(|m| m.len()).unwrap_or(0);

    let reader = BufReader::with_capacity(64 * 1024, file);

    let _ = app_handle.emit("audio:progress", 0.0);

    let buffer = context
        .decode_audio_data(reader)
        .await
        .map_err(|e| format!("Failed to decode audio data: {}", e))?;

    let duration = buffer.duration();
    let _ = app_handle.emit("audio:duration", duration);
    let _ = app_handle.emit("audio:progress", 1.0);
    let _ = app_handle.emit("audio:buffering", false);

    let mut src = context.create_buffer_source();
    src.set_buffer(buffer);
    src.set_loop(false);

    src.connect(&context.destination());
    src.start();

    player.set_source(src);

    Ok(())
}
