use crate::config::effective_hotwords;
use crate::session::{SessionController, SessionPhase};
use crate::{app_log, asr, asr_ws, audio, config::AppConfig};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::future::Future;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue, Request};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use uuid::Uuid;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const TASK_STARTED_TIMEOUT: Duration = Duration::from_secs(8);
const TEST_FINISHED_TIMEOUT: Duration = Duration::from_secs(8);
const RESPONSE_POLL_TIMEOUT: Duration = Duration::from_millis(20);
const CONTEXT_MESSAGE_LIMIT: usize = 5;
const CONTEXT_MESSAGE_MAX_CHARS: usize = 400;
const CONTEXT_HOTWORD_LIMIT: usize = 80;
const ALIYUN_CONNECT_TIMEOUT_MESSAGE: &str = "连接阿里云 ASR 超时，请检查网络或代理后重试。";
const ALIYUN_FINAL_TIMEOUT_MESSAGE: &str = "等待阿里云 ASR 最终结果超时，请检查网络后重试。";
const ALIYUN_CONNECTION_CLOSED_MESSAGE: &str =
    "阿里云 ASR 连接已结束，但未返回完整最终结果。请重试，或检查网络稳定性。";

#[derive(Debug, Clone, PartialEq, Eq)]
enum AliyunServerEvent {
    TaskStarted,
    ResultGenerated {
        text: String,
        sentence_end: bool,
    },
    TaskFinished,
    TaskFailed {
        error_code: String,
        error_message: String,
    },
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProviderSurfaceEvent {
    Started,
    PartialText(String),
    StableText(String),
    Finished,
}

#[derive(Debug, Default)]
struct AliyunFinalGate {
    task_started: bool,
    task_finished: bool,
    final_sentences: Vec<String>,
    failed: Option<String>,
}

impl AliyunFinalGate {
    fn apply(&mut self, event: AliyunServerEvent) -> Option<ProviderSurfaceEvent> {
        match event {
            AliyunServerEvent::TaskStarted => {
                self.task_started = true;
                Some(ProviderSurfaceEvent::Started)
            }
            AliyunServerEvent::ResultGenerated { text, sentence_end } => {
                let text = normalize_live_text(&text);
                if text.trim().is_empty() {
                    return None;
                }
                if sentence_end {
                    self.final_sentences.push(text.clone());
                    Some(ProviderSurfaceEvent::StableText(text))
                } else {
                    Some(ProviderSurfaceEvent::PartialText(text))
                }
            }
            AliyunServerEvent::TaskFinished => {
                self.task_finished = true;
                Some(ProviderSurfaceEvent::Finished)
            }
            AliyunServerEvent::TaskFailed {
                error_code,
                error_message,
            } => {
                self.failed = Some(friendly_task_failed_error(&error_code, &error_message));
                None
            }
            AliyunServerEvent::Unknown(_) => None,
        }
    }

    fn final_text(&self) -> Result<Option<String>, String> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }
        if !self.task_finished {
            return Ok(None);
        }
        Ok(Some(
            self.final_sentences
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join("")
                .trim()
                .to_string(),
        ))
    }
}

pub(crate) async fn test_connection(config: &AppConfig) -> Result<(), String> {
    validate_configured(config)?;

    let mut test_config = config.clone();
    test_config.context.hotwords.clear();
    test_config.context.prompt_context.clear();
    test_config.context.recent_context.clear();

    let request = build_client_request(&test_config)?;
    let (mut websocket, _) = await_connect(
        connect_async(request),
        CONNECT_TIMEOUT,
        ALIYUN_CONNECT_TIMEOUT_MESSAGE,
    )
    .await?;
    let task_id = Uuid::new_v4().to_string();
    send_run_task(&mut websocket, &task_id, &test_config, Vec::new()).await?;
    wait_for_task_started(&mut websocket, TASK_STARTED_TIMEOUT).await?;
    websocket
        .send(Message::Binary(silent_test_audio(&test_config).into()))
        .await
        .map_err(|err| format!("发送阿里云 ASR 测试音频失败: {}", err))?;
    send_finish_task(&mut websocket, &task_id).await?;
    wait_for_task_finished(&mut websocket, TEST_FINISHED_TIMEOUT).await?;
    let _ = websocket.close(None).await;
    Ok(())
}

