use crate::audio::state::PlayMode;
use crate::errors::AppError;
use crate::models::{PlaybackQueue, Track};
use crate::services::playback_service::PlaybackService;
use crate::state::playback_state::{lock_audio_state, with_audio_state};
use tauri::{command, AppHandle};

#[command]
pub fn get_playback_queue() -> Result<PlaybackQueue, AppError> {
    with_audio_state(|state| state.playback_queue.clone())
}

#[command]
pub async fn clear_queue(app_handle: AppHandle) -> Result<(), AppError> {
    {
        let mut state = lock_audio_state()?;
        state.playback_queue.clear();
    }

    PlaybackService::new(app_handle.clone()).stop().await?;
    PlaybackService::emit_queue_changed(&app_handle)?;
    Ok(())
}

#[command]
pub fn set_play_mode(app_handle: AppHandle, mode: PlayMode) -> Result<(), AppError> {
    {
        let mut state = lock_audio_state()?;
        state.playback_queue.play_mode = mode;
    }
    PlaybackService::emit_queue_changed(&app_handle)?;
    Ok(())
}

#[command]
pub fn insert_tracks_as_next(app_handle: AppHandle, tracks: Vec<Track>) -> Result<(), AppError> {
    {
        let mut state = lock_audio_state()?;
        state.playback_queue.insert_tracks_as_next(tracks);
    }
    PlaybackService::emit_queue_changed(&app_handle)?;
    Ok(())
}

#[command]
pub async fn replace_playlist_and_play(
    app_handle: AppHandle,
    tracks: Vec<Track>,
    target_id: i64,
) -> Result<(), AppError> {
    let play_index = {
        let mut state = lock_audio_state()?;
        state.playback_queue.replace_playlist(tracks, target_id)?
    };
    PlaybackService::emit_queue_changed(&app_handle)?;
    PlaybackService::new(app_handle).play_queue_index(play_index)?;
    Ok(())
}
