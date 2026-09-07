use std::sync::{LazyLock, RwLock};

use tauri::{AppHandle, Emitter};

use crate::errors::AppError;
use crate::events::payloads::{AppEvent, LyricsLoadedPayload};

pub type EventListener = Box<dyn Fn(&AppEvent) + Send + Sync>;

static LISTENERS: LazyLock<RwLock<Vec<EventListener>>> =
    LazyLock::new(|| RwLock::new(Vec::new()));

pub struct EventBus;

impl EventBus {
    pub const CHANNEL: &'static str = "global-app-event";

    pub fn subscribe(listener: EventListener) {
        if let Ok(mut lock) = LISTENERS.write() {
            lock.push(listener);
        }
    }

    pub fn emit(app_handle: &AppHandle, event: AppEvent) -> Result<(), AppError> {
        Self::notify_local(&event);
        app_handle.emit(Self::CHANNEL, event)?;
        Ok(())
    }

    pub fn emit_local_only(event: &AppEvent) {
        Self::notify_local(event);
    }

    pub fn emit_lyrics_loaded(
        app_handle: &AppHandle,
        payload: LyricsLoadedPayload,
    ) -> Result<(), AppError> {
        Self::emit(app_handle, AppEvent::LyricsLoaded(payload))
    }

    fn notify_local(event: &AppEvent) {
        let listeners = match LISTENERS.read() {
            Ok(lock) => lock,
            Err(_) => return,
        };
        for listener in listeners.iter() {
            listener(event);
        }
    }
}
