use crate::session::{SessionController, SessionPhase};
use crate::{
    app_log, asr, audio, config, config::AppConfig, hotword_history, llm_post_edit, overlay,
    protocol, screen_context, stats, text_output,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone, Serialize)]
pub struct AsrFinalText {
    pub text: String,
    pub error: Option<String>,
    pub error_code: Option<String>,
    pub warning: Option<String>,
    pub warning_code: Option<String>,
}

const ATTENTION_OVERLAY_HOLD: Duration = Duration::from_millis(1_800);
// 近期实测表明：速度优化应优先放在响应轮询和字幕节流，而不是缩短默认 200ms ASR 音频包。
// 豆包双向流式对 100-200ms 音频包更稳；这里的 50/20ms 只影响本地显示和收包等待。
// 维护依据见 docs/asr-quality-latency-guardrails.md。
const PARTIAL_TEXT_MIN_INTERVAL: Duration = Duration::from_millis(50);
const RESPONSE_POLL_TIMEOUT: Duration = Duration::from_millis(20);
const POST_EDITING_OVERLAY_DELAY: Duration = Duration::from_millis(450);
// 收到最终包后短暂 settle，给二遍修正一次补尾机会；不回退到直接接受中间结果。
const FINAL_PACKET_SETTLE: Duration = Duration::from_millis(300);
// 头部保护并入第一包真实音频，避免独立短包破坏豆包推荐的发包节奏。
const INITIAL_AUDIO_SILENCE_PADDING_MS: u64 = 50;
const ASR_FINAL_TIMEOUT_MESSAGE: &str = "等待豆包 ASR 最终结果超时，请检查网络后重试。";
const ASR_FINAL_INCOMPLETE_MESSAGE: &str =
    "豆包 ASR 连接已结束，但未返回完整最终结果。请重试，或检查网络稳定性。";

pub fn spawn_asr_worker(
    config: AppConfig,
    audio_rx: Receiver<Vec<u8>>,
    started_at: Instant,
    app: AppHandle,
    session: SessionController,
    generation: u64,
    screen_context_rx: Option<screen_context::ScreenContextReceiver>,
) {
    thread::spawn(move || {
        app_log::info(format!("ASR worker 已启动: generation={}", generation));
        if config.auth.app_key.trim().is_empty() || config.auth.access_key.trim().is_empty() {
            let error = "ASR skipped: app_key/access_key is not configured.".to_string();
            if session.abort_generation_from_worker_with_code(
                &app,
                generation,
                &error,
                "ASR_AUTH_MISSING",
            ) {
                emit_error(&app, &session, generation, "ASR_AUTH_MISSING", error);
            }
            return;
        }

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => {
                let error = format!("启动 ASR 运行时失败: {}", err);
                if session
                    .finish_generation(
                        generation,
                        Some(&app),
                        SessionPhase::Failed,
                        &error,
                        Some("ASR_NETWORK_FAILED"),
                    )
                    .is_some()
                {
                    emit_error(&app, &session, generation, "ASR_NETWORK_FAILED", error);
                }
                return;
            }
        };
        let typing = config.typing.clone();
        let remove_trailing_period = config.typing.remove_trailing_period;
        let screen_context =
            screen_context::wait_for_context(screen_context_rx, config.screen_context.timeout_ms);
        let screen_context_text = screen_context.as_ref().map(|item| item.text.clone());
        let runtime_result = runtime.block_on(async {
            let text = run_websocket_session(
                config.clone(),
                audio_rx,
                app.clone(),
                session.clone(),
                generation,
                screen_context_text.as_deref(),
            )
            .await?;
            if text.trim().is_empty() {
                return Ok::<llm_post_edit::PolishOutcome, String>(llm_post_edit::PolishOutcome {
                    text,
                    warning: Some("EMPTY_TRANSCRIPT".to_string()),
                });
            }
            if llm_post_edit::should_polish(&config, &text) {
                Ok::<llm_post_edit::PolishOutcome, String>(
                    polish_with_delayed_status(
                        &config,
                        &text,
                        screen_context_text.as_deref(),
                        &app,
                        &session,
                        generation,
                    )
                    .await,
                )
            } else {
                Ok::<llm_post_edit::PolishOutcome, String>(llm_post_edit::PolishOutcome {
                    text,
                    warning: None,
                })
            }
        });
        match runtime_result {
            Ok(outcome) => {
                let text = asr::normalize_final_text(&outcome.text, remove_trailing_period);
                let llm_warning = outcome.warning;
                let mut should_hold_overlay = false;
                if !session.is_current_generation(generation) {
                    app_log::info(format!(
                        "忽略过期 ASR worker 输出: generation={}, chars={}",
                        generation,
                        text.chars().count()
                    ));
                    return;
                }
                app_log::info(format!("ASR worker 返回文本长度: {}", text.chars().count()));
                if text.trim().is_empty() {
                    handle_empty_transcript(&app, &session, generation);
                    return;
                }
                if !text.trim().is_empty() {
                    overlay::update_text(&app, &text);
                    let duration = started_at.elapsed().as_secs_f64();
                    if session
                        .set_phase_for_generation(
                            generation,
                            Some(&app),
                            SessionPhase::Pasting,
                            "Pasting transcript.",
                            None,
                        )
                        .is_none()
                    {
                        return;
                    }
                    let llm_warning_for_final = llm_warning.clone();
                    let mut output_sent_finalized = false;
                    let (output_warning, output_warning_code) =
                        match text_output::output_text(&text, &typing, || {
                            if !output_sent_finalized {
                                output_sent_finalized =
                                    finish_output_sent_session(&session, generation, Some(&app));
                                if output_sent_finalized {
                                    overlay::hide(&app);
                                    record_successful_transcript_side_effects(
                                        &app, &text, duration,
                                    );
                                    emit_successful_final_text(
                                        &app,
                                        &text,
                                        llm_warning_for_final.clone(),
                                        None,
                                    );
                                    app_log::info(format!(
                                        "ASR session output sent: chars={}",
                                        text.chars().count()
                                    ));
                                }
                            }
                        }) {
                            Ok(result) => (result.warning, result.warning_code),
                            Err(err) => {
                                let error_code = if err.contains("剪贴板") {
                                    "CLIPBOARD_WRITE_FAILED"
                                } else {
                                    "PASTE_FAILED"
                                };
                                if session.abort_generation_from_worker_with_code(
                                    &app, generation, &err, error_code,
                                ) {
                                    emit_error(&app, &session, generation, error_code, err);
                                }
                                return;
                            }
                        };
                    if !output_sent_finalized {
                        output_sent_finalized =
                            finish_output_sent_session(&session, generation, Some(&app));
                        if output_sent_finalized {
                            record_successful_transcript_side_effects(&app, &text, duration);
                            emit_successful_final_text(&app, &text, llm_warning.clone(), None);
                        }
                    }
                    should_hold_overlay = should_hold_overlay_for_output_warning(
                        output_warning.as_deref(),
                        output_warning_code.as_deref(),
                    );
                    if session.is_current_generation(generation) {
                        if should_hold_overlay {
                            if let Some(warning) = output_warning.as_deref() {
                                overlay::show_message(&app, &config.ui, warning);
                            }
                            if llm_warning.is_none() {
                                emit_successful_final_text(
                                    &app,
                                    &text,
                                    output_warning.clone(),
                                    output_warning_code.clone(),
                                );
                            }
                        } else {
                            overlay::hide(&app);
                        }
                    }
                    if output_warning.is_some() {
                        app_log::warn(format!(
                            "输出文本完成但存在提示: {}",
                            output_warning.as_deref().unwrap_or_default()
                        ));
                    }
                    app_log::info(format!(
                        "ASR session finished: chars={}",
                        text.chars().count()
                    ));
                }
                if should_hold_overlay {
                    thread::sleep(ATTENTION_OVERLAY_HOLD);
                }
                if session.is_current_generation(generation) {
                    overlay::hide(&app);
                }
            }
            Err(err) => {
                let error_code = classify_asr_error(&err);
                if session
                    .abort_generation_from_worker_with_code(&app, generation, &err, error_code)
                {
                    emit_error(&app, &session, generation, error_code, err);
                }
            }
        }
    });
}

