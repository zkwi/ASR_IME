use serde::{Deserialize, Serialize};
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

use crate::app_log;
use crate::asr_ws;
use crate::audio::{self, AudioCapture};
use crate::config;
use crate::overlay;
use crate::screen_context;
use crate::system_audio::{self, VolumeState};
use crate::tray;

const STOP_TAIL_POLL_MS: u64 = 50;
// 尾字偶发截断时优先收更完整的真实尾音，不用中间 ASR 文本或尾部静音兜底。
// 维护依据见 docs/asr-quality-latency-guardrails.md。
const STOP_TAIL_MIN_CAPTURE_MS: u64 = 150;
const STOP_TAIL_MIN_QUIET_MS: u64 = 150;
const STOP_TAIL_MAX_EXTRA_MS: u64 = 150;

#[derive(Debug, Clone, Serialize)]
pub struct SessionState {
    pub recording: bool,
    pub phase: SessionPhase,
    pub message: String,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    #[default]
    Idle,
    Starting,
    Recording,
    Stopping,
    WaitingFinalResult,
    PostEditing,
    Pasting,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioLevel {
    pub level: f32,
}

#[derive(Default)]
struct InnerSession {
    recording: bool,
    phase: SessionPhase,
    message: String,
    error_code: Option<String>,
    generation: u64,
    audio_capture: Option<AudioCapture>,
    volume_state: Option<VolumeState>,
}

#[derive(Clone, Default)]
pub struct SessionController {
    inner: Arc<Mutex<InnerSession>>,
}

impl SessionController {
    pub fn current_state(&self) -> SessionState {
        let Ok(inner) = self.inner.lock() else {
            app_log::warn("读取会话状态失败：session mutex poisoned");
            return SessionState {
                recording: false,
                phase: SessionPhase::Failed,
                message: "Session state is unavailable.".to_string(),
                error_code: Some("SESSION_STATE_UNAVAILABLE".to_string()),
            };
        };
        state_from_inner(&inner)
    }

    /// 启动一轮录音会话，并把后续 ASR worker 绑定到当前 generation。
    ///
    /// 已进入等待最终结果、润色或粘贴阶段时不会重新启动录音，调用方会拿到当前状态。
    /// 这是全局热键、右 Alt、鼠标中键和托盘入口共用的保护边界。
    pub fn start(&self, app: Option<AppHandle>) -> Result<SessionState, String> {
        let current = self.current_state();
        if is_processing_phase(current.phase) {
            emit_state(app.as_ref(), &current);
            return Ok(current);
        }
        let loaded = config::load_config()?;
        let max_seconds = loaded.data.audio.max_record_seconds;
        if loaded.data.auth.app_key.trim().is_empty()
            || loaded.data.auth.access_key.trim().is_empty()
        {
            let message = if loaded.exists {
                "ASR 未配置 app_key/access_key，请先在配置页填写豆包认证信息并保存。"
            } else {
                "未找到 config.toml。请先在配置页填写豆包认证信息并保存，或复制 config.example.toml 为 config.toml 后手动编辑。"
            };
            let error_code = if loaded.exists {
                "ASR_AUTH_MISSING"
            } else {
                "CONFIG_MISSING"
            };
            app_log::warn(format!(
                "录音启动被拦截: config_exists={}, auth_ready=false",
                loaded.exists
            ));
            let state = SessionState {
                recording: false,
                phase: SessionPhase::Failed,
                message: message.to_string(),
                error_code: Some(error_code.to_string()),
            };
            self.set_state_values(false, SessionPhase::Failed, message, Some(error_code));
            if let Some(app) = app.as_ref() {
                emit_state(Some(app), &state);
            }
            return Err(message.to_string());
        }
        let generation = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| "session mutex poisoned".to_string())?;
            if inner.recording || is_processing_phase(inner.phase) {
                let state = state_from_inner(&inner);
                drop(inner);
                emit_state(app.as_ref(), &state);
                return Ok(state);
            }
            inner.recording = true;
            inner.phase = SessionPhase::Starting;
            inner.message = "Recording is starting.".to_string();
            inner.error_code = None;
            inner.generation = inner.generation.wrapping_add(1);
            inner.audio_capture = None;
            inner.volume_state = None;
            inner.generation
        };