pub(crate) async fn recognize_stream(
    config: AppConfig,
    audio_rx: Receiver<Vec<u8>>,
    app: AppHandle,
    session: SessionController,
    generation: u64,
    screen_context: Option<&str>,
) -> Result<String, String> {
    validate_configured(&config)?;

    let request = build_client_request(&config)?;
    let (mut websocket, _) = await_connect(
        connect_async(request),
        CONNECT_TIMEOUT,
        ALIYUN_CONNECT_TIMEOUT_MESSAGE,
    )
    .await?;
    app_log::info("阿里云 ASR WebSocket 已连接");

    let task_id = Uuid::new_v4().to_string();
    let context = build_context_payload(&config, screen_context);
    send_run_task(&mut websocket, &task_id, &config, context).await?;
    wait_for_task_started(&mut websocket, TASK_STARTED_TIMEOUT).await?;
    app_log::info("阿里云 ASR 任务已启动");

    let mut pending_audio: VecDeque<Vec<u8>> = VecDeque::new();
    let mut audio_pacer = AudioSendPacer::new();
    let mut audio_input_closed = false;
    let mut finish_task_sent = false;
    let mut final_wait_started: Option<Instant> = None;
    let final_timeout =
        Duration::from_secs_f64(config.request.final_result_timeout_seconds.max(0.5));
    let mut gate = AliyunFinalGate {
        task_started: true,
        ..AliyunFinalGate::default()
    };

    loop {
        if !audio_input_closed {
            loop {
                match audio_rx.try_recv() {
                    Ok(chunk) => {
                        if !chunk.is_empty() {
                            pending_audio.push_back(chunk);
                        }
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        audio_input_closed = true;
                        break;
                    }
                }
            }
        }

        if !finish_task_sent && audio_pacer.ready_to_send() {
            if let Some(chunk) = pending_audio.pop_front() {
                websocket
                    .send(Message::Binary(chunk.clone().into()))
                    .await
                    .map_err(|err| format!("发送阿里云 ASR 音频失败: {}", err))?;
                audio_pacer.mark_sent_bytes(chunk.len());
            } else if audio_input_closed {
                send_finish_task(&mut websocket, &task_id).await?;
                finish_task_sent = true;
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
                app_log::info("阿里云 ASR finish-task 已发送");
            }
        }

        let poll_timeout = if finish_task_sent {
            RESPONSE_POLL_TIMEOUT
        } else {
            audio_pacer.response_poll_timeout(RESPONSE_POLL_TIMEOUT)
        };
        if let Some(event) = poll_server_event(&mut websocket, poll_timeout).await? {
            if let Some(surface_event) = gate.apply(event) {
                match surface_event {
                    ProviderSurfaceEvent::PartialText(text)
                    | ProviderSurfaceEvent::StableText(text) => {
                        asr_ws::emit_partial_text(&app, &text);
                    }
                    ProviderSurfaceEvent::Finished => break,
                    ProviderSurfaceEvent::Started => {}
                }
            }
        }

        if finish_task_sent
            && final_wait_started
                .map(|started| started.elapsed() >= final_timeout)
                .unwrap_or(false)
        {
            return Err(ALIYUN_FINAL_TIMEOUT_MESSAGE.to_string());
        }
    }

    Ok(asr::normalize_final_text(
        &gate.final_text()?.unwrap_or_default(),
        config.typing.remove_trailing_period,
    ))
}