pub async fn test_connection(config: &AppConfig) -> Result<(), String> {
    if config.auth.app_key.trim().is_empty() || config.auth.access_key.trim().is_empty() {
        return Err("请先填写豆包 App Key 和 Access Key。".to_string());
    }
    if config.auth.resource_id.trim().is_empty() {
        return Err("请先填写豆包 Resource ID。".to_string());
    }

    let mut test_config = config.clone();
    test_config.context.hotwords.clear();
    test_config.context.prompt_context.clear();
    test_config.context.recent_context.clear();
    let preview = asr::build_request_preview(&test_config, None);
    let mut request = preview
        .ws_url
        .as_str()
        .into_client_request()
        .map_err(|err| format!("创建豆包 ASR 测试请求失败: {}", err))?;
    for (name, value) in preview.headers {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|err| format!("豆包 ASR header 名称无效: {}", err))?;
        let value = HeaderValue::from_str(&value)
            .map_err(|err| format!("豆包 ASR header 值无效: {}", err))?;
        request.headers_mut().insert(name, value);
    }

    let (mut websocket, _) = tokio::time::timeout(Duration::from_secs(20), connect_async(request))
        .await
        .map_err(|_| "连接豆包 ASR 测试超时，请检查网络或代理设置。".to_string())?
        .map_err(|err| friendly_asr_connection_error(&err.to_string()))?;
    websocket
        .send(Message::Binary(
            protocol::build_full_request(&preview.payload, 1)?.into(),
        ))
        .await
        .map_err(|err| format!("发送豆包 ASR 测试首包失败: {}", err))?;
    let test_audio = silent_test_audio(&test_config);
    websocket
        .send(Message::Binary(
            protocol::build_audio_request(2, &test_audio, false)?.into(),
        ))
        .await
        .map_err(|err| format!("发送豆包 ASR 测试音频包失败: {}", err))?;
    websocket
        .send(Message::Binary(
            protocol::build_audio_request(3, &[], true)?.into(),
        ))
        .await
        .map_err(|err| format!("发送豆包 ASR 测试结束包失败: {}", err))?;

    let response = tokio::time::timeout(Duration::from_secs(8), websocket.next())
        .await
        .map_err(|_| "豆包 ASR 已连接，但未收到测试响应，请稍后重试。".to_string())?;
    let Some(response) = response else {
        return Err("豆包 ASR 连接已关闭，未收到测试响应。".to_string());
    };
    match response {
        Ok(Message::Binary(data)) => {
            let parsed = protocol::parse_response(&data)?;
            if is_success_code(parsed.code) {
                let _ = websocket.close(None).await;
                Ok(())
            } else {
                Err(friendly_asr_service_error(parsed.code))
            }
        }
        Ok(Message::Close(_)) => Err("豆包 ASR 连接已关闭，未收到有效测试响应。".to_string()),
        Ok(_) => Err("豆包 ASR 返回了非预期测试响应。".to_string()),
        Err(err) => Err(format!("接收豆包 ASR 测试响应失败: {}", err)),
    }
}

async fn polish_with_delayed_status(
    config: &AppConfig,
    text: &str,
    screen_context: Option<&str>,
    app: &AppHandle,
    session: &SessionController,
    generation: u64,
) -> llm_post_edit::PolishOutcome {
    let config = config.clone();
    let original_text = text.to_string();
    let task_text = original_text.clone();
    let screen_context = screen_context.map(str::to_string);
    let mut polish_task = tokio::spawn(async move {
        llm_post_edit::polish(&config, &task_text, screen_context.as_deref()).await
    });

    match tokio::time::timeout(POST_EDITING_OVERLAY_DELAY, &mut polish_task).await {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(err)) => {
            app_log::warn(format!("大模型润色任务异常: {}", err));
            llm_post_edit::PolishOutcome {
                text: original_text,
                warning: Some("大模型润色任务异常，已使用原文。".to_string()),
            }
        }
        Err(_) => {
            if session
                .set_phase_for_generation(
                    generation,
                    Some(app),
                    SessionPhase::PostEditing,
                    "Post-editing transcript.",
                    None,
                )
                .is_some()
            {
                overlay::update_text(app, overlay::POST_EDITING_TEXT);
            }
            match polish_task.await {
                Ok(outcome) => outcome,
                Err(err) => {
                    app_log::warn(format!("大模型润色任务异常: {}", err));
                    llm_post_edit::PolishOutcome {
                        text: original_text,
                        warning: Some("大模型润色任务异常，已使用原文。".to_string()),
                    }
                }
            }
        }
    }
}

