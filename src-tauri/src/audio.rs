use crate::app_log;
use crate::config::AudioConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig, SupportedStreamConfig};
use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const ASR_OUTPUT_SAMPLE_RATE: u32 = 16_000;
pub const ASR_OUTPUT_CHANNELS: u16 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct AudioCaptureInfo {
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub chunks: usize,
    pub pcm_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioDeviceInfo {
    pub index: u32,
    pub name: String,
    pub is_default: bool,
}

#[derive(Clone)]
struct CaptureCounters {
    chunks: Arc<AtomicUsize>,
    pcm_bytes: Arc<AtomicUsize>,
}

struct CaptureOutputs {
    chunk_tx: Option<mpsc::Sender<Vec<u8>>>,
    level_tx: Option<mpsc::Sender<f32>>,
    silence_tx: Option<mpsc::Sender<()>>,
    silence_auto_stop_seconds: u64,
    silence_level_threshold: f32,
}

pub struct AudioCapture {
    stop_tx: mpsc::Sender<()>,
    join_handle: Option<JoinHandle<()>>,
    device_name: String,
    sample_rate: u32,
    channels: u16,
    counters: CaptureCounters,
}

impl AudioCapture {
    pub fn info(&self) -> AudioCaptureInfo {
        AudioCaptureInfo {
            device_name: self.device_name.clone(),
            sample_rate: self.sample_rate,
            channels: self.channels,
            chunks: self.counters.chunks.load(Ordering::Relaxed),
            pcm_bytes: self.counters.pcm_bytes.load(Ordering::Relaxed),
        }
    }
}

pub fn start_capture(
    audio: &AudioConfig,
    chunk_tx: Option<mpsc::Sender<Vec<u8>>>,
    level_tx: Option<mpsc::Sender<f32>>,
    silence_tx: Option<mpsc::Sender<()>>,
) -> Result<AudioCapture, String> {
    let audio = audio.clone();
    let counters = CaptureCounters {
        chunks: Arc::new(AtomicUsize::new(0)),
        pcm_bytes: Arc::new(AtomicUsize::new(0)),
    };
    let worker_counters = counters.clone();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();

    let join_handle = thread::spawn(move || {
        let outputs = CaptureOutputs {
            chunk_tx,
            level_tx,
            silence_tx,
            silence_auto_stop_seconds: audio.silence_auto_stop_seconds,
            silence_level_threshold: audio.silence_level_threshold,
        };
        let (stream, device_name, sample_rate, channels) =
            match start_capture_in_thread(&audio, worker_counters, outputs) {
                Ok(result) => result,
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
        if ready_tx
            .send(Ok((device_name, sample_rate, channels)))
            .is_err()
        {
            return;
        }
        let _ = stop_rx.recv();
        drop(stream);
    });

    let (device_name, sample_rate, channels) = ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "启动麦克风采集超时".to_string())??;

    Ok(AudioCapture {
        stop_tx,
        join_handle: Some(join_handle),
        device_name,
        sample_rate,
        channels,
        counters,
    })
}

pub fn list_input_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|device| device.description().ok())
        .map(|description| description.name().to_string());
    let devices = host
        .input_devices()
        .map_err(|err| format!("枚举输入设备失败: {}", err))?;
    Ok(devices
        .enumerate()
        .map(|(index, device)| {
            let name = device
                .description()
                .map(|description| description.name().to_string())
                .unwrap_or_else(|_| format!("Input device {}", index));
            AudioDeviceInfo {
                index: index as u32,
                is_default: default_name.as_deref() == Some(name.as_str()),
                name,
            }
        })
        .collect())
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

