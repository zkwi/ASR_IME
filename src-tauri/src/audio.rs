use crate::app_log;
use crate::config::AudioConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig, SupportedStreamConfig};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const ASR_OUTPUT_SAMPLE_RATE: u32 = 16_000;
pub const ASR_OUTPUT_CHANNELS: u16 = 1;
pub const ASR_MIN_SEGMENT_MS: u64 = 100;
pub const ASR_MAX_SEGMENT_MS: u64 = 200;
const AUDIO_QUALITY_ACTIVE_LEVEL: f32 = 0.035;
const AUDIO_QUALITY_LOW_RMS_LEVEL: f32 = 0.05;
const AUDIO_QUALITY_LOW_PEAK_LEVEL: f32 = 0.18;
const AUDIO_QUALITY_CLIPPING_LEVEL: f32 = 0.96;
const AUDIO_QUALITY_MIN_ACTIVE_RATIO: f32 = 0.10;
const AUDIO_QUALITY_MIN_LEVEL_COUNT: usize = 3;

#[derive(Debug, Clone, Serialize)]
pub struct AudioCaptureInfo {
    pub device_name: String,
    pub device_fallback: Option<AudioDeviceFallbackNotice>,
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

#[derive(Debug, Clone, Serialize)]
pub struct AudioDeviceFallbackNotice {
    pub configured_name: Option<String>,
    pub selected_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioQualityDiagnostic {
    pub rms_dbfs: f32,
    pub peak_dbfs: f32,
    pub active_ratio: f32,
    pub duration_ms: u64,
    pub level_count: usize,
    pub clipping: bool,
    pub status: String,
}

#[derive(Clone)]
struct CaptureCounters {
    chunks: Arc<AtomicUsize>,
    pcm_bytes: Arc<AtomicUsize>,
}

struct CaptureOutputs {
    chunk_tx: Option<mpsc::Sender<Vec<u8>>>,
    pcm_sink: Option<Arc<Mutex<PcmSink>>>,
    level_tx: Option<mpsc::Sender<f32>>,
    silence_tx: Option<mpsc::Sender<()>>,
    error_reporter: CaptureErrorReporter,
    input_gain_factor: f32,
    silence_auto_stop_seconds: u64,
    silence_level_threshold: f32,
}

type CaptureStartResult = (
    Stream,
    String,
    u32,
    u16,
    Option<Arc<Mutex<PcmSink>>>,
    Option<AudioDeviceFallbackNotice>,
);

pub struct AudioCapture {
    stop_tx: mpsc::Sender<()>,
    join_handle: Option<JoinHandle<()>>,
    device_name: String,
    device_fallback: Option<AudioDeviceFallbackNotice>,
    sample_rate: u32,
    channels: u16,
    counters: CaptureCounters,
    pcm_sink: Option<Arc<Mutex<PcmSink>>>,
}

impl AudioCapture {
    pub fn info(&self) -> AudioCaptureInfo {
        AudioCaptureInfo {
            device_name: self.device_name.clone(),
            device_fallback: self.device_fallback.clone(),
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
    error_tx: Option<mpsc::Sender<String>>,
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
            pcm_sink: None,
            level_tx,
            silence_tx,
            error_reporter: CaptureErrorReporter::new(error_tx),
            input_gain_factor: input_gain_factor(audio.input_gain_db),
            silence_auto_stop_seconds: audio.silence_auto_stop_seconds,
            silence_level_threshold: audio.silence_level_threshold,
        };
        let (stream, device_name, sample_rate, channels, pcm_sink, device_fallback) =
            match start_capture_in_thread(&audio, worker_counters, outputs) {
                Ok(result) => result,
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
        if ready_tx
            .send(Ok((
                device_name,
                sample_rate,
                channels,
                pcm_sink,
                device_fallback,
            )))
            .is_err()
        {
            return;
        }
        let _ = stop_rx.recv();
        drop(stream);
    });

    let (device_name, sample_rate, channels, pcm_sink, device_fallback) = ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "启动麦克风采集超时".to_string())??;