fn silent_test_audio(config: &AppConfig) -> Vec<u8> {
    let bytes_per_second =
        audio::ASR_OUTPUT_SAMPLE_RATE as usize * audio::ASR_OUTPUT_CHANNELS as usize * 2;
    let requested = bytes_per_second
        .saturating_mul(audio::effective_asr_segment_ms(config.audio.segment_ms) as usize)
        / 1000;
    let byte_len = requested.clamp(3_200, 32_000);
    vec![0; byte_len]
}

async fn run_websocket_session(
    config: AppConfig,
    audio_rx: Receiver<Vec<u8>>,
    app: AppHandle,
    session: SessionController,
    generation: u64,
    screen_context: Option<&str>,
) -> Result<String, String> {
    let remove_trailing_period = config.typing.remove_trailing_period;
    let preview = asr::build_request_preview(&config, screen_context);
    let mut request = preview
        .ws_url
        .as_str()
        .into_client_request()
        .map_err(|err| format!("创建 ASR WebSocket 请求失败: {}", err))?;
    for (name, value) in preview.headers {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|err| format!("ASR header 名称无效: {}", err))?;
        let value =
            HeaderValue::from_str(&value).map_err(|err| format!("ASR header 值无效: {}", err))?;
        request.headers_mut().insert(name, value);
    }

    let (mut websocket, _) = connect_async(request).await.map_err(|err| {
        let detail = err.to_string();
        let message = friendly_asr_connection_error(&detail);
        app_log::warn(format!(
            "连接 ASR WebSocket 失败: {}; user_message={}",
            detail, message
        ));
        message
    })?;
    app_log::info("ASR WebSocket 已连接");
    websocket
        .send(Message::Binary(
            protocol::build_full_request(&preview.payload, 1)?.into(),
        ))
        .await
        .map_err(|err| format!("发送 ASR 首包失败: {}", err))?;
    app_log::info("ASR 首包已发送");

    let mut seq = 2;
    let mut audio_pacer = AudioSendPacer::new();
    // 只在开头补很短静音；结束依赖停录尾音等待和 flush，不再额外补静音。
    let mut pending_audio = AsrAudioQueue::new(&config);
    let mut audio_input_closed = false;
    let mut end_packet_pending = false;
    let mut audio_finished = false;
    let mut final_wait_started: Option<Instant> = None;
    let final_timeout =
        Duration::from_secs_f64(config.request.final_result_timeout_seconds.max(0.5));
    let mut display_text = String::new();
    let mut final_packet_text: Option<String> = None;
    let mut definitive_segments = Vec::new();
    let mut partial_limiter = PartialTextLimiter::new();
    let mut connection_closed_before_final = false;
    let mut final_packet_settle_started: Option<Instant> = None;

    loop {
        if !audio_finished {
            if !audio_input_closed {
                loop {
                    match audio_rx.try_recv() {
                        Ok(chunk) => pending_audio.push_real_audio(chunk),
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            audio_input_closed = true;
                            pending_audio.close_input();
                            end_packet_pending = true;
                            break;
                        }
                    }
                }
            }

            if audio_pacer.ready_to_send() {
                if let Some(chunk) = pending_audio.pop_front() {
                    let is_last_audio_chunk = end_packet_pending && pending_audio.is_empty();
                    websocket
                        .send(Message::Binary(
                            protocol::build_audio_request(seq, &chunk, is_last_audio_chunk)?.into(),
                        ))
                        .await
                        .map_err(|err| format!("发送 ASR 音频包失败: {}", err))?;
                    if is_last_audio_chunk {
                        audio_finished = true;
                        end_packet_pending = false;
                        final_wait_started = Some(Instant::now());
                        if session
                            .set_phase_for_generation(
                                generation,
                                Some(&app),
                                SessionPhase::WaitingFinalResult,
                                "Waiting for final ASR result.",
                                None,
                            )
                            .is_none()
                        {
                            return Err("ASR session expired.".to_string());
                        }
                        app_log::info("ASR 最后一包音频已发送");
                    } else {
                        audio_pacer.mark_sent_bytes(chunk.len());
                    }
                    seq += 1;
                } else if end_packet_pending {
                    websocket
                        .send(Message::Binary(
                            protocol::build_audio_request(seq, &[], true)?.into(),
                        ))
                        .await
                        .map_err(|err| format!("发送 ASR 结束包失败: {}", err))?;
                    audio_finished = true;
                    end_packet_pending = false;
                    final_wait_started = Some(Instant::now());
                    if session
                        .set_phase_for_generation(
                            generation,
                            Some(&app),
                            SessionPhase::WaitingFinalResult,
                            "Waiting for final ASR result.",
                            None,
                        )
                        .is_none()
                    {
                        return Err("ASR session expired.".to_string());
                    }
                    app_log::info("ASR 音频结束包已发送");
                }
            }
        }

        let response_poll_timeout =
            websocket_response_poll_timeout(audio_finished, &audio_pacer, RESPONSE_POLL_TIMEOUT);
        match tokio::time::timeout(response_poll_timeout, websocket.next()).await {
            Ok(Some(Ok(Message::Binary(data)))) => {
                let parsed = protocol::parse_response(&data)?;
                if !is_success_code(parsed.code) {
                    return Err(friendly_asr_service_error(parsed.code));
                }
                let packet_text =
                    normalize_live_text(&asr::extract_display_text(parsed.payload_msg.as_ref()));
                let final_packet_candidate =
                    normalize_live_text(&asr::extract_final_text(parsed.payload_msg.as_ref()));
                let settling_after_final = final_packet_settle_started.is_some();
                let live_packet_text_seen = !packet_text.is_empty();
                if live_packet_text_seen && packet_text != display_text {
                    display_text = packet_text.clone();
                    if let Some(text) = partial_limiter.emit_or_defer(&display_text) {
                        emit_partial_text(&app, &text);
                    }
                }
                let mut final_update_seen = false;
                for segment in asr::extract_definite_segments(parsed.payload_msg.as_ref()) {
                    if upsert_definite_segment(&mut definitive_segments, segment) {
                        final_update_seen = true;
                        definitive_segments.sort_by_key(|item| (item.start_time, item.end_time));
                        let text = definitive_segments
                            .iter()
                            .map(|item| item.text.as_str())
                            .collect::<Vec<_>>()
                            .join("");
                        if !live_packet_text_seen && !text.trim().is_empty() {
                            let normalized =
                                asr::normalize_final_text(&text, remove_trailing_period);
                            if let Some(text) = partial_limiter.emit_or_defer(&normalized) {
                                emit_partial_text(&app, &text);
                            }
                        }
                    }
                }
                if settling_after_final {
                    if !final_packet_candidate.is_empty() {
                        final_packet_text = Some(final_packet_candidate.clone());
                        final_update_seen = true;
                    }
                    if final_update_seen {
                        final_packet_settle_started = Some(Instant::now());
                    }
                }
                if parsed.is_last_package {
                    final_packet_text = Some(final_packet_candidate);
                    final_packet_settle_started = Some(Instant::now());
                }
            }
            Ok(Some(Ok(Message::Close(_)))) => {
                connection_closed_before_final = final_packet_text.is_none();
                break;
            }
            Ok(Some(Ok(_))) | Err(_) => {}
            Ok(Some(Err(err))) => return Err(format!("接收 ASR 响应失败: {}", err)),
            Ok(None) => {
                connection_closed_before_final = final_packet_text.is_none();
                break;
            }
        }
        if let Some(text) = partial_limiter.emit_pending_if_ready() {
            emit_partial_text(&app, &text);
        }

        if should_timeout_waiting_final(
            final_packet_text.is_some(),
            final_wait_started.map(|started| started.elapsed()),
            final_timeout,
        ) {
            return Err(ASR_FINAL_TIMEOUT_MESSAGE.to_string());
        }
        if should_finish_final_packet_settle(
            final_packet_settle_started.map(|started| started.elapsed()),
        ) {
            break;
        }
    }

    if final_packet_text.is_none() {
        return Err(if connection_closed_before_final {
            ASR_FINAL_INCOMPLETE_MESSAGE.to_string()
        } else {
            ASR_FINAL_TIMEOUT_MESSAGE.to_string()
        });
    }

    select_final_output_text(
        &definitive_segments,
        final_packet_text.as_deref(),
        &display_text,
        remove_trailing_period,
    )
}