fn validate_configured(config: &AppConfig) -> Result<(), String> {
    if config.aliyun_asr.api_key.trim().is_empty() {
        return Err("请先填写阿里云 ASR API Key。".to_string());
    }
    if config.aliyun_asr.model.trim().is_empty() {
        return Err("请先填写阿里云 ASR 模型名。".to_string());
    }
    if config.aliyun_asr.workspace_id.trim().is_empty()
        && config.aliyun_asr.websocket_url.trim().is_empty()
    {
        return Err("请先填写阿里云 ASR Workspace ID，或填写自定义 WebSocket 地址。".to_string());
    }
    Ok(())
}

fn build_client_request(config: &AppConfig) -> Result<Request<()>, String> {
    let websocket_url = build_websocket_url(config);
    let mut request = websocket_url
        .as_str()
        .into_client_request()
        .map_err(|err| format!("创建阿里云 ASR WebSocket 请求失败: {}", err))?;
    request.headers_mut().insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_str(&format!("Bearer {}", config.aliyun_asr.api_key.trim()))
            .map_err(|err| format!("阿里云 ASR Authorization header 无效: {}", err))?,
    );
    let workspace_id = config.aliyun_asr.workspace_id.trim();
    if !workspace_id.is_empty() {
        request.headers_mut().insert(
            HeaderName::from_static("x-dashscope-workspace"),
            HeaderValue::from_str(workspace_id)
                .map_err(|err| format!("阿里云 ASR Workspace header 无效: {}", err))?,
        );
    }
    Ok(request)
}

fn build_websocket_url(config: &AppConfig) -> String {
    let custom = config.aliyun_asr.websocket_url.trim();
    if !custom.is_empty() {
        return custom.to_string();
    }
    format!(
        "wss://{}.{}.maas.aliyuncs.com/api-ws/v1/inference",
        config.aliyun_asr.workspace_id.trim(),
        config.aliyun_asr.region.trim()
    )
}

fn build_run_task_payload(task_id: &str, config: &AppConfig, context: Vec<Value>) -> Value {
    let mut parameters = json!({
        "format": "pcm",
        "sample_rate": audio::ASR_OUTPUT_SAMPLE_RATE,
        "semantic_punctuation_enabled": config.aliyun_asr.semantic_punctuation_enabled,
        "max_sentence_silence": config.aliyun_asr.max_sentence_silence,
    });
    if !config.aliyun_asr.language_hint.trim().is_empty() {
        parameters["language_hints"] = json!([config.aliyun_asr.language_hint.trim()]);
    }
    if !config.aliyun_asr.vocabulary_id.trim().is_empty() {
        parameters["vocabulary_id"] = json!(config.aliyun_asr.vocabulary_id.trim());
    }
    let input = if context.is_empty() {
        json!({})
    } else {
        json!({ "context": context })
    };

    json!({
        "header": {
            "action": "run-task",
            "task_id": task_id,
            "streaming": "duplex",
        },
        "payload": {
            "task_group": "audio",
            "task": "asr",
            "function": "recognition",
            "model": config.aliyun_asr.model,
            "parameters": parameters,
            "input": input,
        }
    })
}

fn build_finish_task_payload(task_id: &str) -> Value {
    json!({
        "header": {
            "action": "finish-task",
            "task_id": task_id,
            "streaming": "duplex",
        },
        "payload": {
            "input": {}
        }
    })
}

