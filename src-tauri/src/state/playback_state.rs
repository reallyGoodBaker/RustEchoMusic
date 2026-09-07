pub use crate::audio::lock_audio_state;
use crate::audio::AudioState;
use crate::errors::AppError;
use crate::models::playback::{NativeTrackMetadata, PlaybackStatusSnapshot};
use crate::models::Track;

pub fn with_audio_state<T>(f: impl FnOnce(&mut AudioState) -> T) -> Result<T, AppError> {
    let value = {
        let mut state = lock_audio_state()?;
        f(&mut state)
    };

    Ok(value)
}

pub fn current_playback_state(state: &mut AudioState) -> PlaybackStatusSnapshot {
    match state.with_engine(|engine| (!engine.paused(), engine.current_time())) {
        Ok((playing, current_time)) => {
            state.playing = playing;

            PlaybackStatusSnapshot {
                playing,
                current_time,
            }
        }

        Err(_) => {
            state.playing = false;

            PlaybackStatusSnapshot {
                playing: false,
                current_time: 0.0,
            }
        }
    }
}

pub fn current_playback_snapshot() -> Result<PlaybackStatusSnapshot, AppError> {
    with_audio_state(current_playback_state)
}

pub fn current_time_from_state() -> f64 {
    current_playback_snapshot()
        .map(|state| state.current_time)
        .unwrap_or(0.0)
}

pub fn current_track_duration(state: &AudioState) -> Option<f64> {
    let index = state.playback_queue.current_index?;
    state
        .playback_queue
        .tracks
        .get(index)
        .map(|track| track.duration as f64 / 1000.0)
}

pub fn should_advance_track() -> bool {
    let Ok((snapshot, duration)) = with_audio_state(|state| {
        let snapshot = current_playback_state(state);
        let duration = current_track_duration(state);
        (snapshot, duration)
    }) else {
        return false;
    };

    snapshot.playing
        && duration
            .map(|duration| duration > 0.0 && snapshot.current_time >= duration - 0.5)
            .unwrap_or(false)
}

pub fn metadata_from_track(track: &Track) -> NativeTrackMetadata {
    NativeTrackMetadata {
        title: track.title.clone(),
        album: track
            .album
            .clone()
            .unwrap_or_else(|| "未知专辑".to_string()),
        artist: track
            .artist
            .clone()
            .unwrap_or_else(|| "未知歌手".to_string()),
        duration: Some(track.duration as f64 / 1000.0),
    }
}

pub fn sanitize_track(mut track: Track) -> Track {
    track.cover = None;
    track
}
