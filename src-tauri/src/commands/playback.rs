use crate::audio::state::PlayMode;
use crate::errors::AppError;
use crate::events::{AppEvent, EventBus};
use crate::models::playback::PlaybackStatusSnapshot;
use crate::models::Track;
use crate::services::playback_service::PlaybackService;
use crate::state::playback_state::{
    current_playback_snapshot, current_time_from_state, lock_audio_state, sanitize_track,
    with_audio_state,
};
use tauri::{command, AppHandle};

pub use crate::services::media_control_service::handle_media_control_event;

#[command]
pub async fn sync_playback_queue(
    app_handle: AppHandle,
    playlist: Vec<Track>,
    current_index: Option<usize>,
    play_mode: PlayMode,
    history: Vec<i64>,
) -> Result<(), AppError> {
    {
        let mut state = lock_audio_state()?;

        state.playback_queue.sync(
            playlist.into_iter().map(sanitize_track).collect(),
            current_index,
            play_mode,
            history,
        );
    }

    PlaybackService::emit_queue_changed(&app_handle)?;

    Ok(())
}

#[command]
pub async fn remove_track_from_queue(app_handle: AppHandle, track_id: i64) -> Result<(), AppError> {
    PlaybackService::new(app_handle)
        .remove_track_from_queue(track_id)
        .await
}

#[command]
pub async fn insert_track_as_next(app_handle: AppHandle, track: Track) -> Result<(), AppError> {
    with_audio_state(|state| {
        state.playback_queue.insert_next(track);
    })?;
    PlaybackService::emit_queue_changed(&app_handle)?;
    Ok(())
}

#[command]
pub async fn play_queue_track(app_handle: AppHandle, index: usize) -> Result<(), AppError> {
    PlaybackService::new(app_handle).play_queue_index(index)?;
    Ok(())
}

#[command]
pub async fn play_next_track(app_handle: AppHandle) -> Result<(), AppError> {
    PlaybackService::new(app_handle).next().await?;
    Ok(())
}

#[command]
pub async fn play_previous_track(app_handle: AppHandle) -> Result<(), AppError> {
    PlaybackService::new(app_handle).previous().await?;
    Ok(())
}

#[command]
pub async fn stop_track(app_handle: AppHandle) -> Result<(), AppError> {
    PlaybackService::new(app_handle).stop().await?;
    Ok(())
}

#[command]
pub async fn play_track(
    app_handle: AppHandle,
    track_id: i64,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<String, AppError> {
    let track = state.tracks.get_track(track_id).await?;
    let track = track.ok_or_else(|| AppError::from("曲目不存在"))?;
    let service = PlaybackService::new(app_handle);
    service.play_track(track, None)?;
    Ok(format!("Playing track: {}", track_id))
}

#[command]
pub async fn resume_track(app_handle: AppHandle) -> Result<(), AppError> {
    PlaybackService::new(app_handle).resume().await?;
    Ok(())
}

#[command]
pub async fn pause_track(app_handle: AppHandle) -> Result<(), AppError> {
    PlaybackService::new(app_handle).pause().await?;
    Ok(())
}

#[command]
pub async fn toggle_track(app_handle: AppHandle) -> Result<bool, AppError> {
    let status = PlaybackService::new(app_handle).toggle().await?;
    Ok(status)
}

#[command]
pub async fn current_time() -> f64 {
    current_time_from_state()
}

#[command]
pub async fn get_current_status() -> Result<PlaybackStatusSnapshot, AppError> {
    current_playback_snapshot()
}

#[command]
pub async fn set_current_time(app_handle: AppHandle, time: f64) -> Result<(), AppError> {
    PlaybackService::new(app_handle).seek(time).await
}

#[command]
pub async fn set_volume(app_handle: tauri::AppHandle, volume: f32) -> Result<(), AppError> {
    let mut state = lock_audio_state()?;
    let safe_volume = (volume / 100.0).clamp(0.0, 1.0);

    state.volume = safe_volume;

    if let Some(ref engine) = state.engine {
        engine.set_volume(safe_volume);
    }

    EventBus::emit(&app_handle, AppEvent::VolumeChanged(safe_volume))?;

    Ok(())
}
