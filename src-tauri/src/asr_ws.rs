use crate::session::{SessionController, SessionPhase};
use crate::{
    app_log, asr, config, config::AppConfig, hotword_history, llm_post_edit, overlay, protocol,
    screen_context, stats, text_output,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
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
const PARTIAL_TEXT_MIN_INTERVAL: Duration = Duration::from_millis(220);
const POST_EDITING_OVERLAY_DELAY: Duration = Duration::from_millis(450);

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
    let bytes_per_second = config.audio.sample_rate as usize * config.audio.channels as usize * 2;
    let requested = bytes_per_second.saturating_mul(config.audio.segment_ms as usize) / 1000;
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
    let mut audio_finished = false;
    let mut final_wait_started: Option<Instant> = None;
    let final_timeout =
        Duration::from_secs_f64(config.request.final_result_timeout_seconds.max(0.5));
    let mut display_text = String::new();
    let mut definitive_segments = Vec::new();
    let mut partial_limiter = PartialTextLimiter::new();

    loop {
        if !audio_finished {
            match audio_rx.try_recv() {
                Ok(chunk) => {
                    websocket
                        .send(Message::Binary(
                            protocol::build_audio_request(seq, &chunk, false)?.into(),
                        ))
                        .await
                        .map_err(|err| format!("发送 ASR 音频包失败: {}", err))?;
                    seq += 1;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    websocket
                        .send(Message::Binary(
                            protocol::build_audio_request(seq, &[], true)?.into(),
                        ))
                        .await
                        .map_err(|err| format!("发送 ASR 结束包失败: {}", err))?;
                    audio_finished = true;
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

        match tokio::time::timeout(Duration::from_millis(40), websocket.next()).await {
            Ok(Some(Ok(Message::Binary(data)))) => {
                let parsed = protocol::parse_response(&data)?;
                if !is_success_code(parsed.code) {
                    return Err(friendly_asr_service_error(parsed.code));
                }
                let partial =
                    normalize_live_text(&asr::extract_display_text(parsed.payload_msg.as_ref()));
                if !partial.is_empty() && partial != display_text {
                    display_text = partial;
                    if partial_limiter.should_emit(&display_text) {
                        emit_partial_text(&app, &display_text);
                    }
                }
                for segment in asr::extract_definite_segments(parsed.payload_msg.as_ref()) {
                    if !definitive_segments
                        .iter()
                        .any(|item: &asr::DefiniteSegment| {
                            item.start_time == segment.start_time
                                && item.end_time == segment.end_time
                        })
                    {
                        definitive_segments.push(segment);
                        definitive_segments.sort_by_key(|item| (item.start_time, item.end_time));
                        let text = definitive_segments
                            .iter()
                            .map(|item| item.text.as_str())
                            .collect::<Vec<_>>()
                            .join("");
                        if !text.trim().is_empty() {
                            let normalized =
                                asr::normalize_final_text(&text, remove_trailing_period);
                            if partial_limiter.should_emit(&normalized) {
                                emit_partial_text(&app, &normalized);
                            }
                        }
                    }
                }
                if parsed.is_last_package {
                    break;
                }
            }
            Ok(Some(Ok(Message::Close(_)))) => break,
            Ok(Some(Ok(_))) | Err(_) => {}
            Ok(Some(Err(err))) => return Err(format!("接收 ASR 响应失败: {}", err)),
            Ok(None) => break,
        }

        if let Some(started) = final_wait_started {
            if started.elapsed() >= final_timeout {
                return Err("等待豆包 ASR 最终结果超时，请检查网络后重试。".to_string());
            }
        }
    }

    if definitive_segments.is_empty() {
        return Ok(asr::normalize_final_text(
            &display_text,
            remove_trailing_period,
        ));
    }
    definitive_segments.sort_by_key(|item| (item.start_time, item.end_time));
    let final_text = definitive_segments
        .into_iter()
        .map(|item| item.text)
        .collect::<Vec<_>>()
        .join("");
    let final_text = asr::normalize_final_text(&final_text, remove_trailing_period);
    if final_text.is_empty() {
        Ok(asr::normalize_final_text(
            &display_text,
            remove_trailing_period,
        ))
    } else {
        Ok(final_text)
    }
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
}

impl PartialTextLimiter {
    fn new() -> Self {
        Self {
            last_emit_at: None,
            last_text: String::new(),
        }
    }

    fn should_emit(&mut self, text: &str) -> bool {
        if text.trim().is_empty() || text == self.last_text {
            return false;
        }
        let now = Instant::now();
        let enough_time = self
            .last_emit_at
            .map(|last| last.elapsed() >= PARTIAL_TEXT_MIN_INTERVAL)
            .unwrap_or(true);
        if !enough_time {
            return false;
        }
        self.last_emit_at = Some(now);
        self.last_text = text.to_string();
        true
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
        finish_output_sent_session, friendly_asr_connection_error, friendly_asr_service_error,
        is_success_code, should_hold_overlay_for_output_warning, silent_test_audio,
    };
    use crate::config::AppConfig;
    use crate::text_output::WARNING_CLIPBOARD_PARTIAL_RESTORE;

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