fn upsert_definite_segment(
    segments: &mut Vec<asr::DefiniteSegment>,
    segment: asr::DefiniteSegment,
) -> bool {
    if let Some(existing) = segments
        .iter_mut()
        .find(|item| item.start_time == segment.start_time && item.end_time == segment.end_time)
    {
        let changed = existing.text != segment.text;
        *existing = segment;
        return changed;
    }

    segments.push(segment);
    true
}

struct AudioSendPacer {
    next_send_at: Option<Instant>,
}

struct AsrAudioQueue {
    pending_packets: VecDeque<Vec<u8>>,
    buffered: Vec<u8>,
    leading_padding: Vec<u8>,
    first_packet_bytes: usize,
    regular_packet_bytes: usize,
    real_audio_seen: bool,
    first_packet_sent: bool,
    closed: bool,
}

impl AsrAudioQueue {
    fn new(config: &AppConfig) -> Self {
        let regular_packet_bytes =
            asr_pcm_bytes_for_ms(audio::effective_asr_segment_ms(config.audio.segment_ms)).max(2)
                as usize;
        let first_packet_bytes = regular_packet_bytes;
        Self {
            pending_packets: VecDeque::new(),
            buffered: Vec::new(),
            leading_padding: vec![
                0;
                asr_pcm_bytes_for_ms(INITIAL_AUDIO_SILENCE_PADDING_MS) as usize
            ],
            first_packet_bytes,
            regular_packet_bytes,
            real_audio_seen: false,
            first_packet_sent: false,
            closed: false,
        }
    }

    fn push_real_audio(&mut self, chunk: Vec<u8>) {
        if chunk.is_empty() || self.closed {
            return;
        }
        if !self.real_audio_seen {
            self.real_audio_seen = true;
            if !self.leading_padding.is_empty() {
                self.buffered
                    .extend(std::mem::take(&mut self.leading_padding));
            }
        }
        self.buffered.extend(chunk);
        self.drain_complete_packets();
    }

    fn close_input(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.drain_complete_packets();
        if !self.buffered.is_empty() {
            self.pending_packets
                .push_back(std::mem::take(&mut self.buffered));
        }
    }

    fn pop_front(&mut self) -> Option<Vec<u8>> {
        self.pending_packets.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.pending_packets.is_empty()
    }

    fn drain_complete_packets(&mut self) {
        loop {
            let target = self.next_packet_bytes();
            if self.buffered.len() < target {
                break;
            }
            let packet = self.buffered.drain(..target).collect::<Vec<_>>();
            self.pending_packets.push_back(packet);
            self.first_packet_sent = true;
        }
    }

    fn next_packet_bytes(&self) -> usize {
        if self.first_packet_sent {
            self.regular_packet_bytes
        } else {
            self.first_packet_bytes
        }
    }
}

impl AudioSendPacer {
    fn new() -> Self {
        Self { next_send_at: None }
    }

    fn interval_for_audio_bytes(byte_len: usize) -> Duration {
        Duration::from_millis(
            asr_pcm_duration_ms_for_bytes(byte_len)
                .clamp(audio::ASR_MIN_SEGMENT_MS, audio::ASR_MAX_SEGMENT_MS),
        )
    }

    fn ready_to_send(&self) -> bool {
        self.next_send_at
            .map(|next_send_at| Instant::now() >= next_send_at)
            .unwrap_or(true)
    }

    fn response_poll_timeout(&self, default_timeout: Duration) -> Duration {
        let Some(next_send_at) = self.next_send_at else {
            return default_timeout;
        };
        let wait = next_send_at
            .checked_duration_since(Instant::now())
            .unwrap_or_default();
        if wait.is_zero() {
            Duration::from_millis(1)
        } else {
            wait.min(default_timeout)
        }
    }