fn start_capture_in_thread(
    audio: &AudioConfig,
    counters: CaptureCounters,
    outputs: CaptureOutputs,
) -> Result<(Stream, String, u32, u16), String> {
    let host = cpal::default_host();
    let device = select_input_device(&host, audio.input_device)?;
    let device_name = device
        .description()
        .map(|description| description.name().to_string())
        .unwrap_or_else(|_| "Unknown input device".to_string());
    let supported = select_input_config(&device, audio)?;
    let sample_format = supported.sample_format();
    let stream_config = StreamConfig {
        channels: supported.channels(),
        sample_rate: supported.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };
    let target_chunk_bytes = target_chunk_bytes(
        ASR_OUTPUT_SAMPLE_RATE,
        ASR_OUTPUT_CHANNELS,
        audio.segment_ms,
    );
    if stream_config.sample_rate != ASR_OUTPUT_SAMPLE_RATE
        || stream_config.channels != ASR_OUTPUT_CHANNELS
    {
        app_log::info(format!(
            "麦克风输入将转换为豆包 ASR 支持格式: input={}Hz/{}ch, output={}Hz/{}ch",
            stream_config.sample_rate,
            stream_config.channels,
            ASR_OUTPUT_SAMPLE_RATE,
            ASR_OUTPUT_CHANNELS
        ));
    }
    let err_fn = |err| app_log::warn(format!("audio input stream error: {}", err));
    let stream = match sample_format {
        SampleFormat::I16 => build_i16_stream(
            &device,
            &stream_config,
            target_chunk_bytes,
            counters.clone(),
            outputs,
            err_fn,
        )?,
        SampleFormat::U16 => build_u16_stream(
            &device,
            &stream_config,
            target_chunk_bytes,
            counters.clone(),
            outputs,
            err_fn,
        )?,
        SampleFormat::U8 => build_u8_stream(
            &device,
            &stream_config,
            target_chunk_bytes,
            counters.clone(),
            outputs,
            err_fn,
        )?,
        SampleFormat::F32 => build_f32_stream(
            &device,
            &stream_config,
            target_chunk_bytes,
            counters.clone(),
            outputs,
            err_fn,
        )?,
        other => return Err(format!("暂不支持的输入采样格式: {:?}", other)),
    };
    stream
        .play()
        .map_err(|err| format!("启动麦克风采集失败: {}", err))?;
    Ok((
        stream,
        device_name,
        ASR_OUTPUT_SAMPLE_RATE,
        ASR_OUTPUT_CHANNELS,
    ))
}

fn select_input_device(host: &cpal::Host, input_device: Option<u32>) -> Result<Device, String> {
    if let Some(index) = input_device {
        return host
            .input_devices()
            .map_err(|err| format!("枚举输入设备失败: {}", err))?
            .nth(index as usize)
            .ok_or_else(|| format!("找不到配置中的输入设备: {}", index));
    }
    host.default_input_device()
        .ok_or_else(|| "未找到默认麦克风输入设备".to_string())
}

fn select_input_config(
    device: &Device,
    audio: &AudioConfig,
) -> Result<SupportedStreamConfig, String> {
    let target_rate = audio.sample_rate;
    let mut fallback = None;
    for range in device
        .supported_input_configs()
        .map_err(|err| format!("读取麦克风采样配置失败: {}", err))?
    {
        if fallback.is_none() {
            fallback = Some(range.with_max_sample_rate());
        }
        if range.channels() == audio.channels
            && range.min_sample_rate() <= target_rate
            && target_rate <= range.max_sample_rate()
        {
            return Ok(range.with_sample_rate(target_rate));
        }
    }
    fallback
        .or_else(|| device.default_input_config().ok())
        .ok_or_else(|| "麦克风没有可用采样配置".to_string())
}

fn target_chunk_bytes(sample_rate: u32, channels: u16, segment_ms: u64) -> usize {
    let frames = ((sample_rate as u64 * segment_ms.max(1)) / 1000).max(1);
    frames as usize * channels.max(1) as usize * 2
}

