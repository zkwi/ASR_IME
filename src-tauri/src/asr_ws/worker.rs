use super::errors::classify_asr_error;
use super::output::{
    emit_error, emit_successful_final_text, finish_output_sent_session, handle_empty_transcript,
    record_successful_transcript_side_effects, should_hold_overlay_for_output_warning,
    ATTENTION_OVERLAY_HOLD,
};
use crate::session::{SessionController, SessionPhase};
use crate::{
    app_log, asr, asr_activity::AsrActivityReporter, asr_provider, config::AppConfig,
    llm_post_edit, overlay, screen_context, text_output,
};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;

const POST_EDITING_OVERLAY_DELAY: Duration = Duration::from_millis(450);

pub(crate) struct AsrWorkerInput {
    pub(crate) config: AppConfig,
    pub(crate) audio_rx: Receiver<Vec<u8>>,
    pub(crate) started_at: Instant,
    pub(crate) app: AppHandle,
    pub(crate) session: SessionController,
    pub(crate) generation: u64,
    pub(crate) screen_context_rx: Option<screen_context::ScreenContextReceiver>,
    pub(crate) activity: AsrActivityReporter,
}

pub fn spawn_asr_worker(input: AsrWorkerInput) {
    thread::spawn(move || {
        let AsrWorkerInput {
            config,
            audio_rx,
            started_at,
            app,
            session,
            generation,
            screen_context_rx,
            activity,
        } = input;
        app_log::info(format!("ASR worker 已启动: generation={}", generation));
        if let Some(config_error) = asr_provider::worker_configuration_error(&config) {
            let error = config_error.message;
            if session.abort_generation_from_worker_with_code(
                &app,
                generation,
                &error,
                config_error.code,
            ) {
                emit_error(&app, &session, generation, config_error.code, error);
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
            let text = asr_provider::recognize_stream(asr_provider::RecognitionInput {
                config: config.clone(),
                audio_rx,
                app: app.clone(),
                session: session.clone(),
                generation,
                screen_context: screen_context_text.clone(),
                activity,
            })
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
                                if let Some(guard_generation) = session
                                    .reset_generation_from_worker_with_code(
                                        &app, generation, &err, error_code,
                                    )
                                {
                                    emit_error(&app, &session, guard_generation, error_code, err);
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
                if let Some(guard_generation) = session
                    .reset_generation_from_worker_with_code(&app, generation, &err, error_code)
                {
                    emit_error(&app, &session, guard_generation, error_code, err);
                }
            }
        }
    });
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