    fn mark_sent_bytes(&mut self, byte_len: usize) {
        self.next_send_at = Some(Instant::now() + Self::interval_for_audio_bytes(byte_len));
    }
}

fn websocket_response_poll_timeout(
    audio_finished: bool,
    audio_pacer: &AudioSendPacer,
    default_timeout: Duration,
) -> Duration {
    if audio_finished {
        default_timeout
    } else {
        audio_pacer.response_poll_timeout(default_timeout)
    }
}

fn asr_pcm_bytes_for_ms(duration_ms: u64) -> u64 {
    if duration_ms == 0 {
        return 0;
    }
    audio::ASR_OUTPUT_SAMPLE_RATE as u64 * audio::ASR_OUTPUT_CHANNELS as u64 * 2 * duration_ms
        / 1000
}

fn asr_pcm_duration_ms_for_bytes(byte_len: usize) -> u64 {
    let bytes_per_second =
        audio::ASR_OUTPUT_SAMPLE_RATE as u64 * audio::ASR_OUTPUT_CHANNELS as u64 * 2;
    (byte_len as u64)
        .saturating_mul(1000)
        .div_ceil(bytes_per_second)
}

fn should_finish_final_packet_settle(elapsed: Option<Duration>) -> bool {
    elapsed
        .map(|elapsed| elapsed >= FINAL_PACKET_SETTLE)
        .unwrap_or(false)
}

fn should_timeout_waiting_final(
    final_packet_received: bool,
    elapsed: Option<Duration>,
    timeout: Duration,
) -> bool {
    !final_packet_received && elapsed.map(|elapsed| elapsed >= timeout).unwrap_or(false)
}

fn select_final_output_text(
    definitive_segments: &[asr::DefiniteSegment],
    final_packet_text: Option<&str>,
    _live_caption_text: &str,
    remove_trailing_period: bool,
) -> Result<String, String> {
    let Some(text) = final_packet_text else {
        return Err(ASR_FINAL_TIMEOUT_MESSAGE.to_string());
    };

    let final_text = asr::normalize_final_text(text, remove_trailing_period);
    if !definitive_segments.is_empty() {
        let mut segments = definitive_segments.to_vec();
        segments.sort_by_key(|item| (item.start_time, item.end_time));
        let text = segments
            .into_iter()
            .map(|item| item.text)
            .collect::<Vec<_>>()
            .join("");
        let definitive_text = asr::normalize_final_text(&text, remove_trailing_period);
        if final_text_covers_definitive_text(&final_text, &definitive_text) {
            return Ok(final_text);
        }
        return Ok(definitive_text);
    }

    Ok(final_text)
}

// 最终输出仍必须来自豆包最终包；definite 分句用于稳定性，但不能压掉更完整的最终包。
// 0.1.102 的回归经验：final 包补齐尾字时，前文可能相对 definite 分句有小幅改写。
// 因此先接受严格包含，再用高重合度兜底；完全不相关的更长 final 包仍会被拒绝。
// 改这里时同步 docs/asr-quality-latency-guardrails.md，并先补 final_output_ 回归测试。
fn final_text_covers_definitive_text(final_text: &str, definitive_text: &str) -> bool {
    let compact_final = compact_for_final_prefix(final_text);
    let compact_definitive = compact_for_final_prefix(definitive_text);
    if compact_final.is_empty() || compact_definitive.is_empty() {
        return false;
    }

    let final_len = compact_final.chars().count();
    let definitive_len = compact_definitive.chars().count();
    if final_len <= definitive_len {
        return false;
    }
    if compact_final.contains(&compact_definitive) {
        return true;
    }

    // 豆包 final 包可能补尾字的同时轻微改写前文；高重合且更长时应信任 final 包。
    common_subsequence_len(&compact_final, &compact_definitive) * 100 >= definitive_len * 75
}

fn common_subsequence_len(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    if right_chars.is_empty() {
        return 0;
    }
    let mut dp = vec![0; right_chars.len() + 1];
    for left_char in left.chars() {
        let mut previous = 0;
        for (index, right_char) in right_chars.iter().enumerate() {
            let saved = dp[index + 1];
            dp[index + 1] = if left_char == *right_char {
                previous + 1
            } else {
                dp[index + 1].max(dp[index])
            };
            previous = saved;
        }
    }
    dp[right_chars.len()]
}

fn compact_for_final_prefix(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace() && !ch.is_ascii_punctuation() && !is_cjk_punctuation(*ch))
        .collect()
}

fn is_cjk_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '。' | '，'
            | '、'
            | '；'
            | '：'
            | '？'
            | '！'
            | '“'
            | '”'
            | '‘'
            | '’'
            | '（'
            | '）'
            | '《'
            | '》'
            | '【'
            | '】'
    )
}

fn emit_partial_text(app: &AppHandle, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    overlay::update_text(app, text.to_string());
}

fn record_successful_transcript_side_effects(app: &AppHandle, text: &str, duration: f64) {
    if let Err(err) = config::remember_recent_context(text) {
        app_log::warn(format!("写入 recent context 失败: {}", err));
    }
    if let Err(err) = hotword_history::append_transcript(text) {
        app_log::warn(format!("写入自动热词历史失败: {}", err));
    }
    if let Err(err) = stats::append_event(text, duration) {
        app_log::warn(err);
    } else if let Err(err) = app.emit("usage-stats-updated", stats::load_stats_snapshot()) {
        app_log::warn(format!("刷新统计事件发送失败: {}", err));
    }
}

fn finish_output_sent_session(
    session: &SessionController,
    generation: u64,
    app: Option<&AppHandle>,
) -> bool {
    session
        .finish_generation(
            generation,
            app,
            SessionPhase::Succeeded,
            "Transcript output completed.",
            None,
        )
        .is_some()
}

fn emit_successful_final_text(
    app: &AppHandle,
    text: &str,
    warning: Option<String>,
    warning_code: Option<String>,
) {
    let _ = app.emit(
        "asr-final-text",
        AsrFinalText {
            text: text.to_string(),
            error: None,
            error_code: None,
            warning,
            warning_code,
        },
    );
}