    Ok(AudioCapture {
        stop_tx,
        join_handle: Some(join_handle),
        device_name,
        device_fallback,
        sample_rate,
        channels,
        counters,
        pcm_sink,
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
        if let Some(sink) = &self.pcm_sink {
            if let Ok(mut sink) = sink.lock() {
                let emitted = sink.flush();
                self.counters
                    .pcm_bytes
                    .fetch_add(emitted, Ordering::Relaxed);
            }
        }
    }
}

#[derive(Clone)]
struct CaptureErrorReporter {
    tx: Option<mpsc::Sender<String>>,
    reported: Arc<AtomicBool>,
}

impl CaptureErrorReporter {
    fn new(tx: Option<mpsc::Sender<String>>) -> Self {
        Self {
            tx,
            reported: Arc::new(AtomicBool::new(false)),
        }
    }

    fn report(&self, message: String) {
        app_log::warn(&message);
        if self.reported.swap(true, Ordering::Relaxed) {
            return;
        }
        if let Some(tx) = &self.tx {
            let _ = tx.send(message);
        }
    }
}

fn start_capture_in_thread(
    audio: &AudioConfig,
    counters: CaptureCounters,
    mut outputs: CaptureOutputs,
) -> Result<CaptureStartResult, String> {
    let host = cpal::default_host();
    let selected = select_input_device(&host, audio)?;
    let device_fallback = selected.fallback;
    let device = selected.device;
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
    outputs.pcm_sink = outputs.chunk_tx.take().map(|tx| {
        Arc::new(Mutex::new(PcmSink::new(
            tx,
            stream_config.sample_rate,
            stream_config.channels.max(1) as usize,
            target_chunk_bytes(
                ASR_OUTPUT_SAMPLE_RATE,
                ASR_OUTPUT_CHANNELS,
                audio.segment_ms,
            ),
            audio.input_gain_db,
        )))
    });
    let pcm_sink = outputs.pcm_sink.clone();
    let stream_error_reporter = outputs.error_reporter.clone();
    let err_fn = move |err| {
        stream_error_reporter.report(format!("麦克风输入流异常，音频可能不完整: {}", err))
    };
    let stream = match sample_format {
        SampleFormat::I16 => {
            build_i16_stream(&device, &stream_config, counters.clone(), outputs, err_fn)?
        }
        SampleFormat::U16 => {
            build_u16_stream(&device, &stream_config, counters.clone(), outputs, err_fn)?
        }
        SampleFormat::U8 => {
            build_u8_stream(&device, &stream_config, counters.clone(), outputs, err_fn)?
        }
        SampleFormat::F32 => {
            build_f32_stream(&device, &stream_config, counters.clone(), outputs, err_fn)?
        }
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
        pcm_sink,
        device_fallback,
    ))
}

struct SelectedInputDevice {
    device: Device,
    fallback: Option<AudioDeviceFallbackNotice>,
}

#[derive(Debug, Clone)]
struct InputDeviceCandidate {
    index: u32,
    name: String,
    is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InputDeviceResolution {
    index: u32,
    fallback: bool,
    configured_name: Option<String>,
}

fn select_input_device(
    host: &cpal::Host,
    audio: &AudioConfig,
) -> Result<SelectedInputDevice, String> {
    let mut default_device = host.default_input_device();
    if audio
        .input_device_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .is_none()
        && audio.input_device.is_none()
    {
        if let Some(device) = default_device.take() {
            return Ok(SelectedInputDevice {
                device,
                fallback: None,
            });
        }
    }
    let default_name = default_device
        .as_ref()
        .and_then(|device| device.description().ok())
        .map(|description| description.name().to_string());
    let devices = host
        .input_devices()
        .map_err(|err| format!("枚举输入设备失败: {}", err))?
        .enumerate()
        .map(|(index, device)| {
            let name = device
                .description()
                .map(|description| description.name().to_string())
                .unwrap_or_else(|_| format!("Input device {}", index));
            let candidate = InputDeviceCandidate {
                index: index as u32,
                is_default: default_name.as_deref() == Some(name.as_str()),
                name,
            };
            (candidate, device)
        })
        .collect::<Vec<_>>();
    let candidates = devices
        .iter()
        .map(|(candidate, _)| candidate.clone())
        .collect::<Vec<_>>();
    let resolution = resolve_input_device(
        &candidates,
        audio.input_device_name.as_deref(),
        audio.input_device,
    )?;
    if resolution.fallback {
        if let Some(device) = default_device.take() {
            let selected_name = device
                .description()
                .map(|description| description.name().to_string())
                .unwrap_or_else(|_| "Unknown input device".to_string());
            let fallback = AudioDeviceFallbackNotice {
                configured_name: resolution.configured_name,
                selected_name,
            };
            app_log::warn(format!(
                "已保存的麦克风不可用，回退默认输入设备: configured={:?}, selected=\"{}\"",
                fallback.configured_name, fallback.selected_name
            ));
            return Ok(SelectedInputDevice {
                device,
                fallback: Some(fallback),
            });
        }
    }
    let selected = devices
        .into_iter()
        .find(|(candidate, _)| candidate.index == resolution.index)
        .ok_or_else(|| "找不到可用麦克风输入设备".to_string())?;
    let fallback = resolution.fallback.then(|| AudioDeviceFallbackNotice {
        configured_name: resolution.configured_name,
        selected_name: selected.0.name.clone(),
    });
    if let Some(fallback) = fallback.as_ref() {
        app_log::warn(format!(
            "已保存的麦克风不可用，回退默认输入设备: configured={:?}, selected=\"{}\"",
            fallback.configured_name, fallback.selected_name
        ));
    }
    Ok(SelectedInputDevice {
        device: selected.1,
        fallback,
    })
}

fn resolve_input_device(
    devices: &[InputDeviceCandidate],
    input_device_name: Option<&str>,
    legacy_input_device: Option<u32>,
) -> Result<InputDeviceResolution, String> {
    if devices.is_empty() {
        return Err("未找到默认麦克风输入设备".to_string());
    }
    let saved_name = input_device_name
        .map(str::trim)
        .filter(|name| !name.is_empty());
    if let Some(name) = saved_name {
        if let Some(index) = legacy_input_device {
            if let Some(device) = devices
                .iter()
                .find(|device| device.index == index && same_device_name(&device.name, name))
            {
                return Ok(InputDeviceResolution {
                    index: device.index,
                    fallback: false,
                    configured_name: Some(name.to_string()),
                });
            }
        }
        let named_devices = devices
            .iter()
            .filter(|device| same_device_name(&device.name, name))
            .collect::<Vec<_>>();
        if let [device] = named_devices.as_slice() {
            return Ok(InputDeviceResolution {
                index: device.index,
                fallback: false,
                configured_name: Some(name.to_string()),
            });
        }
        let fallback = preferred_default_device(devices);
        return Ok(InputDeviceResolution {
            index: fallback.index,
            fallback: true,
            configured_name: Some(name.to_string()),
        });
    }
    if let Some(index) = legacy_input_device {
        if let Some(device) = devices.iter().find(|device| device.index == index) {
            return Ok(InputDeviceResolution {
                index: device.index,
                fallback: false,
                configured_name: None,
            });
        }
        let fallback = preferred_default_device(devices);
        return Ok(InputDeviceResolution {
            index: fallback.index,
            fallback: true,
            configured_name: Some(format!("index {}", index)),
        });
    }
    let fallback = preferred_default_device(devices);
    Ok(InputDeviceResolution {
        index: fallback.index,
        fallback: false,
        configured_name: None,
    })
}

fn preferred_default_device(devices: &[InputDeviceCandidate]) -> &InputDeviceCandidate {
    devices
        .iter()
        .find(|device| device.is_default)
        .unwrap_or(&devices[0])
}

fn same_device_name(left: &str, right: &str) -> bool {
    left.trim().to_lowercase() == right.trim().to_lowercase()
}

fn select_input_config(
    device: &Device,
    audio: &AudioConfig,
) -> Result<SupportedStreamConfig, String> {
    let target_rate = audio.sample_rate;
    let default_config = device.default_input_config().ok();
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
    default_config
        .or(fallback)
        .ok_or_else(|| "麦克风没有可用采样配置".to_string())
}

fn target_chunk_bytes(sample_rate: u32, channels: u16, segment_ms: u64) -> usize {
    let frames = ((sample_rate as u64 * effective_asr_segment_ms(segment_ms)) / 1000).max(1);
    frames as usize * channels.max(1) as usize * 2
}

pub fn effective_asr_segment_ms(segment_ms: u64) -> u64 {
    segment_ms.clamp(ASR_MIN_SEGMENT_MS, ASR_MAX_SEGMENT_MS)
}

fn send_level(tx: &Option<mpsc::Sender<f32>>, level: f32) {
    if let Some(tx) = tx {
        let _ = tx.send(level.clamp(0.0, 1.0));
    }
}

struct SegmentedAudioBuffer {
    tx: mpsc::Sender<Vec<u8>>,
    pending: Vec<u8>,
    target_bytes: usize,
}

impl SegmentedAudioBuffer {
    fn new(tx: mpsc::Sender<Vec<u8>>, target_bytes: usize) -> Self {
        Self {
            tx,
            pending: Vec::new(),
            target_bytes: target_bytes.max(1),
        }
    }

