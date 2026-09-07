use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use base64::{engine::general_purpose, Engine as _};
use lofty::prelude::*;
use lofty::probe::Probe;
use tauri::{command, State};
use walkdir::WalkDir;

use crate::commands::playback::remove_track_from_queue;
use crate::errors::AppError;
use crate::metadata::parse_single_track;
use crate::models::{NewTrack, Track};
use crate::state::AppState;

const SUPPORTED_EXT: [&str; 5] = ["mp3", "flac", "m4a", "wav", "ogg"];

fn is_supported_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXT.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn scan_single_directory(dir: &str) -> Result<Vec<NewTrack>, AppError> {
    let root_path = Path::new(dir);

    if !root_path.exists() || !root_path.is_dir() {
        return Err(format!("无效目录: {}", dir).into());
    }

    let mut tracks = Vec::new();

    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if !path.is_file() || !is_supported_audio_file(path) {
            continue;
        }

        if let Some(mut track) = parse_single_track(path) {
            track.cover = None;
            tracks.push(track);
        }
    }

    Ok(tracks)
}

fn dedupe_new_tracks(tracks: &mut Vec<NewTrack>) {
    let mut seen = HashSet::new();
    tracks.retain(|track| seen.insert(track.path.clone()));
}

pub fn execute_scan(dirs: Vec<String>) -> Result<Vec<NewTrack>, AppError> {
    let mut all_tracks = Vec::new();

    for dir in dirs {
        if let Ok(mut tracks) = scan_single_directory(&dir) {
            all_tracks.append(&mut tracks);
        }
    }

    dedupe_new_tracks(&mut all_tracks);
    Ok(all_tracks)
}

async fn upsert_scanned_tracks(
    state: &AppState,
    tracks: Vec<NewTrack>,
) -> Result<Vec<Track>, AppError> {
    state
        .tracks
        .upsert_tracks(tracks)
        .await
}

#[command]
pub async fn get_track_cover(track_id: i64, state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    let track = state.tracks.get_track(track_id).await?;
    let track = match track {
        Some(t) => t,
        None => return Err("曲目不存在".into()),
    };

    let file_path = Path::new(&track.path);

    if !file_path.exists() {
        return Err("音频文件不存在".into());
    }

    let tagged_file = Probe::open(file_path)
        .map_err(|e| e.to_string())?
        .read()
        .map_err(|e| e.to_string())?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let cover = tag.and_then(|t| t.pictures().first()).map(|pic| {
        let b64_encoded = general_purpose::STANDARD.encode(pic.data());
        let mime_type = pic.mime_type().map(|m| m.as_str()).unwrap_or("image/jpeg");

        format!("data:{};base64,{}", mime_type, b64_encoded)
    });

    Ok(cover)
}

fn validate_audio_path(path: &str) -> Result<&Path, AppError> {
    let file_path = Path::new(path);

    if !file_path.exists() {
        return Err("音频文件不存在".into());
    }

    if !file_path.is_file() || !is_supported_audio_file(file_path) {
        return Err(format!("拒绝操作非音频文件: {}", path).into());
    }

    Ok(file_path)
}

#[command]
pub async fn show_in_folder(track_id: i64, state: State<'_, AppState>) -> Result<(), AppError> {
    let track = state.tracks.get_track(track_id).await?;
    let track = track.ok_or_else(|| AppError::from("曲目不存在"))?;
    let file_path = Path::new(&track.path);

    if !file_path.exists() {
        return Err("文件不存在".into());
    }

    #[cfg(target_os = "linux")]
    let parent = file_path
        .parent()
        .ok_or_else(|| AppError::from("无法定位文件所在目录"))?;

    #[cfg(target_os = "windows")]
    let status = Command::new("explorer.exe")
        .arg(format!("/select,{}", file_path.display()))
        .status();

    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg("-R").arg(file_path).status();

    #[cfg(target_os = "linux")]
    let status = Command::new("xdg-open").arg(parent).status();

    let status = status?;

    if status.success() {
        Ok(())
    } else {
        Err("打开文件所在目录失败".into())
    }
}

#[command]
pub async fn delete_track_file(
    app_handle: tauri::AppHandle,
    track_id: i64,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let track = state.tracks.get_track(track_id).await?;
    let track = track.ok_or_else(|| AppError::from("曲目不存在"))?;

    let file_path = validate_audio_path(&track.path)?;
    trash::delete(file_path).map_err(|e| format!("移至回收站失败 {}: {}", track.path, e))?;

    remove_track_from_queue(app_handle, track_id).await?;

    Ok(())
}

#[command]
pub async fn trash_track_files(track_ids: Vec<i64>, state: State<'_, AppState>) -> Result<(), AppError> {
    for track_id in track_ids {
        let track = state.tracks.get_track(track_id).await?;
        let track = match track {
            Some(t) => t,
            None => continue,
        };

        let file_path = Path::new(&track.path);
        if !file_path.exists() {
            continue;
        }

        if !file_path.is_file() || !is_supported_audio_file(file_path) {
            return Err(format!("拒绝移至回收站非音频文件: {}", track.path).into());
        }

        trash::delete(file_path).map_err(|e| format!("移至回收站失败 {}: {}", track.path, e))?;
    }

    Ok(())
}

#[command]
pub async fn delete_track_files(track_ids: Vec<i64>, state: State<'_, AppState>) -> Result<(), AppError> {
    for track_id in track_ids {
        let track = state.tracks.get_track(track_id).await?;
        let track = match track {
            Some(t) => t,
            None => continue,
        };

        let file_path = Path::new(&track.path);
        if !file_path.exists() {
            continue;
        }

        if !file_path.is_file() || !is_supported_audio_file(file_path) {
            return Err(format!("拒绝删除非音频文件: {}", track.path).into());
        }

        tokio::fs::remove_file(file_path)
            .await
            .map_err(|e| format!("删除文件失败 {}: {}", track.path, e))?;
    }

    Ok(())
}

#[command]
pub async fn scan_track_directories(
    state: State<'_, AppState>,
    dirs: Vec<String>,
) -> Result<Vec<Track>, AppError> {
    let scanned = execute_scan(dirs)?;
    let tracks = upsert_scanned_tracks(&state, scanned).await?;
    Ok(tracks)
}

#[command]
pub async fn load_track_library(state: State<'_, AppState>) -> Result<Vec<Track>, AppError> {
    let tracks = state.tracks.list_tracks().await?;

    if tracks.is_empty() {
        return rebuild_and_get_library(&state).await;
    }

    Ok(tracks)
}

async fn rebuild_and_get_library(state: &AppState) -> Result<Vec<Track>, AppError> {
    let settings = state
        .settings
        .get_settings()
        .await
        .map_err(|error| error.to_string())?;
    let scanned = tauri::async_runtime::spawn_blocking(move || execute_scan(settings.library_dirs))
        .await
        .map_err(|e| e.to_string())??;
    let tracks = upsert_scanned_tracks(state, scanned).await?;
    Ok(tracks)
}
