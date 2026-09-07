pub mod engine;
pub mod state;
pub mod web_audio;

pub use engine::AudioEngine;
pub use state::{init_audio_state, lock_audio_state, AudioState};
pub use web_audio::WebAudioEngine;
use web_audio_api::context::AudioContext;

pub fn get_audio_context() -> AudioContext {
    AudioContext::default()
}
