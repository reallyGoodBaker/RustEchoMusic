use crate::audio::engine::AudioEngine;
use crate::audio::get_audio_context;
use crate::errors::AppError;
use crate::services::plugin::audio_processor_registry::AudioProcessorRegistry;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use web_audio_api::{
    context::BaseAudioContext,
    node::{AudioNode, AudioNodeOptions, BiquadFilterNode, BiquadFilterType, GainNode, MediaElementAudioSourceNode, StereoPannerNode},
    worklet::{AudioParamValues, AudioWorkletGlobalScope, AudioWorkletNode, AudioWorkletNodeOptions, AudioWorkletProcessor},
    MediaElement,
};

const EQ_FREQUENCIES: [f32; 10] = [
    31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];
const EQ_Q: f32 = 1.41;
const EQ_BAND_COUNT: usize = 10;

struct ExternalProcessorNode {
    registry: Arc<AudioProcessorRegistry>,
    in_buf: Vec<f32>,
    out_buf: Vec<f32>,
}

impl AudioWorkletProcessor for ExternalProcessorNode {
    type ProcessorOptions = Arc<AudioProcessorRegistry>;

    fn constructor(registry: Self::ProcessorOptions) -> Self
    where
        Self: Sized,
    {
        Self {
            registry,
            in_buf: Vec::new(),
            out_buf: Vec::new(),
        }
    }