        let (audio_tx, audio_rx) = std::sync::mpsc::channel();
        let (level_tx, level_rx) = if app.is_some() {
            let (tx, rx) = std::sync::mpsc::channel();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        let silence_auto_stop_enabled = loaded.data.audio.silence_auto_stop_seconds > 0;
        let (silence_tx, silence_rx) = std::sync::mpsc::channel();
        let silence_tx = if silence_auto_stop_enabled {
            Some(silence_tx)
        } else {
            None
        };
        let (audio_error_tx, audio_error_rx) = std::sync::mpsc::channel();
        app_log::info(format!(
            "录音启动请求: max_seconds={}, stop_grace_ms={}, silence_auto_stop_seconds={}, silence_level_threshold={}, mute_system_volume={}",
            max_seconds,
            loaded.data.audio.stop_grace_ms,
            loaded.data.audio.silence_auto_stop_seconds,
            loaded.data.audio.silence_level_threshold,
            loaded.data.audio.mute_system_volume_while_recording
        ));
        if let Some(app) = app.as_ref() {
            overlay::show_message(app, &loaded.data.ui, overlay::STARTING_TEXT);
            let starting = SessionState {
                recording: true,
                phase: SessionPhase::Starting,
                message: "Recording is starting.".to_string(),
                error_code: None,
            };
            emit_state(Some(app), &starting);
        }
        let audio_capture = match audio::start_capture(
            &loaded.data.audio,
            Some(audio_tx),
            level_tx,
            silence_tx,
            Some(audio_error_tx),
        ) {
            Ok(capture) => capture,
            Err(err) => {
                let error_code =
                    if err.contains("未找到") || err.contains("找不到") || err.contains("没有可用")
                    {
                        "MIC_DEVICE_NOT_FOUND"
                    } else {
                        "MIC_START_FAILED"
                    };
                let state = self.force_stop_generation(
                    generation,
                    SessionPhase::Failed,
                    "Recording failed to start.",
                    Some(error_code),
                );
                if let Some(app) = app.as_ref() {
                    overlay::update_text(app, format!("启动录音失败: {}", err));
                    overlay::hide(app);
                    emit_state(
                        Some(app),
                        &state.unwrap_or(SessionState {
                            recording: false,
                            phase: SessionPhase::Failed,
                            message: format!("Recording failed: {}", err),
                            error_code: Some(error_code.to_string()),
                        }),
                    );
                }
                app_log::warn(format!("启动麦克风失败: {}", err));
                return Err(err);
            }
        };
        let started_at = Instant::now();
        let volume_state = if loaded.data.audio.mute_system_volume_while_recording {
            system_audio::safe_mute_and_save()
        } else {
            None
        };
        let screen_context_rx = screen_context::spawn_capture(&loaded.data.screen_context);
        let audio_info = audio_capture.info();
        app_log::info(format!(
            "麦克风采集已启动: device=\"{}\", rate={}Hz, channels={}",
            audio_info.device_name, audio_info.sample_rate, audio_info.channels
        ));
        if let (Some(app_for_level), Some(level_rx)) = (app.clone(), level_rx) {
            spawn_audio_level_emitter(app_for_level, level_rx);
        }
        if silence_auto_stop_enabled {
            spawn_silence_auto_stop_listener(
                self.clone(),
                app.clone(),
                generation,
                silence_rx,
                loaded.data.audio.stop_grace_ms,
            );
        }
        spawn_audio_error_listener(self.clone(), app.clone(), generation, audio_error_rx);
        let mut runtime_config = loaded.data.clone();
        runtime_config.audio.sample_rate = audio_info.sample_rate;
        runtime_config.audio.channels = audio_info.channels;
        let mut audio_capture = Some(audio_capture);
        let mut volume_state = volume_state;
        let started = {
            let mut inner = match self.inner.lock() {
                Ok(inner) => inner,
                Err(_) => {
                    system_audio::safe_restore(volume_state);
                    return Err("session mutex poisoned".to_string());
                }
            };
            if !inner.recording || inner.generation != generation {
                false
            } else {
                inner.audio_capture = audio_capture.take();
                inner.volume_state = volume_state.take();
                inner.phase = SessionPhase::Recording;
                inner.message = "Recording started.".to_string();
                inner.error_code = None;
                true
            }
        };
        if !started {
            system_audio::safe_restore(volume_state);
            return Ok(SessionState {
                recording: false,
                phase: SessionPhase::Idle,
                message: "Recording is already idle.".to_string(),
                error_code: None,
            });
        }

        let state = SessionState {
            recording: true,
            phase: SessionPhase::Recording,
            message: "Recording started.".to_string(),
            error_code: None,
        };
        app_log::info("录音会话已开始");
        if let Some(app) = app.as_ref() {
            overlay::update_text(app, overlay::RECORDING_TEXT);
        }
        emit_state(app.as_ref(), &state);
        if let Some(app) = app.clone() {
            asr_ws::spawn_asr_worker(
                runtime_config,
                audio_rx,
                started_at,
                app,
                self.clone(),
                generation,
                screen_context_rx,
            );
        }

        let controller = self.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(max_seconds.max(1)));
            let stopped = controller.force_stop_generation(
                generation,
                SessionPhase::WaitingFinalResult,
                "Recording reached the configured maximum duration.",
                None,
            );
            if let (Some(app), Some(state)) = (app, stopped) {
                emit_state(Some(&app), &state);
            }
        });

        Ok(state)
    }

    /// 停止当前录音会话。
    ///
    /// 若配置了尾音保留，会先进入 `Stopping`，短暂等待后再切到等待最终结果；
    /// generation 必须保持不变，避免旧 ASR worker 覆盖新会话状态。
    pub fn stop(&self, app: Option<AppHandle>) -> Result<SessionState, String> {
        let loaded = config::load_config()?;
        let grace_ms = loaded.data.audio.stop_grace_ms;
        let generation = {
            let inner = self
                .inner
                .lock()
                .map_err(|_| "session mutex poisoned".to_string())?;
            if !inner.recording {
                return Ok(SessionState {
                    recording: false,
                    phase: inner.phase,
                    message: "Recording is already idle.".to_string(),
                    error_code: inner.error_code.clone(),
                });
            }
            inner.generation
        };
        let Some(state) = self.stop_generation_with_grace(
            app,
            generation,
            grace_ms,
            "Recording stopped.",
            "Recording stopped after grace period.",
            "收到停止录音请求",
        ) else {
            return Ok(self.current_state());
        };

        Ok(state)
    }

    pub fn toggle(&self, app: Option<AppHandle>) -> Result<SessionState, String> {
        let current = self.current_state();
        match current.phase {
            SessionPhase::Starting | SessionPhase::Stopping => {
                emit_state(app.as_ref(), &current);
                Ok(current)
            }
            SessionPhase::Recording => self.stop(app),
            SessionPhase::WaitingFinalResult
            | SessionPhase::PostEditing
            | SessionPhase::Pasting => {
                emit_state(app.as_ref(), &current);
                Ok(current)
            }
            _ => self.start(app),
        }
    }

    fn force_stop(
        &self,
        phase: SessionPhase,
        message: &str,
        error_code: Option<&str>,
    ) -> SessionState {
        let Ok(mut inner) = self.inner.lock() else {
            app_log::warn("停止会话失败：session mutex poisoned");
            return SessionState {
                recording: false,
                phase: SessionPhase::Failed,
                message: message.to_string(),
                error_code: Some("SESSION_STOP_FAILED".to_string()),
            };
        };
        inner.recording = false;
        inner.phase = phase;
        inner.message = message.to_string();
        inner.error_code = error_code.map(str::to_string);
        system_audio::safe_restore(inner.volume_state.take());
        inner.audio_capture = None;
        app_log::info(message);
        SessionState {
            recording: false,
            phase,
            message: message.to_string(),
            error_code: error_code.map(str::to_string),
        }
    }

    fn force_stop_generation(
        &self,
        generation: u64,
        phase: SessionPhase,
        message: &str,
        error_code: Option<&str>,
    ) -> Option<SessionState> {
        let Ok(mut inner) = self.inner.lock() else {
            app_log::warn("停止指定会话失败：session mutex poisoned");
            return None;
        };
        if !inner.recording || inner.generation != generation {
            return None;
        }
        inner.recording = false;
        inner.phase = phase;
        inner.message = message.to_string();
        inner.error_code = error_code.map(str::to_string);
        system_audio::safe_restore(inner.volume_state.take());
        inner.audio_capture = None;
        app_log::info(message);
        Some(SessionState {
            recording: false,
            phase,
            message: message.to_string(),
            error_code: error_code.map(str::to_string),
        })
    }

    fn fail_recording_generation_and_invalidate(
        &self,
        generation: u64,
        message: &str,
        error_code: &str,
    ) -> Option<(SessionState, u64)> {
        let Ok(mut inner) = self.inner.lock() else {
            app_log::warn("终止异常录音失败：session mutex poisoned");
            return None;
        };
        if !inner.recording || inner.generation != generation {
            return None;
        }
        inner.recording = false;
        inner.phase = SessionPhase::Failed;
        inner.message = message.to_string();
        inner.error_code = Some(error_code.to_string());
        system_audio::safe_restore(inner.volume_state.take());
        inner.audio_capture = None;
        inner.generation = inner.generation.wrapping_add(1);
        app_log::warn(message);
        let guard_generation = inner.generation;
        Some((state_from_inner(&inner), guard_generation))
    }

    fn begin_stopping_generation(&self, generation: u64, message: &str) -> Option<SessionState> {
        let Ok(mut inner) = self.inner.lock() else {
            app_log::warn("停止指定会话失败：session mutex poisoned");
            return None;
        };
        if !inner.recording || inner.generation != generation {
            return None;
        }
        inner.phase = SessionPhase::Stopping;
        inner.message = message.to_string();
        inner.error_code = None;
        Some(state_from_inner(&inner))
    }

    fn stop_generation_with_grace(
        &self,
        app: Option<AppHandle>,
        generation: u64,
        grace_ms: u64,
        immediate_message: &'static str,
        grace_message: &'static str,
        log_source: &'static str,
    ) -> Option<SessionState> {
        if grace_ms == 0 {
            let state = self.force_stop_generation(
                generation,
                SessionPhase::WaitingFinalResult,
                immediate_message,
                None,
            );
            if let (Some(app), Some(state)) = (app, state.as_ref()) {
                emit_state(Some(&app), state);
            }
            return state;
        }

        let state = self.begin_stopping_generation(generation, "Recording is stopping.")?;
        emit_state(app.as_ref(), &state);
        app_log::info(format!("{}，等待 {} ms 收尾", log_source, grace_ms));

        let controller = self.clone();
        thread::spawn(move || {
            wait_for_stop_tail(&controller, generation, grace_ms);
            let stopped = controller.force_stop_generation(
                generation,
                SessionPhase::WaitingFinalResult,
                grace_message,
                None,
            );
            if let (Some(app), Some(state)) = (app, stopped) {
                emit_state(Some(&app), &state);
            }
        });

        Some(state)
    }

    pub fn abort_from_worker(&self, app: &AppHandle, message: &str) {
        self.abort_from_worker_with_code(app, message, "SESSION_FAILED");
    }

    pub fn abort_from_worker_with_code(&self, app: &AppHandle, message: &str, error_code: &str) {
        let state = self.force_stop(SessionPhase::Failed, message, Some(error_code));
        emit_state(Some(app), &state);
    }

    pub fn abort_generation_from_worker_with_code(
        &self,
        app: &AppHandle,
        generation: u64,
        message: &str,
        error_code: &str,
    ) -> bool {
        match self.finish_generation(
            generation,
            Some(app),
            SessionPhase::Failed,
            message,
            Some(error_code),
        ) {
            Some(_) => true,
            None => {
                app_log::info(format!(
                    "忽略过期 ASR worker 失败状态: generation={}, error_code={}",
                    generation, error_code
                ));
                false
            }
        }
    }

    pub fn is_current_generation(&self, generation: u64) -> bool {
        let Ok(inner) = self.inner.lock() else {
            app_log::warn("检查会话 generation 失败：session mutex poisoned");
            return false;
        };
        inner.generation == generation
    }

    fn recent_voice_elapsed_for_generation(&self, generation: u64) -> Option<Option<Duration>> {
        let Ok(inner) = self.inner.lock() else {
            app_log::warn("检查会话尾音状态失败：session mutex poisoned");
            return None;
        };
        if !inner.recording || inner.generation != generation {
            return None;
        }
        let audio_capture = inner.audio_capture.as_ref()?;
        Some(audio_capture.recent_voice_elapsed())
    }

    pub fn set_phase_for_generation(
        &self,
        generation: u64,
        app: Option<&AppHandle>,
        phase: SessionPhase,
        message: &str,
        error_code: Option<&str>,
    ) -> Option<SessionState> {
        let state = self.set_state_values_for_generation(generation, phase, message, error_code);
        if let Some(state) = state.as_ref() {
            emit_state(app, state);
        } else {
            app_log::info(format!(
                "忽略过期会话状态更新: generation={}, phase={:?}",
                generation, phase
            ));
        }
        state
    }

    pub fn finish_generation(
        &self,
        generation: u64,
        app: Option<&AppHandle>,
        phase: SessionPhase,
        message: &str,
        error_code: Option<&str>,
    ) -> Option<SessionState> {
        let Ok(mut inner) = self.inner.lock() else {
            app_log::warn("结束指定会话失败：session mutex poisoned");
            return None;
        };
        if inner.generation != generation {
            return None;
        }
        inner.recording = false;
        inner.phase = phase;
        inner.message = message.to_string();
        inner.error_code = error_code.map(str::to_string);
        system_audio::safe_restore(inner.volume_state.take());
        inner.audio_capture = None;
        app_log::info(message);
        let state = state_from_inner(&inner);
        drop(inner);
        emit_state(app, &state);
        Some(state)
    }

    fn set_state_values(
        &self,
        recording: bool,
        phase: SessionPhase,
        message: &str,
        error_code: Option<&str>,
    ) {
        let Ok(mut inner) = self.inner.lock() else {
            app_log::warn("更新会话状态失败：session mutex poisoned");
            return;
        };
        inner.recording = recording;
        inner.phase = phase;
        inner.message = message.to_string();
        inner.error_code = error_code.map(str::to_string);
    }

    fn set_state_values_for_generation(
        &self,
        generation: u64,
        phase: SessionPhase,
        message: &str,
        error_code: Option<&str>,
    ) -> Option<SessionState> {
        let Ok(mut inner) = self.inner.lock() else {
            app_log::warn("更新指定会话状态失败：session mutex poisoned");
            return None;
        };
        if inner.generation != generation {
            return None;
        }
        inner.recording = matches!(
            phase,
            SessionPhase::Starting | SessionPhase::Recording | SessionPhase::Stopping
        );
        inner.phase = phase;
        inner.message = message.to_string();
        inner.error_code = error_code.map(str::to_string);
        Some(state_from_inner(&inner))
    }
}