    fn push(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
        while self.pending.len() >= self.target_bytes {
            let chunk = self.pending.drain(..self.target_bytes).collect::<Vec<_>>();
            let _ = self.tx.send(chunk);
        }
    }

    fn flush(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let tail = std::mem::take(&mut self.pending);
        let _ = self.tx.send(tail);
    }
}

struct PcmSink {
    normalizer: PcmNormalizer,
    buffer: SegmentedAudioBuffer,
    normalized: Vec<u8>,
    gain_factor: f32,
}

impl PcmSink {
    fn new(
        tx: mpsc::Sender<Vec<u8>>,
        source_rate: u32,
        source_channels: usize,
        target_chunk_bytes: usize,
        input_gain_db: f32,
    ) -> Self {
        Self {
            normalizer: PcmNormalizer::new(source_rate, source_channels, ASR_OUTPUT_SAMPLE_RATE),
            buffer: SegmentedAudioBuffer::new(tx, target_chunk_bytes),
            normalized: Vec::new(),
            gain_factor: input_gain_factor(input_gain_db),
        }
    }

    fn push_i16(&mut self, data: &[i16]) -> usize {
        self.push_with(|normalizer, pending| normalizer.push_i16(data, pending))
    }

    fn push_u16(&mut self, data: &[u16]) -> usize {
        self.push_with(|normalizer, pending| normalizer.push_u16(data, pending))
    }

