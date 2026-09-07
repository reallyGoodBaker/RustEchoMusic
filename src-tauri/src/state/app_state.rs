use std::sync::Arc;

use plugin_runtime::PluginRuntime;
use crate::services::plugin::AudioProcessorRegistry;
use crate::services::{PlaylistService, SettingsService, TrackService};

#[derive(Clone)]
pub struct AppState {
    pub tracks: Arc<TrackService>,
    pub playlists: Arc<PlaylistService>,
    pub settings: Arc<SettingsService>,
    pub plugins: Arc<PluginRuntime>,
    pub audio_processor_registry: Arc<AudioProcessorRegistry>,
}

impl AppState {
    pub fn new(
        tracks: Arc<TrackService>,
        playlists: Arc<PlaylistService>,
        settings: Arc<SettingsService>,
        plugins: Arc<PluginRuntime>,
        audio_processor_registry: Arc<AudioProcessorRegistry>,
    ) -> Self {
        Self {
            tracks,
            playlists,
            settings,
            plugins,
            audio_processor_registry,
        }
    }

    pub fn audio_processor_registry(&self) -> Arc<AudioProcessorRegistry> {
        Arc::clone(&self.audio_processor_registry)
    }
}