pub fn emit_state(app: Option<&AppHandle>, state: &SessionState) {
    if let Some(app) = app {
        tray::set_input_active(app, is_tray_input_active_phase(state.phase));
        let _ = app.emit("session-state-changed", state);
    }
}

fn spawn_audio_level_emitter(app: AppHandle, level_rx: std::sync::mpsc::Receiver<f32>) {
    thread::spawn(move || {
        let mut last_emit = Instant::now()
            .checked_sub(Duration::from_millis(100))
            .unwrap_or_else(Instant::now);
        while let Ok(level) = level_rx.recv() {
            if last_emit.elapsed() < Duration::from_millis(80) {
                continue;
            }
            let _ = app.emit(
                "audio-level",
                AudioLevel {
                    level: level.clamp(0.0, 1.0),
                },
            );
            last_emit = Instant::now();
        }
        let _ = app.emit("audio-level", AudioLevel { level: 0.0 });
    });
}

fn spawn_silence_auto_stop_listener(
    controller: SessionController,
    app: Option<AppHandle>,
    generation: u64,
    silence_rx: Receiver<()>,
    stop_grace_ms: u64,
) {
    thread::spawn(move || {
        if silence_rx.recv().is_err() {
            return;
        }
        let stopped = controller.stop_generation_with_grace(
            app,
            generation,
            stop_grace_ms,
            "Recording stopped after local silence timeout.",
            "Recording stopped after local silence timeout.",
            "本地静音超时触发停止录音",
        );
        if stopped.is_some() {
            app_log::info("本地静音超时已按手动停止流程结束录音。");
        }
    });
}

