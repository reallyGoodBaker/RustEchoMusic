use std::fs::File;

use tauri::command;
use tauri::path::BaseDirectory;
use tauri::Manager;

use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

#[command]
fn play_music(name: &str, app_handle: tauri::AppHandle) -> Result<String, String> {
    let context = AudioContext::default();

    let resource_path = app_handle
        .path()
        .resolve(name, BaseDirectory::Resource)
        .map_err(|e| format!("Failed to resolve path: {}", e))?;

    println!("Attempting to open file at: {:?}", resource_path);

    let file = File::open(&resource_path)
        .map_err(|e| format!("Failed to open file at {:?}: {}", resource_path, e))?;

    let buffer = context
        .decode_audio_data_sync(file)
        .map_err(|e| format!("Failed to decode audio data: {}", e))?;

    let mut src = context.create_buffer_source();
    src.set_buffer(buffer);
    src.set_loop(false);

    src.connect(&context.destination());
    src.start();

    Ok(format!("Playing music: {}", name))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![play_music])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
