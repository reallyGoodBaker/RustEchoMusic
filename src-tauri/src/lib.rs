use std::fs::File;

use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

#[tauri::command]
fn play_music(name: &str) -> String {
    let context = AudioContext::default();

    let current_dir = std::env::current_dir().unwrap();
    let file_path = current_dir.join("..").join("static").join(name);

    let file = File::open(file_path).unwrap();
    let buffer = context.decode_audio_data_sync(file).unwrap();

    let mut src = context.create_buffer_source();
    src.set_buffer(buffer);
    src.set_loop(false);

    // // create a biquad filter
    // let biquad = context.create_biquad_filter();
    // biquad.frequency().set_value(125.);
    // connect the audio nodes
    // src.connect(&biquad);
    // biquad.connect(&context.destination());

    src.connect(&context.destination());

    src.start();

    let buffer_duration = src.buffer().unwrap().duration() as f64;
    let sleep_duration = std::time::Duration::from_secs_f64(buffer_duration);
    std::thread::sleep(sleep_duration);

    format!("Playing music: {}", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![play_music])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
