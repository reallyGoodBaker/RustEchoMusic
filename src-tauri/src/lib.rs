use std::fs::{self, File};

use serde::{Deserialize, Serialize};
use tauri::command;
use tauri::path::BaseDirectory;
use tauri::Manager;

use tauri_plugin_dialog::DialogExt;

use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

#[derive(Debug, Serialize, Deserialize)]
struct MusicFile {
    path: String,
    name: String,
}

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

#[command]
fn select_music_folder(app_handle: tauri::AppHandle) -> Result<Vec<MusicFile>, String> {
    let folder_path = app_handle.dialog().file().blocking_pick_folder();

    let folder_path = folder_path.ok_or("No folder selected")?;

    let folder_path_str = folder_path.to_string();

    println!("Scanning folder: {}", folder_path_str);

    let entries =
        fs::read_dir(folder_path_str).map_err(|e| format!("Failed to read folder: {}", e))?;

    let mut music_files = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ["mp3", "flac", "wav", "ogg", "m4a", "aac"].contains(&ext_str.as_str()) {
                    if let Some(name) = path.file_stem() {
                        music_files.push(MusicFile {
                            path: path.to_string_lossy().to_string(),
                            name: name.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    if music_files.is_empty() {
        return Err("No music files found in the selected folder(s)".to_string());
    }

    println!("Found {} music files", music_files.len());

    Ok(music_files)
}

#[command]
fn play_music_from_path(path: &str) -> Result<String, String> {
    let context = AudioContext::default();

    let file = File::open(path).map_err(|e| format!("Failed to open file at {}: {}", path, e))?;

    let buffer = context
        .decode_audio_data_sync(file)
        .map_err(|e| format!("Failed to decode audio data: {}", e))?;

    let mut src = context.create_buffer_source();
    src.set_buffer(buffer);
    src.set_loop(false);

    src.connect(&context.destination());
    src.start();

    Ok(format!("Playing music: {}", path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            play_music,
            select_music_folder,
            play_music_from_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
