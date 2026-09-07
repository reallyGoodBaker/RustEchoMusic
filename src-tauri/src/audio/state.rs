use crate::audio::engine::AudioEngine;
use crate::errors::AppError;
use crate::models::PlaybackQueue;
use std::sync::{Mutex, MutexGuard, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayMode {
    ListLoop,
    SingleLoop,
    Shuffle,
}

pub struct AudioState {
    pub engine: Option<Box<dyn AudioEngine + Send>>,
    pub volume: f32,
    pub pan: f32,
    pub playing: bool,
    pub current_track_id: Option<i64>,
    pub playback_queue: PlaybackQueue,
    pub eq_bands: [f64; 10],
    pub eq_enabled: bool,
}

impl AudioState {
    pub fn with_engine<F, R>(&mut self, f: F) -> Result<R, AppError>
    where
        F: FnOnce(&mut Box<dyn AudioEngine + Send>) -> R,
    {
        if let Some(ref mut engine) = self.engine {
            Ok(f(engine))
        } else {
            Err(AppError::from("Audio engine is missing"))
        }
    }

    pub fn play_mode(&self) -> PlayMode {
        self.playback_queue.play_mode
    }
    pub fn set_play_mode(&mut self, mode: PlayMode) {
        self.playback_queue.play_mode = mode;
    }
}

static AUDIO_STATE: OnceLock<Mutex<AudioState>> = OnceLock::new();

pub fn init_audio_state(queue: PlaybackQueue) {
    AUDIO_STATE.get_or_init(|| {
        Mutex::new(AudioState {
            engine: None,
            volume: 1.0,
            pan: 0.0,
            playing: false,
            current_track_id: None,
            playback_queue: queue,
            eq_bands: [0.0; 10],
            eq_enabled: false,
        })
    });
}

pub fn lock_audio_state() -> Result<MutexGuard<'static, AudioState>, AppError> {
    AUDIO_STATE
        .get()
        .ok_or_else(|| AppError::from("Audio state not initialized"))?
        .lock()
        .map_err(|_| AppError::from("Failed to lock audio state"))
}