fn spawn_audio_error_listener(
    controller: SessionController,
    app: Option<AppHandle>,
    generation: u64,
    audio_error_rx: Receiver<String>,
) {
    thread::spawn(move || {
        let Ok(detail) = audio_error_rx.recv() else {
            return;
        };
        let message = format!(
            "麦克风采集异常，已停止本次识别，避免使用不完整音频。{}",
            detail
        );
        let Some((state, guard_generation)) = controller.fail_recording_generation_and_invalidate(
            generation,
            &message,
            "MIC_STREAM_FAILED",
        ) else {
            return;
        };
        if let Some(app) = app.as_ref() {
            emit_state(Some(app), &state);
            overlay::update_text(app, &message);
            let _ = app.emit(
                "asr-final-text",
                asr_ws::AsrFinalText {
                    text: String::new(),
                    error: Some(message),
                    error_code: Some("MIC_STREAM_FAILED".to_string()),
                    warning: None,
                    warning_code: None,
                },
            );
        }
        thread::sleep(Duration::from_millis(1_800));
        if let Some(app) = app.as_ref() {
            if controller.is_current_generation(guard_generation) {
                overlay::hide(app);
            }
        }
    });
}

fn wait_for_stop_tail(controller: &SessionController, generation: u64, grace_ms: u64) {
    let started_at = Instant::now();
    let min_wait = effective_stop_tail_min_wait(grace_ms);
    let quiet_window = Duration::from_millis(grace_ms.max(STOP_TAIL_MIN_QUIET_MS));
    let max_wait = min_wait + Duration::from_millis(STOP_TAIL_MAX_EXTRA_MS);

    loop {
        let elapsed = started_at.elapsed();
        let Some(recent_voice_elapsed) = controller.recent_voice_elapsed_for_generation(generation)
        else {
            return;
        };
        if should_finish_stop_tail_wait(
            elapsed,
            recent_voice_elapsed,
            min_wait,
            quiet_window,
            max_wait,
        ) {
            return;
        }
        let remaining_min_wait = min_wait.saturating_sub(elapsed);
        let poll = Duration::from_millis(STOP_TAIL_POLL_MS);
        thread::sleep(if remaining_min_wait.is_zero() {
            poll
        } else {
            remaining_min_wait.min(poll)
        });
    }
}