fn send_segmented_bytes(tx: &mpsc::Sender<Vec<u8>>, pending: &mut Vec<u8>, target_bytes: usize) {
    while pending.len() >= target_bytes {
        let chunk = pending.drain(..target_bytes).collect::<Vec<_>>();
        let _ = tx.send(chunk);
    }
}

fn send_level(tx: &Option<mpsc::Sender<f32>>, level: f32) {
    if let Some(tx) = tx {
        let _ = tx.send(level.clamp(0.0, 1.0));
    }
}

fn send_silence_auto_stop(
    tx: &Option<mpsc::Sender<()>>,
    silence: &mut SilenceAutoStopper,
    level: f32,
    frame_count: usize,
) {
    if let Some(tx) = tx {
        if silence.observe(level, frame_count) {
            let _ = tx.send(());
        }
    }
}

// 短促杂音不应打断静音计时，否则键盘声或碰麦会让空录音持续过久。
const SILENCE_RESET_CONFIRM_MS: u64 = 200;

struct SilenceAutoStopper {
    silence_frames: u64,
    limit_frames: u64,
    consecutive_active_frames: u64,
    active_reset_confirm_frames: u64,
    level_threshold: f32,
    triggered: bool,
}

impl SilenceAutoStopper {
    fn new(sample_rate: u32, seconds: u64, level_threshold: f32) -> Self {
        Self {
            silence_frames: 0,
            limit_frames: sample_rate as u64 * seconds,
            consecutive_active_frames: 0,
            active_reset_confirm_frames: (sample_rate as u64 * SILENCE_RESET_CONFIRM_MS / 1000)
                .max(1),
            level_threshold: level_threshold.clamp(0.001, 0.5),
            triggered: seconds == 0,
        }
    }

    fn observe(&mut self, level: f32, frame_count: usize) -> bool {
        if self.triggered || self.limit_frames == 0 || frame_count == 0 {
            return false;
        }
        if level <= self.level_threshold {
            self.silence_frames = self.silence_frames.saturating_add(frame_count as u64);
            self.consecutive_active_frames = 0;
        } else {
            self.consecutive_active_frames = self
                .consecutive_active_frames
                .saturating_add(frame_count as u64);
            if self.consecutive_active_frames >= self.active_reset_confirm_frames {
                self.silence_frames = 0;
                self.consecutive_active_frames = self.active_reset_confirm_frames;
            }
        }
        if self.silence_frames >= self.limit_frames {
            self.triggered = true;
            true
        } else {
            false
        }
    }
}

fn rms_i16(data: &[i16]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let sum = data
        .iter()
        .map(|sample| {
            let value = *sample as f32 / i16::MAX as f32;
            value * value
        })
        .sum::<f32>();
    (sum / data.len() as f32).sqrt().clamp(0.0, 1.0)
}

fn rms_u16(data: &[u16]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let sum = data
        .iter()
        .map(|sample| {
            let value = (*sample as f32 - 32768.0) / 32768.0;
            value * value
        })
        .sum::<f32>();
    (sum / data.len() as f32).sqrt().clamp(0.0, 1.0)
}

fn rms_u8(data: &[u8]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let sum = data
        .iter()
        .map(|sample| {
            let value = (*sample as f32 - 128.0) / 128.0;
            value * value
        })
        .sum::<f32>();
    (sum / data.len() as f32).sqrt().clamp(0.0, 1.0)
}

fn rms_f32(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let sum = data
        .iter()
        .map(|sample| {
            let value = sample.clamp(-1.0, 1.0);
            value * value
        })
        .sum::<f32>();
    (sum / data.len() as f32).sqrt().clamp(0.0, 1.0)
}

struct PcmNormalizer {
    source_rate: u32,
    source_channels: usize,
    target_rate: u32,
    phase: u32,
    downsample_sum: i64,
    downsample_count: u32,
}