fn should_hold_overlay_for_output_warning(
    warning: Option<&str>,
    warning_code: Option<&str>,
) -> bool {
    warning.is_some() && !text_output::is_quiet_output_warning_code(warning_code)
}

struct PartialTextLimiter {
    last_emit_at: Option<Instant>,
    last_text: String,
    pending_text: Option<String>,
}

impl PartialTextLimiter {
    fn new() -> Self {
        Self {
            last_emit_at: None,
            last_text: String::new(),
            pending_text: None,
        }
    }

    fn emit_or_defer(&mut self, text: &str) -> Option<String> {
        let text = text.trim();
        if text.is_empty() || text == self.last_text {
            return None;
        }
        if self.can_emit_now() {
            return Some(self.mark_emitted(text.to_string()));
        }
        self.pending_text = Some(text.to_string());
        None
    }

    fn emit_pending_if_ready(&mut self) -> Option<String> {
        if !self.can_emit_now() {
            return None;
        }
        let text = self.pending_text.take()?;
        if text.trim().is_empty() || text == self.last_text {
            return None;
        }
        Some(self.mark_emitted(text))
    }

    fn can_emit_now(&self) -> bool {
        self.last_emit_at
            .map(|last| last.elapsed() >= PARTIAL_TEXT_MIN_INTERVAL)
            .unwrap_or(true)
    }

    fn mark_emitted(&mut self, text: String) -> String {
        self.last_emit_at = Some(Instant::now());
        self.last_text = text.clone();
        self.pending_text = None;
        text
    }
}

fn normalize_live_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn handle_empty_transcript(app: &AppHandle, session: &SessionController, generation: u64) {
    app_log::info("ASR session finished: empty transcript");
    if session
        .finish_generation(
            generation,
            Some(app),
            SessionPhase::Failed,
            overlay::EMPTY_TRANSCRIPT_TEXT,
            Some("EMPTY_TRANSCRIPT"),
        )
        .is_none()
    {
        return;
    }
    if session.is_current_generation(generation) {
        overlay::update_text(app, overlay::EMPTY_TRANSCRIPT_TEXT);
    }
    let _ = app.emit(
        "asr-final-text",
        AsrFinalText {
            text: String::new(),
            error: Some(overlay::EMPTY_TRANSCRIPT_TEXT.to_string()),
            error_code: Some("EMPTY_TRANSCRIPT".to_string()),
            warning: None,
            warning_code: None,
        },
    );
    thread::sleep(ATTENTION_OVERLAY_HOLD);
    if session.is_current_generation(generation) {
        overlay::hide(app);
    }
}

fn emit_error(
    app: &AppHandle,
    session: &SessionController,
    generation: u64,
    error_code: &str,
    error: String,
) {
    app_log::warn(&error);
    let message = if error_code == "PASTE_FAILED" {
        overlay::PASTE_FAILED_TEXT.to_string()
    } else {
        format!("识别失败: {}", error)
    };
    if session.is_current_generation(generation) {
        overlay::update_text(app, message);
    }
    let _ = app.emit(
        "asr-final-text",
        AsrFinalText {
            text: String::new(),
            error: Some(error),
            error_code: Some(error_code.to_string()),
            warning: None,
            warning_code: None,
        },
    );
    thread::sleep(ATTENTION_OVERLAY_HOLD);
    if session.is_current_generation(generation) {
        overlay::hide(app);
    }
}

fn classify_asr_error(error: &str) -> &'static str {
    if error.contains("认证")
        || error.contains("权限")
        || error.contains("App Key")
        || error.contains("Access Key")
        || error.contains("Resource ID")
    {
        "ASR_AUTH_MISSING"
    } else if error.contains("超时") || error.to_ascii_lowercase().contains("timeout") {
        "ASR_FINAL_TIMEOUT"
    } else {
        "ASR_NETWORK_FAILED"
    }
}

fn is_success_code(code: i32) -> bool {
    code == 0 || code == 20_000_000
}

fn friendly_asr_connection_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("401")
        || lower.contains("403")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
    {
        "豆包 ASR 认证失败，请检查 App Key、Access Key 和 Resource ID。".to_string()
    } else if lower.contains("dns")
        || lower.contains("resolve")
        || lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection")
        || lower.contains("connect")
        || lower.contains("proxy")
        || lower.contains("tls")
    {
        "无法连接豆包 ASR 服务，请检查网络、代理或防火墙设置。".to_string()
    } else {
        "连接豆包 ASR 失败，请检查网络环境和豆包认证配置。".to_string()
    }
}

