pub mod commands;
pub mod models;
use commands::audio::{self, PlayerState};
use commands::file;
use web_audio_api::context::AudioContext;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = AudioContext::default();
    let player_state = PlayerState::new();

    tauri::Builder::default()
        .manage(context)
        .manage(player_state)
        .plugin(tauri_plugin_dialog::init())
        // .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            audio::play_music,
            audio::play_music_from_path,
            audio::pause_music,
            audio::resume_music,
            audio::stop_music,
            file::select_music_folder,
            file::delete_file,
            file::open_containing_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