impl PcmNormalizer {
    fn new(source_rate: u32, source_channels: usize, target_rate: u32) -> Self {
        Self {
            source_rate: source_rate.max(1),
            source_channels: source_channels.max(1),
            target_rate: target_rate.max(1),
            phase: 0,
            downsample_sum: 0,
            downsample_count: 0,
        }
    }

    fn push_i16(&mut self, data: &[i16], pending: &mut Vec<u8>) -> usize {
        let before = pending.len();
        for frame in data.chunks_exact(self.source_channels) {
            self.push_mono_sample(mix_i16_frame(frame), pending);
        }
        pending.len().saturating_sub(before)
    }

    fn push_u16(&mut self, data: &[u16], pending: &mut Vec<u8>) -> usize {
        let before = pending.len();
        for frame in data.chunks_exact(self.source_channels) {
            let sum = frame
                .iter()
                .map(|sample| *sample as i32 - 32768)
                .sum::<i32>();
            self.push_mono_sample(clamp_i32_to_i16(sum / frame.len() as i32), pending);
        }
        pending.len().saturating_sub(before)
    }

    fn push_u8(&mut self, data: &[u8], pending: &mut Vec<u8>) -> usize {
        let before = pending.len();
        for frame in data.chunks_exact(self.source_channels) {
            let sum = frame
                .iter()
                .map(|sample| (*sample as i32 - 128) << 8)
                .sum::<i32>();
            self.push_mono_sample(clamp_i32_to_i16(sum / frame.len() as i32), pending);
        }
        pending.len().saturating_sub(before)
    }

    fn push_f32(&mut self, data: &[f32], pending: &mut Vec<u8>) -> usize {
        let before = pending.len();
        for frame in data.chunks_exact(self.source_channels) {
            let avg = frame
                .iter()
                .map(|sample| sample.clamp(-1.0, 1.0))
                .sum::<f32>()
                / frame.len() as f32;
            self.push_mono_sample((avg * i16::MAX as f32) as i16, pending);
        }
        pending.len().saturating_sub(before)
    }

    fn push_mono_sample(&mut self, sample: i16, pending: &mut Vec<u8>) {
        if self.target_rate >= self.source_rate {
            self.push_upsampled_sample(sample, pending);
        } else {
            self.push_downsampled_sample(sample, pending);
        }
    }

    fn push_upsampled_sample(&mut self, sample: i16, pending: &mut Vec<u8>) {
        self.phase = self.phase.saturating_add(self.target_rate);
        while self.phase >= self.source_rate {
            pending.extend(sample.to_le_bytes());
            self.phase -= self.source_rate;
        }
    }

    fn push_downsampled_sample(&mut self, sample: i16, pending: &mut Vec<u8>) {
        self.downsample_sum += sample as i64;
        self.downsample_count += 1;
        self.phase = self.phase.saturating_add(self.target_rate);

        while self.phase >= self.source_rate {
            pending.extend(self.downsample_average().to_le_bytes());
            self.downsample_sum = 0;
            self.downsample_count = 0;
            self.phase -= self.source_rate;
        }
    }

    fn downsample_average(&self) -> i16 {
        if self.downsample_count == 0 {
            return 0;
        }
        clamp_i64_to_i16(self.downsample_sum / self.downsample_count as i64)
    }
}

fn mix_i16_frame(frame: &[i16]) -> i16 {
    let sum = frame.iter().map(|sample| *sample as i32).sum::<i32>();
    clamp_i32_to_i16(sum / frame.len().max(1) as i32)
}

