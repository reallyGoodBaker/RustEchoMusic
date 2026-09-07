use crate::audio::{lock_audio_state, AudioEngine, WebAudioEngine};
use crate::errors::AppError;
use crate::events::{
    AppEvent, EventBus, PlaybackProgressPayload, PlaybackStatePayload, TrackStartedPayload,
};
use crate::media_controls::{update_media_controls_metadata, update_media_controls_playback};
use crate::models::Track;
use crate::state::app_state::AppState;
use crate::state::playback_state::{
    current_playback_snapshot, metadata_from_track, should_advance_track, with_audio_state,
};
use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Manager};

fn emit_and_dispatch(app_handle: &AppHandle, event: AppEvent) -> Result<(), AppError> {
    EventBus::emit(app_handle, event.clone())
}

pub struct PlaybackService {
    app_handle: AppHandle,
}

impl PlaybackService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn play_track(&self, track: Track, index: Option<usize>) -> Result<(), AppError> {
        let file_path = Path::new(&track.path);

        if !file_path.exists() {
            return Err("音频文件不存在或已被移动".into());
        }

        let mut state = lock_audio_state()?;

        let volume = state.volume;
        let pan = state.pan;
        state.engine = None;

        let audio_processor_registry = self
            .app_handle
            .try_state::<AppState>()
            .map(|s| s.audio_processor_registry())
            .unwrap_or_else(|| std::sync::Arc::new(crate::services::plugin::AudioProcessorRegistry::new()));

        let mut engine = match WebAudioEngine::new(file_path, volume, pan, audio_processor_registry) {
            Ok(eng) => eng,
            Err(e) => {
                state.current_track_id = None;
                state.playing = false;
                return Err(e);
            }
        };

        if let Err(e) = engine.play() {
            state.current_track_id = None;
            state.playing = false;
            return Err(e);
        }

        state.engine = Some(Box::new(engine));
        state.current_track_id = Some(track.id);

        if let Some(next_index) = index {
            state.playback_queue.current_index = Some(next_index);
        }

        state.playing = true;
        let target_index = index.or(state.playback_queue.current_index).unwrap_or(0);

        drop(state);

        Self::emit_track_started(&self.app_handle, track, target_index)?;

        Ok(())
    }

    pub fn play_queue_index(&self, index: usize) -> Result<(), AppError> {
        let track = with_audio_state(|state| state.playback_queue.require_track(index))??;
        self.play_track(track, Some(index))
    }

    pub async fn next(&self) -> Result<(), AppError> {
        let (track, index) = with_audio_state(|state| {
            let index = state
                .playback_queue
                .move_next()
                .ok_or_else(|| AppError::from("Queue is empty"))?;

            let track = state.playback_queue.require_track(index)?;

            Ok::<(Track, usize), AppError>((track, index))
        })??;

        self.play_track(track, Some(index))
    }

    pub async fn previous(&self) -> Result<(), AppError> {
        let (track, index) = with_audio_state(|state| {
            let index = state
                .playback_queue
                .move_previous()
                .ok_or_else(|| AppError::from("Queue is empty"))?;

            let track = state.playback_queue.require_track(index)?;

            Ok::<(Track, usize), AppError>((track, index))
        })??;

        self.play_track(track, Some(index))
    }

    pub async fn resume(&self) -> Result<(), AppError> {
        let current_time = with_audio_state(|state| {
            let res = state
                .with_engine(|engine| {
                    engine.play()?;
                    Ok(engine.current_time())
                })
                .and_then(|inner| inner);

            if res.is_ok() {
                state.playing = true;
            }
            res
        })??;

        Self::sync_playback_state(&self.app_handle, true, current_time)?;
        Ok(())
    }

    pub async fn pause(&self) -> Result<(), AppError> {
        let current_time = with_audio_state(|state| {
            let res = state.with_engine(|engine| {
                engine.pause();
                engine.current_time()
            });
            if res.is_ok() {
                state.playing = false;
            }
            res
        })??;

        Self::sync_playback_state(&self.app_handle, false, current_time)?;
        Ok(())
    }

    pub async fn toggle(&self) -> Result<bool, AppError> {
        let (playing, current_time) = with_audio_state(|state| {
            let res = state
                .with_engine(|engine| {
                    if engine.paused() {
                        engine.play()?;
                        Ok((true, engine.current_time()))
                    } else {
                        engine.pause();
                        Ok((false, engine.current_time()))
                    }
                })
                .and_then(|i| i);
            if let Ok((p, _)) = res {
                state.playing = p;
            }
            res
        })??;

        Self::sync_playback_state(&self.app_handle, playing, current_time)?;
        Ok(playing)
    }

    pub async fn stop(&self) -> Result<(), AppError> {
        with_audio_state(|state| {
            let _ = state.with_engine(|engine| {
                engine.pause();
                engine.seek(0.0);
            });
            state.playing = false;
        })?;

        Self::sync_playback_state(&self.app_handle, false, 0.0)?;
        Ok(())
    }

    pub async fn seek(&self, time: f64) -> Result<(), AppError> {
        let playing = with_audio_state(|state| {
            let res = state.with_engine(|engine| {
                engine.seek(time);
                !engine.paused()
            });
            if let Ok(p) = res {
                state.playing = p;
            }
            res
        })??;

        Self::sync_playback_state(&self.app_handle, playing, time)?;
        Ok(())
    }

    pub fn emit_track_started(
        app_handle: &AppHandle,
        track: Track,
        index: usize,
    ) -> Result<(), AppError> {
        update_media_controls_metadata(metadata_from_track(&track));

        emit_and_dispatch(
            app_handle,
            AppEvent::TrackStarted(TrackStartedPayload { track, index }),
        )
    }

    pub fn sync_playback_state(
        app_handle: &AppHandle,
        playing: bool,
        current_time: f64,
    ) -> Result<(), AppError> {
        let payload = PlaybackStatePayload {
            playing,
            current_time,
        };

        update_media_controls_playback(payload.clone());

        emit_and_dispatch(app_handle, AppEvent::PlaybackStateChanged(payload))
    }

    pub fn emit_progress(app_handle: &AppHandle) -> Result<(), AppError> {
        if let Ok(snapshot) = current_playback_snapshot() {
            emit_and_dispatch(
                app_handle,
                AppEvent::PlaybackProgress(PlaybackProgressPayload {
                    current_time: snapshot.current_time,
                }),
            )?;
        }
        Ok(())
    }

    pub fn spawn_playback_progress_task(app_handle: AppHandle) {
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));

            loop {
                interval.tick().await;

                let _ = Self::emit_progress(&app_handle);

                if should_advance_track() {
                    let _ = PlaybackService::new(app_handle.clone()).next().await;
                }
            }
        });
    }

    pub(crate) async fn remove_track_from_queue(&self, track_id: i64) -> Result<(), AppError> {
        let result = {
            let mut state = lock_audio_state()?;
            state.playback_queue.remove_track(track_id)
        };

        let Some(remove_result) = result else {
            return Ok(());
        };

        if remove_result.should_stop {
            self.stop().await?;
        } else if let Some(index) = remove_result.play_index {
            self.play_queue_index(index)?;
        }

        Self::emit_queue_changed(&self.app_handle)?;

        Ok(())
    }

    pub(crate) fn emit_queue_changed(app_handle: &tauri::AppHandle) -> Result<(), AppError> {
        let queue = with_audio_state(|state| state.playback_queue.clone())?;

        emit_and_dispatch(app_handle, AppEvent::QueueChanged(queue))
    }
}