    fn push_u8(&mut self, data: &[u8]) -> usize {
        self.push_with(|normalizer, pending| normalizer.push_u8(data, pending))
    }

    fn push_f32(&mut self, data: &[f32]) -> usize {
        self.push_with(|normalizer, pending| normalizer.push_f32(data, pending))
    }

    fn flush(&mut self) -> usize {
        self.normalized.clear();
        let emitted = self.normalizer.flush(&mut self.normalized);
        if !self.normalized.is_empty() {
            apply_input_gain_to_pcm(&mut self.normalized, self.gain_factor);
            self.buffer.push(&self.normalized);
        }
        self.buffer.flush();
        emitted
    }

    fn push_with(&mut self, push: impl FnOnce(&mut PcmNormalizer, &mut Vec<u8>) -> usize) -> usize {
        self.normalized.clear();
        let emitted = push(&mut self.normalizer, &mut self.normalized);
        if !self.normalized.is_empty() {
            apply_input_gain_to_pcm(&mut self.normalized, self.gain_factor);
            self.buffer.push(&self.normalized);
        }
        emitted
    }
}

fn input_gain_factor(input_gain_db: f32) -> f32 {
    if !input_gain_db.is_finite() {
        return 1.0;
    }
    10.0_f32.powf(input_gain_db.clamp(-12.0, 24.0) / 20.0)
}

fn apply_level_gain(level: f32, gain_factor: f32) -> f32 {
    if !gain_factor.is_finite() {
        return level.clamp(0.0, 1.0);
    }
    (level * gain_factor).clamp(0.0, 1.0)
}

fn apply_input_gain_to_pcm(bytes: &mut [u8], gain_factor: f32) {
    if (gain_factor - 1.0).abs() < f32::EPSILON || !gain_factor.is_finite() {
        return;
    }
    for sample in bytes.chunks_exact_mut(2) {
        let value = i16::from_le_bytes([sample[0], sample[1]]);
        let boosted = ((value as f32) * gain_factor).round() as i32;
        sample.copy_from_slice(&clamp_i32_to_i16(boosted).to_le_bytes());
    }
}

fn linear_ratio_to_db(value: f32) -> f32 {
    20.0 * value.max(f32::EPSILON).log10()
}

fn linear_to_dbfs(value: f32) -> f32 {
    if value <= 0.0 || !value.is_finite() {
        return -90.0;
    }
    linear_ratio_to_db(value).max(-90.0)
}

pub struct AudioQualityAccumulator {
    started_at: Instant,
    sum_square: f64,
    peak: f32,
    active_count: usize,
    clipping_count: usize,
    level_count: usize,
}

impl AudioQualityAccumulator {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            sum_square: 0.0,
            peak: 0.0,
            active_count: 0,
            clipping_count: 0,
            level_count: 0,
        }
    }