fn build_context_payload(config: &AppConfig, screen_context: Option<&str>) -> Vec<Value> {
    let hotwords = effective_hotwords(config)
        .into_iter()
        .filter_map(|word| non_empty(&word))
        .take(CONTEXT_HOTWORD_LIMIT)
        .collect::<Vec<_>>();
    let hotword_block = if hotwords.is_empty() {
        None
    } else {
        Some(format!("Recognition terms:\n{}", hotwords.join("\n")))
    };
    let screen_block = screen_context.and_then(non_empty).map(|text| {
        format!(
            "Screen OCR context. Use only to correct names, file names, code identifiers and UI terms:\n{}",
            text
        )
    });

    let mut blocks = Vec::new();
    if let Some(block) = hotword_block {
        blocks.push(block);
    }

    let reserved_for_screen = usize::from(screen_block.is_some());
    let middle_limit = CONTEXT_MESSAGE_LIMIT.saturating_sub(blocks.len() + reserved_for_screen);
    for block in recent_and_prompt_context_blocks(config)
        .into_iter()
        .take(middle_limit)
    {
        blocks.push(block);
    }
    if let Some(block) = screen_block {
        blocks.push(block);
    }

    blocks
        .into_iter()
        .take(CONTEXT_MESSAGE_LIMIT)
        .map(|text| {
            json!({
                "role": "user",
                "content": [{
                    "type": "input_text",
                    "text": cap_chars(&text, CONTEXT_MESSAGE_MAX_CHARS),
                }]
            })
        })
        .collect()
}

fn recent_and_prompt_context_blocks(config: &AppConfig) -> Vec<String> {
    let mut blocks = Vec::new();
    if config.context.enable_recent_context {
        let mut recent = config
            .context
            .recent_context
            .iter()
            .rev()
            .filter_map(|item| non_empty(&item.text))
            .take(2)
            .collect::<Vec<_>>();
        recent.reverse();
        for item in recent {
            blocks.push(format!("Recent voice input:\n{}", item));
        }
    }
    for item in config
        .context
        .prompt_context
        .iter()
        .filter_map(|item| non_empty(&item.text))
        .take(2)
    {
        blocks.push(format!("Writing context:\n{}", item));
    }
    blocks
}

async fn send_run_task<S>(
    websocket: &mut S,
    task_id: &str,
    config: &AppConfig,
    context: Vec<Value>,
) -> Result<(), String>
where
    S: Sink<Message, Error = WsError> + Unpin,
{
    websocket
        .send(Message::Text(
            build_run_task_payload(task_id, config, context)
                .to_string()
                .into(),
        ))
        .await
        .map_err(|err| format!("发送阿里云 ASR run-task 失败: {}", err))
}

async fn send_finish_task<S>(websocket: &mut S, task_id: &str) -> Result<(), String>
where
    S: Sink<Message, Error = WsError> + Unpin,
{
    websocket
        .send(Message::Text(
            build_finish_task_payload(task_id).to_string().into(),
        ))
        .await
        .map_err(|err| format!("发送阿里云 ASR finish-task 失败: {}", err))
}

async fn wait_for_task_started<S>(websocket: &mut S, timeout: Duration) -> Result<(), String>
where
    S: Stream<Item = Result<Message, WsError>> + Unpin,
{
    let started_at = Instant::now();
    loop {
        if started_at.elapsed() >= timeout {
            return Err("阿里云 ASR 已连接，但未收到 task-started。".to_string());
        }
        let poll_timeout = (timeout - started_at.elapsed()).min(Duration::from_millis(250));
        if let Some(event) = poll_server_event(websocket, poll_timeout).await? {
            match event {
                AliyunServerEvent::TaskStarted => return Ok(()),
                AliyunServerEvent::TaskFailed {
                    error_code,
                    error_message,
                } => return Err(friendly_task_failed_error(&error_code, &error_message)),
                _ => {}
            }
        }
    }
}

async fn wait_for_task_finished<S>(websocket: &mut S, timeout: Duration) -> Result<(), String>
where
    S: Stream<Item = Result<Message, WsError>> + Unpin,
{
    let started_at = Instant::now();
    loop {
        if started_at.elapsed() >= timeout {
            return Err("阿里云 ASR 已连接，但测试未收到 task-finished。".to_string());
        }
        let poll_timeout = (timeout - started_at.elapsed()).min(Duration::from_millis(250));
        if let Some(event) = poll_server_event(websocket, poll_timeout).await? {
            match event {
                AliyunServerEvent::TaskFinished => return Ok(()),
                AliyunServerEvent::TaskFailed {
                    error_code,
                    error_message,
                } => return Err(friendly_task_failed_error(&error_code, &error_message)),
                _ => {}
            }
        }
    }
}