fn clamp_i32_to_i16(value: i32) -> i16 {
    value.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn clamp_i64_to_i16(value: i64) -> i16 {
    value.clamp(i16::MIN as i64, i16::MAX as i64) as i16
}

fn build_i16_stream(
    device: &Device,
    config: &StreamConfig,
    target_chunk_bytes: usize,
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let mut pending = Vec::new();
    let mut normalizer = PcmNormalizer::new(config.sample_rate, channels, ASR_OUTPUT_SAMPLE_RATE);
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        chunk_tx,
        level_tx,
        silence_tx,
        ..
    } = outputs;
    device
        .build_input_stream(
            config,
            move |data: &[i16], _| {
                let frame_count = data.len() / channels;
                counters.chunks.fetch_add(1, Ordering::Relaxed);
                let level = rms_i16(data);
                send_level(&level_tx, level);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                if let Some(tx) = &chunk_tx {
                    let emitted = normalizer.push_i16(data, &mut pending);
                    counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
                    send_segmented_bytes(tx, &mut pending, target_chunk_bytes);
                }
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

fn build_u16_stream(
    device: &Device,
    config: &StreamConfig,
    target_chunk_bytes: usize,
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let mut pending = Vec::new();
    let mut normalizer = PcmNormalizer::new(config.sample_rate, channels, ASR_OUTPUT_SAMPLE_RATE);
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        chunk_tx,
        level_tx,
        silence_tx,
        ..
    } = outputs;
    device
        .build_input_stream(
            config,
            move |data: &[u16], _| {
                let frame_count = data.len() / channels;
                counters.chunks.fetch_add(1, Ordering::Relaxed);
                let level = rms_u16(data);
                send_level(&level_tx, level);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                if let Some(tx) = &chunk_tx {
                    let emitted = normalizer.push_u16(data, &mut pending);
                    counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
                    send_segmented_bytes(tx, &mut pending, target_chunk_bytes);
                }
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

fn build_u8_stream(
    device: &Device,
    config: &StreamConfig,
    target_chunk_bytes: usize,
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let mut pending = Vec::new();
    let mut normalizer = PcmNormalizer::new(config.sample_rate, channels, ASR_OUTPUT_SAMPLE_RATE);
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        chunk_tx,
        level_tx,
        silence_tx,
        ..
    } = outputs;
    device
        .build_input_stream(
            config,
            move |data: &[u8], _| {
                let frame_count = data.len() / channels;
                counters.chunks.fetch_add(1, Ordering::Relaxed);
                let level = rms_u8(data);
                send_level(&level_tx, level);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                if let Some(tx) = &chunk_tx {
                    let emitted = normalizer.push_u8(data, &mut pending);
                    counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
                    send_segmented_bytes(tx, &mut pending, target_chunk_bytes);
                }
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

fn build_f32_stream(
    device: &Device,
    config: &StreamConfig,
    target_chunk_bytes: usize,
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let mut pending = Vec::new();
    let mut normalizer = PcmNormalizer::new(config.sample_rate, channels, ASR_OUTPUT_SAMPLE_RATE);
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        chunk_tx,
        level_tx,
        silence_tx,
        ..
    } = outputs;
    device
        .build_input_stream(
            config,
            move |data: &[f32], _| {
                let frame_count = data.len() / channels;
                counters.chunks.fetch_add(1, Ordering::Relaxed);
                let level = rms_f32(data);
                send_level(&level_tx, level);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                if let Some(tx) = &chunk_tx {
                    let emitted = normalizer.push_f32(data, &mut pending);
                    counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
                    send_segmented_bytes(tx, &mut pending, target_chunk_bytes);
                }
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

#[cfg(test)]
mod tests {
    use super::{PcmNormalizer, SilenceAutoStopper, ASR_OUTPUT_CHANNELS, ASR_OUTPUT_SAMPLE_RATE};

    fn pcm_bytes_to_i16(bytes: &[u8]) -> Vec<i16> {
        bytes
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect()
    }

    #[test]
    fn silence_auto_stop_fires_after_configured_silent_audio_duration() {
        let mut stopper = SilenceAutoStopper::new(16_000, 10, 0.04);

        assert!(!stopper.observe(0.0, 16_000 * 9));
        assert!(stopper.observe(0.0, 16_000));
        assert!(!stopper.observe(0.0, 16_000));
    }

    #[test]
    fn silence_auto_stop_counts_low_background_noise_as_silence() {
        let mut stopper = SilenceAutoStopper::new(16_000, 10, 0.04);

        assert!(!stopper.observe(0.03, 16_000 * 9));
        assert!(stopper.observe(0.03, 16_000));
    }

    #[test]
    fn silence_auto_stop_resets_when_level_exceeds_threshold() {
        let mut stopper = SilenceAutoStopper::new(16_000, 10, 0.04);

        assert!(!stopper.observe(0.0, 16_000 * 8));
        assert!(!stopper.observe(0.05, 16_000));
        assert!(!stopper.observe(0.0, 16_000 * 9));
        assert!(stopper.observe(0.0, 16_000));
    }

    #[test]
    fn silence_auto_stop_resets_after_sustained_active_chunks() {
        let mut stopper = SilenceAutoStopper::new(16_000, 10, 0.04);

        assert!(!stopper.observe(0.0, 16_000 * 9));
        assert!(!stopper.observe(0.08, 1_600));
        assert!(!stopper.observe(0.08, 1_600));
        assert!(!stopper.observe(0.0, 16_000 * 9));
        assert!(stopper.observe(0.0, 16_000));
    }

    #[test]
    fn silence_auto_stop_ignores_brief_loud_spikes() {
        let mut stopper = SilenceAutoStopper::new(16_000, 10, 0.04);

        assert!(!stopper.observe(0.0, 16_000 * 9));
        assert!(!stopper.observe(0.08, 1_600));
        assert!(stopper.observe(0.0, 16_000));
    }

    #[test]
    fn silence_auto_stop_can_be_disabled() {
        let mut stopper = SilenceAutoStopper::new(16_000, 0, 0.04);

        assert!(!stopper.observe(0.0, 16_000 * 600));
    }

    #[test]
    fn pcm_normalizer_keeps_16khz_mono_i16_unchanged() {
        let mut normalizer = PcmNormalizer::new(
            ASR_OUTPUT_SAMPLE_RATE,
            ASR_OUTPUT_CHANNELS as usize,
            ASR_OUTPUT_SAMPLE_RATE,
        );
        let mut pending = Vec::new();

        normalizer.push_i16(&[100, -200, 300], &mut pending);

        assert_eq!(pcm_bytes_to_i16(&pending), vec![100, -200, 300]);
    }

    #[test]
    fn pcm_normalizer_downmixes_and_resamples_to_16khz_mono() {
        let mut normalizer = PcmNormalizer::new(48_000, 2, ASR_OUTPUT_SAMPLE_RATE);
        let mut pending = Vec::new();
        let stereo_48k = [
            1000, 3000, 2000, 4000, 3000, 5000, 4000, 6000, 5000, 7000, 6000, 8000,
        ];

        normalizer.push_i16(&stereo_48k, &mut pending);

        assert_eq!(pcm_bytes_to_i16(&pending), vec![3000, 6000]);
    }

    #[test]
    fn pcm_normalizer_keeps_resample_phase_across_callbacks() {
        let mut normalizer = PcmNormalizer::new(48_000, 1, ASR_OUTPUT_SAMPLE_RATE);
        let mut pending = Vec::new();

        normalizer.push_i16(&[100, 200], &mut pending);
        assert!(pending.is_empty());

        normalizer.push_i16(&[300, 400, 500, 600], &mut pending);

        assert_eq!(pcm_bytes_to_i16(&pending), vec![200, 500]);
    }

    #[test]
    fn pcm_normalizer_upsamples_low_rate_input() {
        let mut normalizer = PcmNormalizer::new(8_000, 1, ASR_OUTPUT_SAMPLE_RATE);
        let mut pending = Vec::new();

        normalizer.push_i16(&[123, -456], &mut pending);

        assert_eq!(pcm_bytes_to_i16(&pending), vec![123, 123, -456, -456]);
    }
}
