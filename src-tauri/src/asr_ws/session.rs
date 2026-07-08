use super::audio_stream::{websocket_response_poll_timeout, AsrAudioQueue, AudioSendPacer};
use super::connection::await_asr_connect;
use super::errors::{friendly_asr_service_error, is_success_code, ASR_FINAL_TIMEOUT_MESSAGE};
use super::final_text::{
    missing_final_result_error, select_final_output_text, should_finish_final_packet_settle,
    should_timeout_waiting_final, upsert_definite_segment,
};
use super::partial_text::{
    emit_partial_text, normalize_live_text, LiveCaptionBuffer, PartialTextLimiter,
};
use crate::session::{SessionController, SessionPhase};
use crate::{app_log, asr, asr_activity::AsrActivityReporter, config::AppConfig, protocol};
use futures_util::{SinkExt, StreamExt};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};
use tokio_tungstenite::tungstenite::Message;

const RESPONSE_POLL_TIMEOUT: Duration = Duration::from_millis(20);
const ASR_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

pub(crate) async fn run_doubao_websocket_session(
    config: AppConfig,
    audio_rx: Receiver<Vec<u8>>,
    app: AppHandle,
    session: SessionController,
    generation: u64,
    screen_context: Option<&str>,
    activity: AsrActivityReporter,
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

    let (mut websocket, _) = await_asr_connect(connect_async(request), ASR_CONNECT_TIMEOUT).await?;
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
    let mut live_caption = LiveCaptionBuffer::new();
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
                let mut definite_feedback_seen = false;
                if live_packet_text_seen && packet_text != display_text {
                    display_text = packet_text.clone();
                    if let Some(caption) = live_caption.update(&display_text) {
                        if let Some(text) = partial_limiter.emit_or_defer(&caption) {
                            emit_partial_text(&app, &text);
                        }
                    }
                }
                let mut final_update_seen = false;
                for segment in asr::extract_definite_segments(parsed.payload_msg.as_ref()) {
                    let segment_has_text = !segment.text.trim().is_empty();
                    if upsert_definite_segment(&mut definitive_segments, segment) {
                        if segment_has_text {
                            definite_feedback_seen = true;
                        }
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
                            if let Some(caption) = live_caption.update(&normalized) {
                                if let Some(text) = partial_limiter.emit_or_defer(&caption) {
                                    emit_partial_text(&app, &text);
                                }
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
                let effective_feedback_seen = doubao_packet_has_effective_feedback(
                    &packet_text,
                    &final_packet_candidate,
                    definite_feedback_seen,
                );
                if parsed.is_last_package {
                    final_packet_text = Some(final_packet_candidate);
                    final_packet_settle_started = Some(Instant::now());
                }
                if effective_feedback_seen {
                    activity.mark_feedback();
                }
            }
            Ok(Some(Ok(Message::Close(_)))) => {
                connection_closed_before_final = final_packet_text.is_none();
                break;
            }
            Ok(Some(Ok(_))) | Err(_) => {}
            Ok(Some(Err(err))) => return Err(format!("豆包 ASR 连接已中断: {}", err)),
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
        return Err(missing_final_result_error(connection_closed_before_final));
    }

    select_final_output_text(
        &definitive_segments,
        final_packet_text.as_deref(),
        &display_text,
        remove_trailing_period,
    )
}

fn doubao_packet_has_effective_feedback(
    packet_text: &str,
    final_packet_candidate: &str,
    definite_feedback_seen: bool,
) -> bool {
    !packet_text.trim().is_empty()
        || !final_packet_candidate.trim().is_empty()
        || definite_feedback_seen
}

#[cfg(test)]
mod tests {
    use super::doubao_packet_has_effective_feedback;

    #[test]
    fn effective_feedback_requires_text_or_definite_update() {
        assert!(!doubao_packet_has_effective_feedback("", "", false));
        assert!(!doubao_packet_has_effective_feedback("   ", " ", false));
        assert!(doubao_packet_has_effective_feedback("实时字幕", "", false));
        assert!(doubao_packet_has_effective_feedback("", "最终文本", false));
        assert!(doubao_packet_has_effective_feedback("", "", true));
    }
}
