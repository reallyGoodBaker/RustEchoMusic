use crate::services::playback_service::PlaybackService;
use crate::state::playback_state::current_time_from_state;
use souvlaki::{MediaControlEvent, SeekDirection};
use tauri::AppHandle;

pub async fn handle_media_control_event(app_handle: AppHandle, event: MediaControlEvent) {
    let service = PlaybackService::new(app_handle);

    match event {
        MediaControlEvent::Play => {
            let _ = service.resume().await;
        }
        MediaControlEvent::Pause => {
            let _ = service.pause().await;
        }
        MediaControlEvent::Toggle => {
            let _ = service.toggle().await;
        }
        MediaControlEvent::Next => {
            let _ = service.next().await;
        }
        MediaControlEvent::Previous => {
            let _ = service.previous().await;
        }
        MediaControlEvent::Stop => {
            let _ = service.stop().await;
        }
        MediaControlEvent::SetPosition(position) => {
            let _ = service.seek(position.0.as_secs_f64()).await;
        }
        MediaControlEvent::Seek(direction) => {
            let current_time = current_time_from_state();
            let offset = match direction {
                SeekDirection::Forward => 10.0,
                SeekDirection::Backward => -10.0,
            };
            let _ = service.seek((current_time + offset).max(0.0)).await;
        }
        MediaControlEvent::SeekBy(direction, duration) => {
            let current_time = current_time_from_state();
            let offset = duration.as_secs_f64();
            let next_time = match direction {
                SeekDirection::Forward => current_time + offset,
                SeekDirection::Backward => current_time - offset,
            };
            let _ = service.seek(next_time.max(0.0)).await;
        }
        _ => {}
    }
}