    fn process<'a, 'b>(
        &mut self,
        inputs: &'b [&'a [&'a [f32]]],
        outputs: &'b mut [&'a mut [&'a mut [f32]]],
        _params: AudioParamValues<'b>,
        _scope: &'b AudioWorkletGlobalScope,
    ) -> bool {
        // 单输入单输出节点：inputs[0] / outputs[0] 为各通道的样本切片。
        let input: &[&[f32]] = inputs.get(0).copied().unwrap_or(&[]);
        let channels = input.len();
        let frames = input.first().map(|c| c.len()).unwrap_or(0);

        let processors = self.registry.iter_active();
        if processors.is_empty() {
            // 无插件：透传（input → output）。
            if let Some(output) = outputs.get_mut(0) {
                for (i, out_chan) in output.iter_mut().enumerate() {
                    if let Some(in_chan) = input.get(i) {
                        let len = in_chan.len().min(out_chan.len());
                        if len > 0 {
                            out_chan[..len].copy_from_slice(&in_chan[..len]);
                        }
                    }
                }
            }
            return true;
        }

        if channels == 0 || frames == 0 {
            // 无输入数据；输出保持静默。
            return true;
        }

        // 构造交错缓冲区：[L0, R0, L1, R1, ...]（frames * channels 个 f32）。
        let total = frames * channels;
        self.in_buf.clear();
        self.in_buf.reserve(total);
        for f in 0..frames {
            for c in 0..channels {
                let v = input
                    .get(c)
                    .and_then(|ch| ch.get(f))
                    .copied()
                    .unwrap_or(0.0);
                self.in_buf.push(v);
            }
        }
        // out_buf 初值 = in_buf（透传给首个插件作为输出缓冲）。
        self.out_buf.clear();
        self.out_buf.extend_from_slice(&self.in_buf);

        // 依次调用各 processor 的 process FFI。成功后用 out_buf 作为下一轮输入。
        for (_plugin_id, api, handle) in processors {
            let Some(process_fn) = api.process else {
                continue;
            };
            let rc = unsafe {
                process_fn(
                    handle,
                    self.in_buf.as_ptr(),
                    self.out_buf.as_mut_ptr(),
                    frames,
                    channels,
                )
            };
            if rc == 0 {
                // 处理成功：用 out_buf 作为下一轮插件输入。
                self.in_buf[..total].copy_from_slice(&self.out_buf);
            }
            // 失败：跳过该插件，保留 out_buf / in_buf 不变。
        }

        // 解交错写回 outputs。
        if let Some(output) = outputs.get_mut(0) {
            for c in 0..channels {
                if let Some(out_chan) = output.get_mut(c) {
                    let n = out_chan.len().min(frames);
                    for f in 0..n {
                        out_chan[f] = self.out_buf[f * channels + c];
                    }
                }
            }
        }

        // 返回 true：节点常驻（即使输入断开，仍保留在图中以便随时接收插件输出）。
        true
    }
}

pub struct WebAudioEngine {
    media: MediaElement,
    src: MediaElementAudioSourceNode,
    gain: GainNode,
    panner: StereoPannerNode,
    eq_filters: Vec<BiquadFilterNode>,
    external_processor_node: AudioWorkletNode,
    eq_enabled: bool,
    eq_bands: [f64; 10],
}

impl WebAudioEngine {
    pub fn new(
        file_path: &Path,
        volume: f32,
        pan: f32,
        audio_processor_registry: Arc<AudioProcessorRegistry>,
    ) -> Result<Self, AppError> {
        let context = get_audio_context();
        let _ = context.resume();

        // 把当前音频输出格式同步给 registry，供插件 init 回调使用。
        audio_processor_registry.set_audio_format(context.sample_rate() as u32, 2);

        let mut media = web_audio_api::MediaElement::new(file_path)
            .map_err(|e| format!("Failed to create media element: {}", e))?;

        let src = context.create_media_element_source(&mut media);
        let gain = context.create_gain();
        let panner = context.create_stereo_panner();

        // 构建 10 段 EQ BiquadFilter 链（始终串联，bypass 时 gain 全为 0dB，不改变频响）
        let mut eq_filters: Vec<BiquadFilterNode> = Vec::with_capacity(EQ_BAND_COUNT);
        for &freq in EQ_FREQUENCIES.iter() {
            let mut filter = context.create_biquad_filter();
            filter.set_type(BiquadFilterType::Peaking);
            filter.frequency().set_value(freq);
            filter.q().set_value(EQ_Q);
            // 初始处于旁路状态：gain = 0 dB
            filter.gain().set_value(0.0);
            eq_filters.push(filter);
        }

        // 外部处理器节点（AudioWorklet）。1 入 1 出，output_channel_count 为空时
        // 继承输入通道数（stereo）。
        let worklet_options = AudioWorkletNodeOptions {
            number_of_inputs: 1,
            number_of_outputs: 1,
            output_channel_count: Vec::new(),
            parameter_data: HashMap::new(),
            processor_options: audio_processor_registry,
            audio_node_options: AudioNodeOptions::default(),
        };
        let external_processor_node =
            AudioWorkletNode::new::<ExternalProcessorNode>(&context, worklet_options);

        // 节点链：src → biquad[0] → ... → biquad[9] → external_processor_node → gain → panner → destination
        src.connect(&eq_filters[0]);
        for i in 0..(EQ_BAND_COUNT - 1) {
            eq_filters[i].connect(&eq_filters[i + 1]);
        }
        eq_filters[EQ_BAND_COUNT - 1].connect(&external_processor_node);
        external_processor_node.connect(&gain);
        gain.connect(&panner);
        panner.connect(&context.destination());

        media.set_loop(false);
        media.set_current_time(0.0);
        gain.gain().set_value(volume);
        panner.pan().set_value(pan);

        Ok(WebAudioEngine {
            media,
            src,
            gain,
            panner,
            eq_filters,
            external_processor_node,
            eq_enabled: false,
            eq_bands: [0.0; 10],
        })
    }
}

impl AudioEngine for WebAudioEngine {
    fn play(&mut self) -> Result<(), AppError> {
        self.media.play();
        Ok(())
    }

    fn pause(&mut self) {
        self.media.pause();
    }

    fn seek(&mut self, time: f64) {
        self.media.set_current_time(time);
    }

    fn set_volume(&self, volume: f32) {
        self.gain.gain().set_value(volume);
    }

    fn set_pan(&self, pan: f32) {
        self.panner.pan().set_value(pan);
    }

    fn current_time(&self) -> f64 {
        self.media.current_time()
    }

    fn paused(&self) -> bool {
        self.media.paused()
    }

    fn set_eq_band_gain(&mut self, band_index: usize, gain_db: f64) -> Result<(), AppError> {
        if band_index >= EQ_BAND_COUNT {
            return Err(AppError::Service(format!(
                "EQ band index {} out of range (0..{})",
                band_index, EQ_BAND_COUNT
            )));
        }
        self.eq_bands[band_index] = gain_db;
        // 仅在 EQ 启用时同步到节点，旁路时保持 0dB
        if self.eq_enabled {
            self.eq_filters[band_index].gain().set_value(gain_db as f32);
        }
        Ok(())
    }

    fn apply_eq_preset(&mut self, gains: &[f64; 10]) -> Result<(), AppError> {
        self.eq_bands = *gains;
        if self.eq_enabled {
            for (i, &g) in gains.iter().enumerate() {
                self.eq_filters[i].gain().set_value(g as f32);
            }
        }
        Ok(())
    }

    fn set_eq_enabled(&mut self, enabled: bool) -> Result<(), AppError> {
        self.eq_enabled = enabled;
        if enabled {
            // 启用时恢复各频段目标增益
            for (i, &g) in self.eq_bands.iter().enumerate() {
                self.eq_filters[i].gain().set_value(g as f32);
            }
        } else {
            // 旁路时所有 biquad gain 设为 0dB（不改变频响）
            for filter in &self.eq_filters {
                filter.gain().set_value(0.0);
            }
        }
        Ok(())
    }

    fn get_eq_bands(&self) -> Result<[f64; 10], AppError> {
        Ok(self.eq_bands)
    }

    fn is_eq_enabled(&self) -> Result<bool, AppError> {
        Ok(self.eq_enabled)
    }
}

impl Drop for WebAudioEngine {
    fn drop(&mut self) {
        let _ = self.panner.disconnect();
        let _ = self.gain.disconnect();
        let _ = self.external_processor_node.disconnect();
        for filter in &self.eq_filters {
            let _ = filter.disconnect();
        }
        let _ = self.src.disconnect();
        self.media.pause();
    }
}