fn effective_stop_tail_min_wait(grace_ms: u64) -> Duration {
    Duration::from_millis(grace_ms.max(STOP_TAIL_MIN_CAPTURE_MS))
}

fn should_finish_stop_tail_wait(
    elapsed: Duration,
    recent_voice_elapsed: Option<Duration>,
    min_wait: Duration,
    quiet_window: Duration,
    max_wait: Duration,
) -> bool {
    if elapsed >= max_wait {
        return true;
    }
    if elapsed < min_wait {
        return false;
    }
    recent_voice_elapsed
        .map(|voice_elapsed| voice_elapsed >= quiet_window)
        .unwrap_or(true)
}

fn is_processing_phase(phase: SessionPhase) -> bool {
    matches!(
        phase,
        SessionPhase::WaitingFinalResult | SessionPhase::PostEditing | SessionPhase::Pasting
    )
}

fn is_tray_input_active_phase(phase: SessionPhase) -> bool {
    matches!(
        phase,
        SessionPhase::Starting
            | SessionPhase::Recording
            | SessionPhase::Stopping
            | SessionPhase::WaitingFinalResult
            | SessionPhase::PostEditing
            | SessionPhase::Pasting
    )
}

fn state_from_inner(inner: &InnerSession) -> SessionState {
    SessionState {
        recording: inner.recording,
        phase: inner.phase,
        message: if matches!(inner.phase, SessionPhase::Recording) {
            if let Some(audio) = &inner.audio_capture {
                let info = audio.info();
                format!(
                    "Recording from {} at {} Hz / {} channel(s), {} PCM bytes captured.",
                    info.device_name, info.sample_rate, info.channels, info.pcm_bytes
                )
            } else {
                "Recording is active, waiting for audio stream.".to_string()
            }
        } else {
            inner.message.clone()
        },
        error_code: inner.error_code.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        effective_stop_tail_min_wait, should_finish_stop_tail_wait, SessionController, SessionPhase,
    };
    use std::time::Duration;

    #[test]
    fn stop_keeps_generation_valid_for_post_processing() {
        let controller = SessionController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.recording = true;
            inner.phase = SessionPhase::Recording;
            inner.generation = 7;
        }

        let stopped = controller.force_stop_generation(
            7,
            SessionPhase::WaitingFinalResult,
            "Recording stopped.",
            None,
        );

        assert!(stopped.is_some());
        assert!(controller.is_current_generation(7));
        assert!(controller
            .set_phase_for_generation(7, None, SessionPhase::PostEditing, "Post-editing.", None)
            .is_some());
        assert!(controller
            .finish_generation(7, None, SessionPhase::Succeeded, "Done.", None)
            .is_some());
    }

    #[test]
    fn stale_worker_cannot_mutate_new_session() {
        let controller = SessionController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.recording = true;
            inner.phase = SessionPhase::Recording;
            inner.message = "Recording started.".to_string();
            inner.generation = 2;
        }

        assert!(controller
            .set_phase_for_generation(1, None, SessionPhase::PostEditing, "Stale update.", None)
            .is_none());
        assert!(controller
            .finish_generation(
                1,
                None,
                SessionPhase::Failed,
                "Stale failure.",
                Some("STALE")
            )
            .is_none());

        let state = controller.current_state();
        assert!(state.recording);
        assert_eq!(state.phase, SessionPhase::Recording);
        assert_eq!(state.error_code, None);
    }

    #[test]
    fn audio_error_failure_invalidates_current_asr_worker() {
        let controller = SessionController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.recording = true;
            inner.phase = SessionPhase::Recording;
            inner.message = "Recording started.".to_string();
            inner.generation = 9;
        }

        let (state, guard_generation) = controller
            .fail_recording_generation_and_invalidate(
                9,
                "Microphone stream failed.",
                "MIC_STREAM_FAILED",
            )
            .unwrap();

        assert!(!state.recording);
        assert_eq!(state.phase, SessionPhase::Failed);
        assert_eq!(state.error_code.as_deref(), Some("MIC_STREAM_FAILED"));
        assert!(!controller.is_current_generation(9));
        assert!(controller.is_current_generation(guard_generation));
        assert!(controller
            .set_phase_for_generation(9, None, SessionPhase::Pasting, "Stale output.", None)
            .is_none());
    }

    #[test]
    fn processing_phases_block_new_start_attempts() {
        assert!(super::is_processing_phase(SessionPhase::WaitingFinalResult));
        assert!(super::is_processing_phase(SessionPhase::PostEditing));
        assert!(super::is_processing_phase(SessionPhase::Pasting));
        assert!(!super::is_processing_phase(SessionPhase::Idle));
        assert!(!super::is_processing_phase(SessionPhase::Recording));
    }

    #[test]
    fn tray_icon_marks_input_and_processing_phases_active() {
        for phase in [
            SessionPhase::Starting,
            SessionPhase::Recording,
            SessionPhase::Stopping,
            SessionPhase::WaitingFinalResult,
            SessionPhase::PostEditing,
            SessionPhase::Pasting,
        ] {
            assert!(super::is_tray_input_active_phase(phase));
        }

        for phase in [
            SessionPhase::Idle,
            SessionPhase::Succeeded,
            SessionPhase::Failed,
        ] {
            assert!(!super::is_tray_input_active_phase(phase));
        }
    }

    #[test]
    fn toggle_ignores_starting_phase() {
        let controller = SessionController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.recording = true;
            inner.phase = SessionPhase::Starting;
            inner.message = "Recording is starting.".to_string();
            inner.generation = 3;
        }

        let state = controller.toggle(None).unwrap();

        assert!(state.recording);
        assert_eq!(state.phase, SessionPhase::Starting);
        assert!(controller.is_current_generation(3));
    }

    #[test]
    fn toggle_ignores_stopping_phase() {
        let controller = SessionController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.recording = true;
            inner.phase = SessionPhase::Stopping;
            inner.message = "Recording is stopping.".to_string();
            inner.generation = 5;
        }

        let state = controller.toggle(None).unwrap();

        assert!(state.recording);
        assert_eq!(state.phase, SessionPhase::Stopping);
        assert!(controller.is_current_generation(5));
    }

    #[test]
    fn toggle_recording_phase_still_allows_stop() {
        let controller = SessionController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.recording = true;
            inner.phase = SessionPhase::Recording;
            inner.message = "Recording started.".to_string();
            inner.generation = 8;
        }

        let state = controller.toggle(None).unwrap();

        assert!(matches!(
            state.phase,
            SessionPhase::Stopping | SessionPhase::WaitingFinalResult
        ));
    }

    #[test]
    fn generation_stop_with_grace_uses_stopping_phase_before_final_stop() {
        let controller = SessionController::default();
        {
            let mut inner = controller.inner.lock().unwrap();
            inner.recording = true;
            inner.phase = SessionPhase::Recording;
            inner.message = "Recording started.".to_string();
            inner.generation = 12;
        }

        let state = controller
            .stop_generation_with_grace(
                None,
                12,
                1,
                "Recording stopped.",
                "Recording stopped after grace period.",
                "测试停止录音",
            )
            .unwrap();

        assert!(state.recording);
        assert_eq!(state.phase, SessionPhase::Stopping);
        assert!(controller.is_current_generation(12));

        std::thread::sleep(std::time::Duration::from_millis(10));
        let state = controller.current_state();
        assert!(!state.recording);
        assert_eq!(state.phase, SessionPhase::WaitingFinalResult);
    }

    #[test]
    fn stop_tail_wait_does_not_finish_before_min_wait() {
        assert!(!should_finish_stop_tail_wait(
            Duration::from_millis(120),
            None,
            Duration::from_millis(150),
            Duration::from_millis(150),
            Duration::from_millis(300),
        ));
    }

    #[test]
    fn effective_stop_tail_min_wait_has_internal_floor() {
        assert_eq!(effective_stop_tail_min_wait(50), Duration::from_millis(150));
        assert_eq!(
            effective_stop_tail_min_wait(800),
            Duration::from_millis(800)
        );
        assert_eq!(
            effective_stop_tail_min_wait(2_500),
            Duration::from_millis(2_500)
        );
    }

    #[test]
    fn stop_tail_wait_extends_when_voice_is_recent() {
        assert!(!should_finish_stop_tail_wait(
            Duration::from_millis(200),
            Some(Duration::from_millis(80)),
            Duration::from_millis(150),
            Duration::from_millis(150),
            Duration::from_millis(300),
        ));
    }

    #[test]
    fn stop_tail_wait_finishes_after_quiet_tail() {
        assert!(should_finish_stop_tail_wait(
            Duration::from_millis(250),
            Some(Duration::from_millis(160)),
            Duration::from_millis(150),
            Duration::from_millis(150),
            Duration::from_millis(300),
        ));
    }

    #[test]
    fn stop_tail_wait_forces_finish_at_max_wait() {
        assert!(should_finish_stop_tail_wait(
            Duration::from_millis(300),
            Some(Duration::from_millis(20)),
            Duration::from_millis(150),
            Duration::from_millis(150),
            Duration::from_millis(300),
        ));
    }
}