fn friendly_asr_service_error(code: i32) -> String {
    if (400..500).contains(&code) || (40_000_000..50_000_000).contains(&code) {
        format!(
            "豆包 ASR 认证或权限校验失败，错误码 {}。请检查 App Key、Access Key、Resource ID 和服务权限。",
            code
        )
    } else {
        format!(
            "豆包 ASR 服务返回错误码 {}。请稍后重试，或检查网络与豆包控制台配置。",
            code
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        asr_pcm_bytes_for_ms, finish_output_sent_session, friendly_asr_connection_error,
        friendly_asr_service_error, is_success_code, select_final_output_text,
        should_finish_final_packet_settle, should_hold_overlay_for_output_warning,
        should_timeout_waiting_final, silent_test_audio, upsert_definite_segment,
        websocket_response_poll_timeout, AsrAudioQueue, AudioSendPacer, PartialTextLimiter,
        PARTIAL_TEXT_MIN_INTERVAL, RESPONSE_POLL_TIMEOUT,
    };
    use crate::text_output::WARNING_CLIPBOARD_PARTIAL_RESTORE;
    use crate::{asr::DefiniteSegment, config::AppConfig};
    use std::time::{Duration, Instant};

    #[test]
    fn accepts_doubao_success_codes() {
        assert!(is_success_code(0));
        assert!(is_success_code(20_000_000));
        assert!(!is_success_code(400));
    }

    #[test]
    fn explains_common_asr_failures() {
        assert!(friendly_asr_connection_error("HTTP error: 401 Unauthorized").contains("认证失败"));
        assert!(friendly_asr_connection_error("dns error").contains("无法连接"));
        assert!(friendly_asr_service_error(40_000_001).contains("权限"));
    }

    #[test]
    fn silent_test_audio_is_small_and_non_empty() {
        let config = AppConfig::default();
        let audio = silent_test_audio(&config);
        assert!(!audio.is_empty());
        assert!(audio.len() >= 3_200);
        assert!(audio.len() <= 32_000);
        assert!(audio.iter().all(|value| *value == 0));
    }

    #[test]
    fn final_output_uses_definite_segments() {
        let segments = vec![DefiniteSegment {
            text: "二遍完整结果。".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text =
            select_final_output_text(&segments, Some("初步结果。"), "初步结果。", true).unwrap();

        assert_eq!(text, "二遍完整结果");
    }

    #[test]
    fn final_output_uses_last_package_text_when_it_extends_definite_segments() {
        let segments = vec![DefiniteSegment {
            text: "按下结束键之后，最后说话".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("按下结束键之后，最后说话还是没有被识别下来。"),
            "按下结束键之后，最后说话还是没有被识别下来。",
            true,
        )
        .unwrap();

        assert_eq!(text, "按下结束键之后，最后说话还是没有被识别下来");
    }

    #[test]
    fn final_output_uses_last_package_text_when_it_adds_missing_head() {
        let segments = vec![DefiniteSegment {
            text: "头两个字可能被截断".to_string(),
            start_time: 200,
            end_time: 1200,
        }];

        let text = select_final_output_text(
            &segments,
            Some("开头的头两个字可能被截断。"),
            "开头的头两个字可能被截断。",
            true,
        )
        .unwrap();

        assert_eq!(text, "开头的头两个字可能被截断");
    }

    #[test]
    fn final_output_uses_last_package_text_when_punctuation_differs() {
        let segments = vec![DefiniteSegment {
            text: "按下结束键之后，最后说话".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("按下结束键之后最后说话还是没有被识别下来。"),
            "按下结束键之后最后说话还是没有被识别下来。",
            true,
        )
        .unwrap();

        assert_eq!(text, "按下结束键之后最后说话还是没有被识别下来");
    }

    #[test]
    fn final_output_uses_last_package_text_when_tail_is_added_after_minor_rewrite() {
        let segments = vec![DefiniteSegment {
            text: "我觉得这个配置可以".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("我觉得这项配置可以生效。"),
            "我觉得这项配置可以生效。",
            true,
        )
        .unwrap();

        assert_eq!(text, "我觉得这项配置可以生效");
    }

    #[test]
    fn final_output_keeps_definite_segments_when_last_package_does_not_cover_them() {
        let segments = vec![DefiniteSegment {
            text: "完整的二遍分句".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(&segments, Some("不相关最终包"), "不相关最终包", false)
            .unwrap();

        assert_eq!(text, "完整的二遍分句");
    }

    #[test]
    fn final_output_keeps_definite_segments_when_longer_last_package_is_unrelated() {
        let segments = vec![DefiniteSegment {
            text: "完整的二遍分句".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("这是一个完全不相关但是更长的最终包"),
            "这是一个完全不相关但是更长的最终包",
            false,
        )
        .unwrap();

        assert_eq!(text, "完整的二遍分句");
    }

    #[test]
    fn definite_segment_update_replaces_earlier_text_for_same_time_range() {
        let mut segments = vec![DefiniteSegment {
            text: "早期结果".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        assert!(upsert_definite_segment(
            &mut segments,
            DefiniteSegment {
                text: "最终修正结果".to_string(),
                start_time: 0,
                end_time: 1000,
            },
        ));

        let text = select_final_output_text(&segments, Some(""), "", false).unwrap();
        assert_eq!(text, "最终修正结果");
    }

    #[test]
    fn final_output_rejects_definite_segments_without_last_package() {
        let segments = vec![DefiniteSegment {
            text: "缺少尾部的二遍结果".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let err = select_final_output_text(&segments, None, "实时字幕", false).unwrap_err();

        assert!(err.contains("最终结果"));
    }

    #[test]
    fn final_output_rejects_interim_text_without_last_package() {
        let err = select_final_output_text(&[], None, "初步结果", false).unwrap_err();

        assert!(err.contains("最终结果"));
    }

    #[test]
    fn final_output_uses_last_package_text_without_definite_segments() {
        let text = select_final_output_text(&[], Some("最终结果"), "最终结果", false).unwrap();

        assert_eq!(text, "最终结果");
    }

    #[test]
    fn initial_audio_padding_keeps_first_packet_at_configured_segment_size() {
        let mut config = AppConfig::default();
        config.audio.segment_ms = 200;
        let real_audio = vec![7; asr_pcm_bytes_for_ms(200) as usize];
        let mut queue = AsrAudioQueue::new(&config);

        queue.push_real_audio(real_audio);
        let first = queue.pop_front().unwrap();
        queue.close_input();
        let tail = queue.pop_front().unwrap();

        assert_eq!(first.len(), asr_pcm_bytes_for_ms(200) as usize);
        assert!(first[..asr_pcm_bytes_for_ms(50) as usize]
            .iter()
            .all(|byte| *byte == 0));
        assert!(first[asr_pcm_bytes_for_ms(50) as usize..]
            .iter()
            .all(|byte| *byte == 7));
        assert_eq!(tail.len(), asr_pcm_bytes_for_ms(50) as usize);
        assert!(tail.iter().all(|byte| *byte == 7));
        assert!(queue.pop_front().is_none());
    }

    #[test]
    fn initial_audio_padding_does_not_repeat_between_real_packets() {
        let config = AppConfig::default();
        let mut queue = AsrAudioQueue::new(&config);

        queue.push_real_audio(vec![7; asr_pcm_bytes_for_ms(200) as usize]);
        let first = queue.pop_front().unwrap();
        queue.push_real_audio(vec![9; asr_pcm_bytes_for_ms(200) as usize]);
        let second = queue.pop_front().unwrap();
        queue.close_input();
        let tail = queue.pop_front().unwrap();

        assert_eq!(first.len(), asr_pcm_bytes_for_ms(200) as usize);
        assert_eq!(second.len(), asr_pcm_bytes_for_ms(200) as usize);
        assert_eq!(tail.len(), asr_pcm_bytes_for_ms(50) as usize);
        assert!(first[..asr_pcm_bytes_for_ms(50) as usize]
            .iter()
            .all(|byte| *byte == 0));
        assert!(second.iter().all(|byte| *byte == 7 || *byte == 9));
        assert!(tail.iter().all(|byte| *byte == 9));
    }

    #[test]
    fn closing_without_real_audio_does_not_send_padding() {
        let config = AppConfig::default();
        let mut queue = AsrAudioQueue::new(&config);

        queue.close_input();

        assert!(queue.pop_front().is_none());
    }

    #[test]
    fn audio_send_pacer_uses_actual_packet_duration_with_documented_bounds() {
        assert_eq!(
            AudioSendPacer::interval_for_audio_bytes(640),
            Duration::from_millis(100)
        );
        assert_eq!(
            AudioSendPacer::interval_for_audio_bytes(3_200),
            Duration::from_millis(100)
        );
        assert_eq!(
            AudioSendPacer::interval_for_audio_bytes(5_120),
            Duration::from_millis(160)
        );
        assert_eq!(
            AudioSendPacer::interval_for_audio_bytes(16_000),
            Duration::from_millis(200)
        );
    }

    #[test]
    fn audio_send_pacer_does_not_block_response_polling_until_next_packet() {
        let mut pacer = AudioSendPacer::new();

        assert!(pacer.ready_to_send());
        assert_eq!(
            pacer.response_poll_timeout(RESPONSE_POLL_TIMEOUT),
            RESPONSE_POLL_TIMEOUT
        );

        pacer.mark_sent_bytes(3_200);

        assert!(!pacer.ready_to_send());
        assert!(pacer.response_poll_timeout(RESPONSE_POLL_TIMEOUT) <= RESPONSE_POLL_TIMEOUT);
    }

    #[test]
    fn final_wait_uses_default_response_poll_timeout_after_audio_finished() {
        let pacer = AudioSendPacer {
            next_send_at: Some(Instant::now() - Duration::from_millis(1)),
        };

        assert_eq!(
            pacer.response_poll_timeout(RESPONSE_POLL_TIMEOUT),
            Duration::from_millis(1)
        );
        assert_eq!(
            websocket_response_poll_timeout(true, &pacer, RESPONSE_POLL_TIMEOUT),
            RESPONSE_POLL_TIMEOUT
        );
    }

    #[test]
    fn partial_text_limiter_allows_faster_live_caption_updates() {
        let mut limiter = PartialTextLimiter::new();

        assert_eq!(PARTIAL_TEXT_MIN_INTERVAL, Duration::from_millis(50));
        assert_eq!(limiter.emit_or_defer("第一段").as_deref(), Some("第一段"));
        assert!(limiter.emit_or_defer("第一段").is_none());
        assert!(limiter.emit_or_defer("").is_none());
    }

    #[test]
    fn partial_text_limiter_coalesces_fast_updates_instead_of_dropping_them() {
        let mut limiter = PartialTextLimiter::new();

        assert_eq!(limiter.emit_or_defer("第一段").as_deref(), Some("第一段"));
        assert!(limiter.emit_or_defer("第一段第二段").is_none());
        assert_eq!(limiter.pending_text.as_deref(), Some("第一段第二段"));
        limiter.last_emit_at = Some(Instant::now() - PARTIAL_TEXT_MIN_INTERVAL);

        assert_eq!(
            limiter.emit_pending_if_ready().as_deref(),
            Some("第一段第二段")
        );
        assert!(limiter.pending_text.is_none());
    }

    #[test]
    fn partial_text_limiter_keeps_latest_fast_update_for_live_caption() {
        let mut limiter = PartialTextLimiter::new();

        assert_eq!(limiter.emit_or_defer("第一段").as_deref(), Some("第一段"));
        assert!(limiter.emit_or_defer("第一段第二段").is_none());
        assert!(limiter.emit_or_defer("第一段第二段第三段").is_none());
        limiter.last_emit_at = Some(Instant::now() - PARTIAL_TEXT_MIN_INTERVAL);

        assert_eq!(
            limiter.emit_pending_if_ready().as_deref(),
            Some("第一段第二段第三段")
        );
    }

    #[test]
    fn final_packet_settle_waits_before_finishing() {
        assert!(!should_finish_final_packet_settle(Some(
            Duration::from_millis(250)
        )));
        assert!(should_finish_final_packet_settle(Some(
            Duration::from_millis(300)
        )));
        assert!(!should_finish_final_packet_settle(None));
    }

    #[test]
    fn final_timeout_only_applies_before_last_package_arrives() {
        assert!(should_timeout_waiting_final(
            false,
            Some(Duration::from_millis(600)),
            Duration::from_millis(500),
        ));
        assert!(!should_timeout_waiting_final(
            true,
            Some(Duration::from_millis(600)),
            Duration::from_millis(500),
        ));
        assert!(!should_timeout_waiting_final(
            false,
            None,
            Duration::from_millis(500),
        ));
    }

    #[test]
    fn empty_last_package_stays_empty_for_failure_flow() {
        let text = select_final_output_text(&[], Some("   "), "初步结果", false).unwrap();

        assert_eq!(text, "");
    }

    #[test]
    fn holds_overlay_only_for_visible_output_warnings() {
        assert!(!should_hold_overlay_for_output_warning(None, None));
        assert!(!should_hold_overlay_for_output_warning(
            Some("quiet"),
            Some(WARNING_CLIPBOARD_PARTIAL_RESTORE),
        ));
        assert!(should_hold_overlay_for_output_warning(
            Some("visible"),
            Some("CLIPBOARD_RESTORE_FAILED"),
        ));
    }

    #[test]
    fn output_sent_finishes_pasting_before_clipboard_cleanup() {
        let session = crate::session::SessionController::default();
        assert!(session
            .set_phase_for_generation(
                0,
                None,
                crate::session::SessionPhase::Pasting,
                "Pasting.",
                None
            )
            .is_some());

        assert!(finish_output_sent_session(&session, 0, None));

        let state = session.current_state();
        assert!(!state.recording);
        assert_eq!(state.phase, crate::session::SessionPhase::Succeeded);
    }
}
