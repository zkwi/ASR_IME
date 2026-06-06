use crate::app_log;
use crate::config::AudioConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig, SupportedStreamConfig};
use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const ASR_OUTPUT_SAMPLE_RATE: u32 = 16_000;
pub const ASR_OUTPUT_CHANNELS: u16 = 1;
pub const ASR_MIN_SEGMENT_MS: u64 = 100;
pub const ASR_MAX_SEGMENT_MS: u64 = 200;

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
    pcm_sink: Option<Arc<Mutex<PcmSink>>>,
    level_tx: Option<mpsc::Sender<f32>>,
    silence_tx: Option<mpsc::Sender<()>>,
    silence_auto_stop_seconds: u64,
    silence_level_threshold: f32,
    voice_activity: VoiceActivity,
}

type CaptureStartResult = (Stream, String, u32, u16, Option<Arc<Mutex<PcmSink>>>);

pub struct AudioCapture {
    stop_tx: mpsc::Sender<()>,
    join_handle: Option<JoinHandle<()>>,
    device_name: String,
    sample_rate: u32,
    channels: u16,
    counters: CaptureCounters,
    pcm_sink: Option<Arc<Mutex<PcmSink>>>,
    voice_activity: VoiceActivity,
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

    pub fn recent_voice_elapsed(&self) -> Option<Duration> {
        self.voice_activity.recent_voice_elapsed()
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
    let voice_activity = VoiceActivity::default();
    let worker_voice_activity = voice_activity.clone();

    let join_handle = thread::spawn(move || {
        let outputs = CaptureOutputs {
            chunk_tx,
            pcm_sink: None,
            level_tx,
            silence_tx,
            silence_auto_stop_seconds: audio.silence_auto_stop_seconds,
            silence_level_threshold: audio.silence_level_threshold,
            voice_activity: worker_voice_activity,
        };
        let (stream, device_name, sample_rate, channels, pcm_sink) =
            match start_capture_in_thread(&audio, worker_counters, outputs) {
                Ok(result) => result,
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
        if ready_tx
            .send(Ok((device_name, sample_rate, channels, pcm_sink)))
            .is_err()
        {
            return;
        }
        let _ = stop_rx.recv();
        drop(stream);
    });

    let (device_name, sample_rate, channels, pcm_sink) = ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "启动麦克风采集超时".to_string())??;

    Ok(AudioCapture {
        stop_tx,
        join_handle: Some(join_handle),
        device_name,
        sample_rate,
        channels,
        counters,
        pcm_sink,
        voice_activity,
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

#[derive(Clone, Default)]
struct VoiceActivity {
    last_voice_at: Arc<Mutex<Option<Instant>>>,
}

impl VoiceActivity {
    fn observe(&self, level: f32, threshold: f32) {
        if level <= threshold {
            return;
        }
        if let Ok(mut last_voice_at) = self.last_voice_at.lock() {
            *last_voice_at = Some(Instant::now());
        }
    }

    fn recent_voice_elapsed(&self) -> Option<Duration> {
        self.last_voice_at
            .lock()
            .ok()
            .and_then(|last_voice_at| last_voice_at.map(|instant| instant.elapsed()))
    }
}

fn start_capture_in_thread(
    audio: &AudioConfig,
    counters: CaptureCounters,
    mut outputs: CaptureOutputs,
) -> Result<CaptureStartResult, String> {
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
        )))
    });
    let pcm_sink = outputs.pcm_sink.clone();
    let err_fn = |err| app_log::warn(format!("audio input stream error: {}", err));
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
}

impl PcmSink {
    fn new(
        tx: mpsc::Sender<Vec<u8>>,
        source_rate: u32,
        source_channels: usize,
        target_chunk_bytes: usize,
    ) -> Self {
        Self {
            normalizer: PcmNormalizer::new(source_rate, source_channels, ASR_OUTPUT_SAMPLE_RATE),
            buffer: SegmentedAudioBuffer::new(tx, target_chunk_bytes),
            normalized: Vec::new(),
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
            self.buffer.push(&self.normalized);
        }
        self.buffer.flush();
        emitted
    }

    fn push_with(&mut self, push: impl FnOnce(&mut PcmNormalizer, &mut Vec<u8>) -> usize) -> usize {
        self.normalized.clear();
        let emitted = push(&mut self.normalizer, &mut self.normalized);
        if !self.normalized.is_empty() {
            self.buffer.push(&self.normalized);
        }
        emitted
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
    let voice_threshold = outputs.silence_level_threshold;
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        pcm_sink,
        level_tx,
        silence_tx,
        voice_activity,
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
                voice_activity.observe(level, voice_threshold);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                if let Some(sink) = &pcm_sink {
                    if let Ok(mut sink) = sink.lock() {
                        let emitted = sink.push_i16(data);
                        counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
                    }
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
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let voice_threshold = outputs.silence_level_threshold;
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        pcm_sink,
        level_tx,
        silence_tx,
        voice_activity,
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
                voice_activity.observe(level, voice_threshold);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                if let Some(sink) = &pcm_sink {
                    if let Ok(mut sink) = sink.lock() {
                        let emitted = sink.push_u16(data);
                        counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
                    }
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
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let voice_threshold = outputs.silence_level_threshold;
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        pcm_sink,
        level_tx,
        silence_tx,
        voice_activity,
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
                voice_activity.observe(level, voice_threshold);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                if let Some(sink) = &pcm_sink {
                    if let Ok(mut sink) = sink.lock() {
                        let emitted = sink.push_u8(data);
                        counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
                    }
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
    counters: CaptureCounters,
    outputs: CaptureOutputs,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String> {
    let channels = config.channels.max(1) as usize;
    let voice_threshold = outputs.silence_level_threshold;
    let mut silence = SilenceAutoStopper::new(
        config.sample_rate,
        outputs.silence_auto_stop_seconds,
        outputs.silence_level_threshold,
    );
    let CaptureOutputs {
        pcm_sink,
        level_tx,
        silence_tx,
        voice_activity,
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
                voice_activity.observe(level, voice_threshold);
                send_silence_auto_stop(&silence_tx, &mut silence, level, frame_count);
                if let Some(sink) = &pcm_sink {
                    if let Ok(mut sink) = sink.lock() {
                        let emitted = sink.push_f32(data);
                        counters.pcm_bytes.fetch_add(emitted, Ordering::Relaxed);
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("创建麦克风采集流失败: {}", err))
}

#[cfg(test)]
mod tests {
    use super::{
        effective_asr_segment_ms, target_chunk_bytes, PcmNormalizer, PcmSink, SegmentedAudioBuffer,
        SilenceAutoStopper, ASR_OUTPUT_CHANNELS, ASR_OUTPUT_SAMPLE_RATE,
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
        let mut sink = PcmSink::new(tx, 48_000, 1, 6);

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