    pub fn observe(&mut self, level: f32) {
        let level = level.clamp(0.0, 1.0);
        self.sum_square += (level as f64) * (level as f64);
        self.peak = self.peak.max(level);
        if level >= AUDIO_QUALITY_ACTIVE_LEVEL {
            self.active_count += 1;
        }
        if level >= AUDIO_QUALITY_CLIPPING_LEVEL {
            self.clipping_count += 1;
        }
        self.level_count += 1;
    }

    pub fn finish(&self) -> AudioQualityDiagnostic {
        let rms = if self.level_count == 0 {
            0.0
        } else {
            (self.sum_square / self.level_count as f64).sqrt() as f32
        };
        let active_ratio = if self.level_count == 0 {
            0.0
        } else {
            self.active_count as f32 / self.level_count as f32
        };
        let clipping = self.clipping_count >= 2 || self.peak >= 0.99;
        let status = if self.level_count < AUDIO_QUALITY_MIN_LEVEL_COUNT
            || active_ratio < AUDIO_QUALITY_MIN_ACTIVE_RATIO
        {
            "low_activity"
        } else if clipping {
            "clipping"
        } else if rms < AUDIO_QUALITY_LOW_RMS_LEVEL && self.peak < AUDIO_QUALITY_LOW_PEAK_LEVEL {
            "low_volume"
        } else {
            "ok"
        };
        AudioQualityDiagnostic {
            rms_dbfs: linear_to_dbfs(rms),
            peak_dbfs: linear_to_dbfs(self.peak),
            active_ratio,
            duration_ms: self.started_at.elapsed().as_millis() as u64,
            level_count: self.level_count,
            clipping,
            status: status.to_string(),
        }
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

fn push_pcm_sink(
    pcm_sink: &Option<Arc<Mutex<PcmSink>>>,
    error_reporter: &CaptureErrorReporter,
    counters: &CaptureCounters,
    push: impl FnOnce(&mut PcmSink) -> usize,
) {
    let Some(sink) = pcm_sink else {
        return;
    };
    match sink.lock() {
        Ok(mut sink) => {
            let emitted = push(&mut sink);
            counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
        }
        Err(_) => {
            error_reporter.report("麦克风音频缓冲异常，音频可能不完整。".to_string());
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
        for frame in data.chunks(self.source_channels) {
            self.push_mono_sample(mix_i16_frame(frame), pending);
        }
        pending.len().saturating_sub(before)
    }

    fn push_u16(&mut self, data: &[u16], pending: &mut Vec<u8>) -> usize {
        let before = pending.len();
        for frame in data.chunks(self.source_channels) {
            let samples = frame
                .iter()
                .map(|sample| *sample as i32 - 32768)
                .collect::<Vec<_>>();
            self.push_mono_sample(mix_centered_i32_frame(&samples), pending);
        }
        pending.len().saturating_sub(before)
    }

    fn push_u8(&mut self, data: &[u8], pending: &mut Vec<u8>) -> usize {
        let before = pending.len();
        for frame in data.chunks(self.source_channels) {
            let samples = frame
                .iter()
                .map(|sample| (*sample as i32 - 128) << 8)
                .collect::<Vec<_>>();
            self.push_mono_sample(mix_centered_i32_frame(&samples), pending);
        }
        pending.len().saturating_sub(before)
    }

    fn push_f32(&mut self, data: &[f32], pending: &mut Vec<u8>) -> usize {
        let before = pending.len();
        for frame in data.chunks(self.source_channels) {
            let avg = mix_f32_frame(frame);
            self.push_mono_sample((avg * i16::MAX as f32) as i16, pending);
        }
        pending.len().saturating_sub(before)
    }

    fn flush(&mut self, pending: &mut Vec<u8>) -> usize {
        let before = pending.len();
        if self.target_rate < self.source_rate && self.downsample_count > 0 {
            pending.extend(self.downsample_average().to_le_bytes());
            self.downsample_sum = 0;
            self.downsample_count = 0;
            self.phase = 0;
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
    let samples = frame
        .iter()
        .map(|sample| *sample as i32)
        .collect::<Vec<_>>();
    mix_centered_i32_frame(&samples)
}

fn mix_centered_i32_frame(frame: &[i32]) -> i16 {
    if frame.is_empty() {
        return 0;
    }
    let sum = frame.iter().sum::<i32>();
    let average = sum / frame.len() as i32;
    let mut strongest = frame[0];
    for sample in &frame[1..] {
        if sample.unsigned_abs() > strongest.unsigned_abs() {
            strongest = *sample;
        }
    }

    if average.unsigned_abs().saturating_mul(2) < strongest.unsigned_abs() {
        clamp_i32_to_i16(strongest)
    } else {
        clamp_i32_to_i16(average)
    }
}

fn mix_f32_frame(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let sum = frame
        .iter()
        .map(|sample| sample.clamp(-1.0, 1.0))
        .sum::<f32>();
    let average = sum / frame.len() as f32;
    let mut strongest = frame[0].clamp(-1.0, 1.0);
    for sample in &frame[1..] {
        let sample = sample.clamp(-1.0, 1.0);
        if sample.abs() > strongest.abs() {
            strongest = sample;
        }
    }

    if average.abs() * 2.0 < strongest.abs() {
        strongest
    } else {
        average
    }
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
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        pcm_sink,
        level_tx,
        silence_tx,
        error_reporter,
        input_gain_factor,
        ..
    } = outputs;
    device
        .build_input_stream(
            config,
            move |data: &[i16], _| {
                let frame_count = data.len() / channels;
                counters.chunks.fetch_add(1, Ordering::Relaxed);
                let level = apply_level_gain(rms_i16(data), input_gain_factor);
                send_level(&level_tx, level);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                push_pcm_sink(&pcm_sink, &error_reporter, &counters, |sink| {
                    sink.push_i16(data)
                });
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

fn build_u16_stream(
    device: &Device,
    config: &StreamConfig,
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        pcm_sink,
        level_tx,
        silence_tx,
        error_reporter,
        input_gain_factor,
        ..
    } = outputs;
    device
        .build_input_stream(
            config,
            move |data: &[u16], _| {
                let frame_count = data.len() / channels;
                counters.chunks.fetch_add(1, Ordering::Relaxed);
                let level = apply_level_gain(rms_u16(data), input_gain_factor);
                send_level(&level_tx, level);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                push_pcm_sink(&pcm_sink, &error_reporter, &counters, |sink| {
                    sink.push_u16(data)
                });
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

fn build_u8_stream(
    device: &Device,
    config: &StreamConfig,
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        pcm_sink,
        level_tx,
        silence_tx,
        error_reporter,
        input_gain_factor,
        ..
    } = outputs;
    device
        .build_input_stream(
            config,
            move |data: &[u8], _| {
                let frame_count = data.len() / channels;
                counters.chunks.fetch_add(1, Ordering::Relaxed);
                let level = apply_level_gain(rms_u8(data), input_gain_factor);
                send_level(&level_tx, level);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                push_pcm_sink(&pcm_sink, &error_reporter, &counters, |sink| {
                    sink.push_u8(data)
                });
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

fn build_f32_stream(
    device: &Device,
    config: &StreamConfig,
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        pcm_sink,
        level_tx,
        silence_tx,
        error_reporter,
        input_gain_factor,
        ..
    } = outputs;
    device
        .build_input_stream(
            config,
            move |data: &[f32], _| {
                let frame_count = data.len() / channels;
                counters.chunks.fetch_add(1, Ordering::Relaxed);
                let level = apply_level_gain(rms_f32(data), input_gain_factor);
                send_level(&level_tx, level);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                push_pcm_sink(&pcm_sink, &error_reporter, &counters, |sink| {
                    sink.push_f32(data)
                });
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

#[cfg(test)]
mod tests {
    use super::{
        apply_input_gain_to_pcm, apply_level_gain, effective_asr_segment_ms, input_gain_factor,
        resolve_input_device, target_chunk_bytes, AudioQualityAccumulator, CaptureErrorReporter,
        InputDeviceCandidate, PcmNormalizer, PcmSink, SegmentedAudioBuffer, SilenceAutoStopper,
        ASR_OUTPUT_CHANNELS, ASR_OUTPUT_SAMPLE_RATE,
    };
    use std::sync::mpsc;

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
    fn asr_segment_ms_is_clamped_to_doubao_recommended_range() {
        assert_eq!(effective_asr_segment_ms(20), 100);
        assert_eq!(effective_asr_segment_ms(160), 160);
        assert_eq!(effective_asr_segment_ms(500), 200);
        assert_eq!(target_chunk_bytes(16_000, 1, 20), 3_200);
        assert_eq!(target_chunk_bytes(16_000, 1, 500), 6_400);
    }

    #[test]
    fn input_device_name_survives_index_change() {
        let devices = vec![
            InputDeviceCandidate {
                index: 0,
                name: "Webcam Mic".to_string(),
                is_default: false,
            },
            InputDeviceCandidate {
                index: 1,
                name: "USB Microphone".to_string(),
                is_default: true,
            },
        ];

        let selected = resolve_input_device(&devices, Some("usb microphone"), Some(0)).unwrap();

        assert_eq!(selected.index, 1);
        assert!(!selected.fallback);
    }

    #[test]
    fn saved_index_disambiguates_duplicate_device_names() {
        let devices = vec![
            InputDeviceCandidate {
                index: 0,
                name: "Microphone".to_string(),
                is_default: false,
            },
            InputDeviceCandidate {
                index: 1,
                name: "Headset Microphone".to_string(),
                is_default: true,
            },
            InputDeviceCandidate {
                index: 2,
                name: "Microphone".to_string(),
                is_default: false,
            },
        ];

        let selected = resolve_input_device(&devices, Some("Microphone"), Some(2)).unwrap();

        assert_eq!(selected.index, 2);
        assert!(!selected.fallback);
    }

    #[test]
    fn duplicate_saved_name_without_matching_index_falls_back() {
        let devices = vec![
            InputDeviceCandidate {
                index: 0,
                name: "Microphone".to_string(),
                is_default: false,
            },
            InputDeviceCandidate {
                index: 1,
                name: "Headset Microphone".to_string(),
                is_default: true,
            },
            InputDeviceCandidate {
                index: 2,
                name: "Microphone".to_string(),
                is_default: false,
            },
        ];

        let selected = resolve_input_device(&devices, Some("Microphone"), Some(9)).unwrap();

        assert_eq!(selected.index, 1);
        assert!(selected.fallback);
        assert_eq!(selected.configured_name.as_deref(), Some("Microphone"));
    }

    #[test]
    fn missing_saved_input_device_falls_back_to_default() {
        let devices = vec![
            InputDeviceCandidate {
                index: 0,
                name: "Webcam Mic".to_string(),
                is_default: false,
            },
            InputDeviceCandidate {
                index: 3,
                name: "System Default Mic".to_string(),
                is_default: true,
            },
        ];

        let selected = resolve_input_device(&devices, Some("USB Microphone"), Some(1)).unwrap();

        assert_eq!(selected.index, 3);
        assert!(selected.fallback);
        assert_eq!(selected.configured_name.as_deref(), Some("USB Microphone"));
    }

    #[test]
    fn default_input_device_is_used_when_no_device_is_saved() {
        let devices = vec![
            InputDeviceCandidate {
                index: 0,
                name: "Webcam Mic".to_string(),
                is_default: false,
            },
            InputDeviceCandidate {
                index: 2,
                name: "Default Array".to_string(),
                is_default: true,
            },
        ];

        let selected = resolve_input_device(&devices, None, None).unwrap();

        assert_eq!(selected.index, 2);
        assert!(!selected.fallback);
    }

    #[test]
    fn capture_error_reporter_only_sends_first_error() {
        let (tx, rx) = mpsc::channel();
        let reporter = CaptureErrorReporter::new(Some(tx));

        reporter.report("first audio error".to_string());
        reporter.report("second audio error".to_string());

        assert_eq!(rx.try_recv().unwrap(), "first audio error");
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn input_gain_factor_uses_db_scale() {
        assert!((input_gain_factor(0.0) - 1.0).abs() < 0.001);
        assert!((input_gain_factor(6.0) - 1.995).abs() < 0.01);
        assert!((apply_level_gain(0.02, input_gain_factor(12.0)) - 0.079).abs() < 0.01);
    }

    #[test]
    fn input_gain_boosts_pcm_with_saturation() {
        let mut bytes = [10_000_i16, -20_000, 20_000]
            .into_iter()
            .flat_map(i16::to_le_bytes)
            .collect::<Vec<_>>();

        apply_input_gain_to_pcm(&mut bytes, input_gain_factor(6.0));

        assert_eq!(pcm_bytes_to_i16(&bytes), vec![19_953, -32768, 32767]);
    }

    #[test]
    fn audio_quality_flags_low_activity() {
        let mut quality = AudioQualityAccumulator::new();
        for _ in 0..20 {
            quality.observe(0.0);
        }

        let diagnostic = quality.finish();

        assert_eq!(diagnostic.status, "low_activity");
        assert_eq!(diagnostic.active_ratio, 0.0);
        assert_eq!(diagnostic.peak_dbfs, -90.0);
    }

    #[test]
    fn audio_quality_flags_low_volume() {
        let mut quality = AudioQualityAccumulator::new();
        for _ in 0..20 {
            quality.observe(0.04);
        }

        let diagnostic = quality.finish();

        assert_eq!(diagnostic.status, "low_volume");
        assert!(diagnostic.rms_dbfs < -25.0);
        assert!(diagnostic.active_ratio > 0.9);
    }

    #[test]
    fn audio_quality_flags_clipping() {
        let mut quality = AudioQualityAccumulator::new();
        for _ in 0..18 {
            quality.observe(0.25);
        }
        quality.observe(0.97);
        quality.observe(0.98);

        let diagnostic = quality.finish();

        assert_eq!(diagnostic.status, "clipping");
        assert!(diagnostic.clipping);
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
    fn pcm_normalizer_preserves_audio_when_stereo_channels_cancel_out() {
        let mut normalizer = PcmNormalizer::new(48_000, 2, ASR_OUTPUT_SAMPLE_RATE);
        let mut pending = Vec::new();
        let phase_inverted_stereo = [
            1200, -1200, 2400, -2400, 3600, -3600, 4800, -4800, 6000, -6000, 7200, -7200,
        ];

        normalizer.push_i16(&phase_inverted_stereo, &mut pending);

        assert_eq!(pcm_bytes_to_i16(&pending), vec![2400, 6000]);
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
    fn pcm_normalizer_flushes_partial_downsample_window() {
        let mut normalizer = PcmNormalizer::new(48_000, 1, ASR_OUTPUT_SAMPLE_RATE);
        let mut pending = Vec::new();

        normalizer.push_i16(&[100, 200], &mut pending);
        assert!(pending.is_empty());

        let emitted = normalizer.flush(&mut pending);

        assert_eq!(emitted, 2);
        assert_eq!(pcm_bytes_to_i16(&pending), vec![150]);
    }

    #[test]
    fn pcm_normalizer_keeps_incomplete_trailing_frame() {
        let mut normalizer = PcmNormalizer::new(ASR_OUTPUT_SAMPLE_RATE, 2, ASR_OUTPUT_SAMPLE_RATE);
        let mut pending = Vec::new();

        normalizer.push_i16(&[100, 200, 300], &mut pending);

        assert_eq!(pcm_bytes_to_i16(&pending), vec![150, 300]);
    }

    #[test]
    fn pcm_normalizer_upsamples_low_rate_input() {
        let mut normalizer = PcmNormalizer::new(8_000, 1, ASR_OUTPUT_SAMPLE_RATE);
        let mut pending = Vec::new();

        normalizer.push_i16(&[123, -456], &mut pending);

        assert_eq!(pcm_bytes_to_i16(&pending), vec![123, 123, -456, -456]);
    }

    #[test]
    fn pcm_sink_flushes_normalizer_residual_and_segment_tail() {
        let (tx, rx) = mpsc::channel();
        let mut sink = PcmSink::new(tx, 48_000, 1, 6, 0.0);

        assert_eq!(sink.push_i16(&[100, 200]), 0);
        assert!(rx.try_recv().is_err());

        assert_eq!(sink.flush(), 2);

        assert_eq!(pcm_bytes_to_i16(&rx.try_recv().unwrap()), vec![150]);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn segmented_audio_buffer_flushes_partial_tail() {
        let (tx, rx) = mpsc::channel();
        let mut buffer = SegmentedAudioBuffer::new(tx, 6);

        buffer.push(&[1, 2, 3, 4]);
        assert!(rx.try_recv().is_err());

        buffer.push(&[5, 6, 7]);
        assert_eq!(rx.try_recv().unwrap(), vec![1, 2, 3, 4, 5, 6]);
        assert!(rx.try_recv().is_err());

        buffer.flush();
        assert_eq!(rx.try_recv().unwrap(), vec![7]);
    }
}
