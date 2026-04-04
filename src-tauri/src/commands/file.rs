use std::fs;
use std::path::PathBuf;

use tauri::command;
use tauri_plugin_dialog::DialogExt;

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::models::MusicFile;

#[command]
pub(crate) async fn select_music_folder(
    app_handle: tauri::AppHandle,
) -> Result<Vec<MusicFile>, String> {
    let folder_path = app_handle.dialog().file().blocking_pick_folder();
    let folder_path_str = folder_path.ok_or("No folder selected")?.to_string();

    let music_files = tauri::async_runtime::spawn_blocking(move || {
        WalkDir::new(folder_path_str)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>()
            .into_par_iter()
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| {
                let path = entry.path();
                let ext = path.extension()?.to_string_lossy().to_lowercase();

                if ["mp3", "flac", "wav", "ogg", "m4a", "aac"].contains(&ext.as_str()) {
                    Some(MusicFile::new(path))
                } else {
                    None
                }
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(music_files)
}

#[command]
pub(crate) async fn delete_file(path: &str) -> Result<String, String> {
    let path_buf = PathBuf::from(path);

    if !path_buf.exists() {
        return Err(format!("File not found: {}", path));
    }

    if !path_buf.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }

    fs::remove_file(&path_buf).map_err(|e| format!("Failed to delete file: {}", e))?;

    Ok(format!("File deleted: {}", path))
}

#[command]
pub(crate) async fn open_containing_folder(
    path: &str,
    _app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let path_buf = PathBuf::from(path);

    if !path_buf.exists() {
        return Err(format!("Path not found: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let _parent = path_buf
            .parent()
            .ok_or("Cannot determine parent directory")?;
        Command::new("explorer")
            .args(["/select,", &path_buf.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg("-R")
            .arg(&path_buf)
            .spawn()
            .map_err(|e| format!("Failed to open Finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let parent = path_buf
            .parent()
            .ok_or("Cannot determine parent directory")?;
        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }

    Ok(())
}