async fn poll_server_event<S>(
    websocket: &mut S,
    timeout_duration: Duration,
) -> Result<Option<AliyunServerEvent>, String>
where
    S: Stream<Item = Result<Message, WsError>> + Unpin,
{
    let message = match tokio::time::timeout(timeout_duration, websocket.next()).await {
        Ok(Some(Ok(message))) => message,
        Ok(Some(Err(err))) => return Err(format!("阿里云 ASR 连接已中断: {}", err)),
        Ok(None) => return Err(ALIYUN_CONNECTION_CLOSED_MESSAGE.to_string()),
        Err(_) => return Ok(None),
    };
    match message {
        Message::Text(text) => parse_server_event(text.as_ref()).map(Some),
        Message::Ping(_) | Message::Pong(_) | Message::Frame(_) | Message::Binary(_) => Ok(None),
        Message::Close(_) => Err(ALIYUN_CONNECTION_CLOSED_MESSAGE.to_string()),
    }
}

async fn await_connect<F, T, E>(
    connect_future: F,
    timeout: Duration,
    timeout_message: &str,
) -> Result<T, String>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    tokio::time::timeout(timeout, connect_future)
        .await
        .map_err(|_| timeout_message.to_string())?
        .map_err(|err| friendly_connection_error(&err.to_string()))
}

