use plugin_sdk::abi::AudioProcessorAbi;
use plugin_sdk::guest::{GuestPlugin, Host};
use plugin_sdk::PluginResult;
use plugin_sdk::guest_plugin;
use serde_json::Value;
use std::ffi::c_void;

const GAIN: f32 = 0.8;

unsafe extern "C" fn process_audio(
    _handle: *mut c_void,
    input: *const f32,
    output: *mut f32,
    frames: usize,
    channels: usize,
) -> i32 {
    let total = frames.checked_mul(channels).unwrap_or(0);
    if input.is_null() || output.is_null() || total == 0 {
        return -1;
    }
    let in_slice = std::slice::from_raw_parts(input, total);
    let out_slice = std::slice::from_raw_parts_mut(output, total);
    for (i, &s) in in_slice.iter().enumerate() {
        out_slice[i] = s * GAIN;
    }
    0
}

static AUDIO_PROCESSOR: AudioProcessorAbi = AudioProcessorAbi {
    init: None,
    process: Some(process_audio),
    reset: None,
};

#[derive(Default)]
struct AudioGain;

impl GuestPlugin for AudioGain {
    fn descriptor(&self) -> Value {
        serde_json::json!({
            "id": "audio-gain",
            "version": "0.1.0",
            "minHost": "0.1.0",
            "abi": plugin_sdk::ABI_VERSION,
            "displayName": "Audio Gain",
            "summary": "示例音频插件：将音量降低 20%（验证实时音频路径走插件 ABI）",
            "capabilities": ["audio.process"],
            "dependsOn": [],
            "optionalDependsOn": []
        })
    }

    fn activate(&self, _host: &Host) -> PluginResult<Value> {
        Ok(serde_json::json!({
            "contributions": [],
            "subscriptions": [],
            "serviceIds": []
        }))
    }

    fn command(&self, _host: &Host, command: &str, _args: &Value) -> PluginResult<Value> {
        Err(plugin_sdk::PluginError::not_found(format!(
            "audio-gain 无命令 '{command}'"
        )))
    }
}

guest_plugin!(AudioGain, audio_processor: &AUDIO_PROCESSOR);