fn parse_server_event(raw: &str) -> Result<AliyunServerEvent, String> {
    let value: Value =
        serde_json::from_str(raw).map_err(|err| format!("解析阿里云 ASR 响应失败: {}", err))?;
    let event = value
        .pointer("/header/event")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match event {
        "task-started" => Ok(AliyunServerEvent::TaskStarted),
        "result-generated" => {
            let sentence = value
                .pointer("/payload/output/sentence")
                .ok_or_else(|| "阿里云 ASR 响应缺少 sentence。".to_string())?;
            let text = sentence
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let sentence_end = sentence
                .get("sentence_end")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            Ok(AliyunServerEvent::ResultGenerated { text, sentence_end })
        }
        "task-finished" => Ok(AliyunServerEvent::TaskFinished),
        "task-failed" => Ok(AliyunServerEvent::TaskFailed {
            error_code: value
                .pointer("/header/error_code")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN")
                .to_string(),
            error_message: value
                .pointer("/header/error_message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        }),
        _ => Ok(AliyunServerEvent::Unknown(event.to_string())),
    }
}

fn friendly_connection_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("401")
        || lower.contains("403")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
    {
        "阿里云 ASR 认证失败，请检查 API Key、Workspace ID、模型和服务权限。".to_string()
    } else if lower.contains("dns")
        || lower.contains("resolve")
        || lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection")
        || lower.contains("connect")
        || lower.contains("proxy")
        || lower.contains("tls")
    {
        "无法连接阿里云 ASR 服务，请检查网络、代理或防火墙设置。".to_string()
    } else {
        "连接阿里云 ASR 失败，请检查网络环境和阿里云认证配置。".to_string()
    }
}

fn friendly_task_failed_error(error_code: &str, error_message: &str) -> String {
    let lower_code = error_code.to_ascii_lowercase();
    let lower_message = error_message.to_ascii_lowercase();
    if lower_code.contains("auth")
        || lower_code.contains("permission")
        || lower_message.contains("unauthorized")
        || lower_message.contains("forbidden")
        || lower_message.contains("api key")
        || lower_message.contains("workspace")
    {
        return "阿里云 ASR 认证失败，请检查 API Key、Workspace ID、模型和服务权限。".to_string();
    }
    let detail = sanitize_service_error(error_message);
    if detail.is_empty() {
        format!(
            "阿里云 ASR 服务返回错误 {}。请稍后重试，或检查阿里云控制台配置。",
            error_code
        )
    } else {
        format!(
            "阿里云 ASR 服务返回错误 {}：{}。请稍后重试，或检查阿里云控制台配置。",
            error_code, detail
        )
    }
}

fn sanitize_service_error(message: &str) -> String {
    let mut tokens = message
        .split_whitespace()
        .take(24)
        .map(|token| {
            if token.starts_with("sk-") {
                "sk-***".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if tokens.is_empty() {
        tokens = message.chars().take(160).collect();
    }
    tokens.chars().take(160).collect()
}

fn normalize_live_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn cap_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn silent_test_audio(config: &AppConfig) -> Vec<u8> {
    let bytes_per_second =
        audio::ASR_OUTPUT_SAMPLE_RATE as usize * audio::ASR_OUTPUT_CHANNELS as usize * 2;
    let requested = bytes_per_second
        .saturating_mul(audio::effective_asr_segment_ms(config.audio.segment_ms) as usize)
        / 1000;
    vec![0; requested.clamp(3_200, 32_000)]
}

struct AudioSendPacer {
    next_send_at: Option<Instant>,
}

impl AudioSendPacer {
    fn new() -> Self {
        Self { next_send_at: None }
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
        self.next_send_at = Some(Instant::now() + interval_for_audio_bytes(byte_len));
    }
}

fn interval_for_audio_bytes(byte_len: usize) -> Duration {
    let bytes_per_second =
        audio::ASR_OUTPUT_SAMPLE_RATE as u64 * audio::ASR_OUTPUT_CHANNELS as u64 * 2;
    let duration_ms = (byte_len as u64)
        .saturating_mul(1000)
        .div_ceil(bytes_per_second)
        .clamp(audio::ASR_MIN_SEGMENT_MS, audio::ASR_MAX_SEGMENT_MS);
    Duration::from_millis(duration_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, ASR_PROVIDER_ALIYUN_FUN};

    fn configured_aliyun() -> AppConfig {
        let mut config = AppConfig::default();
        config.asr.provider = ASR_PROVIDER_ALIYUN_FUN.to_string();
        config.aliyun_asr.api_key = "sk-test-secret".to_string();
        config.aliyun_asr.workspace_id = "workspace-a".to_string();
        config
    }

    #[test]
    fn websocket_url_uses_workspace_and_region_or_custom_url() {
        let mut config = configured_aliyun();
        assert_eq!(
            build_websocket_url(&config),
            "wss://workspace-a.cn-beijing.maas.aliyuncs.com/api-ws/v1/inference"
        );

        config.aliyun_asr.region = "ap-southeast-1".to_string();
        assert_eq!(
            build_websocket_url(&config),
            "wss://workspace-a.ap-southeast-1.maas.aliyuncs.com/api-ws/v1/inference"
        );

        config.aliyun_asr.websocket_url = "wss://custom.example/asr".to_string();
        assert_eq!(build_websocket_url(&config), "wss://custom.example/asr");
    }

    #[test]
    fn run_task_payload_has_required_shape_and_no_secret() {
        let mut config = configured_aliyun();
        config.aliyun_asr.language_hint = "zh".to_string();
        config.aliyun_asr.vocabulary_id = "vocab-1".to_string();
        config.context.hotwords = vec!["VoxType".to_string()];
        let context = build_context_payload(&config, Some("settings panel"));
        let payload = build_run_task_payload("task-id", &config, context);
        let serialized = serde_json::to_string(&payload).unwrap();

        assert_eq!(payload["header"]["action"], "run-task");
        assert_eq!(payload["header"]["streaming"], "duplex");
        assert_eq!(payload["payload"]["task_group"], "audio");
        assert_eq!(payload["payload"]["task"], "asr");
        assert_eq!(payload["payload"]["function"], "recognition");
        assert_eq!(payload["payload"]["model"], "fun-asr-realtime");
        assert_eq!(payload["payload"]["parameters"]["format"], "pcm");
        assert_eq!(
            payload["payload"]["parameters"]["sample_rate"],
            audio::ASR_OUTPUT_SAMPLE_RATE
        );
        assert_eq!(payload["payload"]["parameters"]["language_hints"][0], "zh");
        assert_eq!(payload["payload"]["parameters"]["vocabulary_id"], "vocab-1");
        assert!(!serialized.contains("sk-test-secret"));
    }

    #[test]
    fn context_payload_is_capped_to_documented_limits() {
        let mut config = configured_aliyun();
        config.context.hotwords = (0..100).map(|index| format!("term-{index}")).collect();
        config.context.enable_recent_context = true;
        config.context.recent_context = vec![
            crate::config::TextContext {
                text: "recent one".repeat(80),
            },
            crate::config::TextContext {
                text: "recent two".repeat(80),
            },
        ];
        config.context.prompt_context = vec![
            crate::config::TextContext {
                text: "prompt one".repeat(80),
            },
            crate::config::TextContext {
                text: "prompt two".repeat(80),
            },
        ];
        let screen = "screen ocr ".repeat(80);

        let context = build_context_payload(&config, Some(&screen));

        assert!(context.len() <= CONTEXT_MESSAGE_LIMIT);
        for item in context {
            let text = item["content"][0]["text"].as_str().unwrap();
            assert!(text.chars().count() <= CONTEXT_MESSAGE_MAX_CHARS);
            assert_eq!(item["role"], "user");
            assert_eq!(item["content"][0]["type"], "input_text");
        }
    }

    #[test]
    fn intermediate_text_is_display_only_until_task_finished() {
        let mut gate = AliyunFinalGate::default();
        assert_eq!(
            gate.apply(parse_server_event(r#"{"header":{"event":"task-started"}}"#).unwrap()),
            Some(ProviderSurfaceEvent::Started)
        );

        let partial = r#"{
            "header": {"event": "result-generated"},
            "payload": {"output": {"sentence": {"text": "中间字幕", "sentence_end": false}}}
        }"#;
        assert_eq!(
            gate.apply(parse_server_event(partial).unwrap()),
            Some(ProviderSurfaceEvent::PartialText("中间字幕".to_string()))
        );
        assert_eq!(gate.final_text().unwrap(), None);

        let stable = r#"{
            "header": {"event": "result-generated"},
            "payload": {"output": {"sentence": {"text": "最终文本", "sentence_end": true}}}
        }"#;
        assert_eq!(
            gate.apply(parse_server_event(stable).unwrap()),
            Some(ProviderSurfaceEvent::StableText("最终文本".to_string()))
        );
        assert_eq!(gate.final_text().unwrap(), None);

        assert_eq!(
            gate.apply(parse_server_event(r#"{"header":{"event":"task-finished"}}"#).unwrap()),
            Some(ProviderSurfaceEvent::Finished)
        );
        assert_eq!(gate.final_text().unwrap(), Some("最终文本".to_string()));
    }

    #[test]
    fn task_failed_maps_to_error_without_raw_payload_or_secret() {
        let mut gate = AliyunFinalGate::default();
        let failed = r#"{
            "header": {
                "event": "task-failed",
                "error_code": "CLIENT_ERROR",
                "error_message": "request timeout for sk-test-secret after 23 seconds with verbose diagnostic body that should be shortened"
            },
            "payload": {"debug": "large internal body"}
        }"#;
        gate.apply(parse_server_event(failed).unwrap());
        let error = gate.final_text().unwrap_err();
        assert!(error.contains("CLIENT_ERROR"));
        assert!(!error.contains("large internal body"));
        assert!(!error.contains("debug"));
        assert!(!error.contains("sk-test-secret"));
        assert!(error.contains("sk-***"));
    }

    #[test]
    fn audio_pacer_uses_documented_packet_bounds() {
        assert_eq!(
            interval_for_audio_bytes(3_200),
            Duration::from_millis(audio::ASR_MIN_SEGMENT_MS)
        );
        assert_eq!(
            interval_for_audio_bytes(6_400),
            Duration::from_millis(audio::ASR_MAX_SEGMENT_MS)
        );
    }
}
