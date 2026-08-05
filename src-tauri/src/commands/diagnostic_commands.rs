use super::{ConnectionTestResult, DiagnosticReport, LocalDataStatus};
use crate::session::SessionController;
use crate::{
    app_log, asr_provider, config, hotword_generator, hotword_history, stats, text_output, tray,
};
use stats::StatsSnapshot;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub(crate) fn open_log_file(app: AppHandle) -> Result<(), String> {
    match tray::open_log_file_from_main(&app) {
        Ok(()) => Ok(()),
        Err(err) => {
            app_log::warn(err.clone());
            Err(err)
        }
    }
}

#[tauri::command]
pub(crate) fn get_diagnostic_report(
    session: State<'_, SessionController>,
) -> Result<DiagnosticReport, String> {
    let report = build_diagnostic_report(&session)?;
    app_log::info("用户生成诊断报告。");
    Ok(report)
}

#[tauri::command]
pub(crate) fn copy_diagnostic_report_to_clipboard(
    session: State<'_, SessionController>,
) -> Result<DiagnosticReport, String> {
    let report = build_diagnostic_report(&session)?;
    text_output::copy_text_to_clipboard(&report.text)?;
    app_log::info("用户复制诊断报告到剪贴板。");
    Ok(report)
}

#[tauri::command]
pub(crate) fn copy_recent_input_text_to_clipboard(text: String) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("没有可复制的识别文本。".to_string());
    }
    text_output::copy_text_to_clipboard(&text)
}

fn build_diagnostic_report(
    session: &State<'_, SessionController>,
) -> Result<DiagnosticReport, String> {
    let loaded = config::load_config()?;
    let state = session.current_state();
    let asr_ready = asr_provider::configuration_error(&loaded.data).is_none();
    let trigger_summary = enabled_trigger_summary(&loaded.data);
    let recent_error = state.error_code.as_deref().unwrap_or("无");
    let recent_context_summary = if loaded.data.context.enable_recent_context {
        format!("已启用，保存条数 {}", config::recent_context_count())
    } else {
        "未启用".to_string()
    };
    let auto_hotword_summary = hotword_history::status()
        .map(|status| {
            if status.enabled {
                format!(
                    "已启用，保存条数 {}，约 {} 字",
                    status.entry_count, status.total_chars
                )
            } else {
                "未启用".to_string()
            }
        })
        .unwrap_or_else(|_| "状态读取失败".to_string());
    let text = format!(
        "VoxType 诊断报告\n\
版本: {}\n\
系统: {} / {}\n\
配置文件: {} ({})\n\
日志文件: {}\n\
ASR 已配置: {}\n\
LLM 润色: {}\n\
最近上下文: {}\n\
自动热词候选: {}\n\
触发方式: {}\n\
最近会话状态: {:?}\n\
最近错误码: {}\n\
诊断报告内容: 不包含识别正文、屏幕 OCR 正文、热词、Prompt、最近上下文正文、自动热词历史正文、候选词、密钥原文\n\
日志脱敏范围: key/token/bearer/password/secret 类字段和本机用户路径\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        redact_user_path(&loaded.path),
        if loaded.exists { "已存在" } else { "未创建" },
        redact_user_path(&app_log::log_path().display().to_string()),
        if asr_ready { "是" } else { "否" },
        if loaded.data.llm_post_edit.enabled {
            "已启用"
        } else {
            "未启用"
        },
        recent_context_summary,
        auto_hotword_summary,
        trigger_summary,
        state.phase,
        recent_error
    );
    Ok(DiagnosticReport { text })
}

#[tauri::command]
pub(crate) fn hide_main_window(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("找不到主窗口。".to_string());
    };
    window
        .hide()
        .map_err(|err| format!("隐藏主窗口失败: {}", err))?;
    let _ = window.emit("main-window-hidden", ());
    app_log::info("主窗口已隐藏到托盘。");
    Ok(())
}

#[tauri::command]
pub(crate) fn set_config_exit_guard(active: bool) {
    crate::main_window::set_config_exit_guard(active);
}

#[tauri::command]
pub(crate) fn exit_application(app: AppHandle) {
    crate::main_window::set_config_exit_guard(false);
    app_log::info("用户从主窗口退出程序。");
    tray::exit_app(&app);
}

#[tauri::command]
pub(crate) fn log_frontend_error(message: String) {
    app_log::warn(format!("frontend error: {}", message));
}

#[tauri::command]
pub(crate) fn log_frontend_event(message: String) {
    app_log::info(format!("frontend event: {}", message));
}

#[tauri::command]
pub(crate) fn get_usage_stats() -> StatsSnapshot {
    stats::load_stats_snapshot()
}

#[tauri::command]
pub(crate) fn get_local_data_status() -> Result<LocalDataStatus, String> {
    let loaded = config::load_config()?;
    let auto_hotword_status = hotword_history::status()?;
    Ok(LocalDataStatus {
        config_path: loaded.path.clone(),
        log_path: app_log::log_path().display().to_string(),
        recent_context_enabled: loaded.data.context.enable_recent_context,
        recent_context_count: config::recent_context_count(),
        auto_hotwords_enabled: auto_hotword_status.enabled,
        auto_hotword_entry_count: auto_hotword_status.entry_count,
        auto_hotword_total_chars: auto_hotword_status.total_chars,
        stats_event_count: stats::stats_event_count(),
        screen_context_enabled: loaded.data.screen_context.enabled,
        llm_post_edit_enabled: loaded.data.llm_post_edit.enabled,
        restore_clipboard_after_paste: loaded.data.typing.restore_clipboard_after_paste,
    })
}

#[tauri::command]
pub(crate) fn clear_usage_stats() -> Result<ConnectionTestResult, String> {
    stats::clear_stats()?;
    app_log::info("用户清除使用统计数据。");
    Ok(ConnectionTestResult::message("使用统计已清除。"))
}

#[tauri::command]
pub(crate) fn clear_recent_context() -> Result<ConnectionTestResult, String> {
    config::clear_recent_context()?;
    app_log::info(format!(
        "用户清除最近上下文: remaining={}",
        config::recent_context_count()
    ));
    Ok(ConnectionTestResult::message("最近上下文已清除。"))
}

#[tauri::command]
pub(crate) fn get_auto_hotword_status() -> Result<hotword_history::AutoHotwordStatus, String> {
    hotword_history::status()
}

#[tauri::command]
pub(crate) fn clear_hotword_history() -> Result<ConnectionTestResult, String> {
    hotword_history::clear_history()?;
    let status = hotword_history::status().ok();
    app_log::info(format!(
        "用户清除自动热词采集文本: remaining_entries={}",
        status.map(|item| item.entry_count).unwrap_or(0)
    ));
    Ok(ConnectionTestResult::message("自动热词采集文本已清空。"))
}

#[tauri::command]
pub(crate) async fn generate_hotword_candidates(
    config: config::AppConfig,
) -> Result<hotword_generator::HotwordGenerationResult, String> {
    hotword_generator::generate_candidates(config).await
}

fn enabled_trigger_summary(config: &config::AppConfig) -> String {
    let mut triggers = Vec::new();
    if config.triggers.hotkey_enabled {
        triggers.push(config.hotkey.to_uppercase());
    }
    if config.triggers.right_alt_enabled {
        triggers.push("右 Alt".to_string());
    }
    if config.triggers.middle_mouse_enabled {
        triggers.push("鼠标中键".to_string());
    }
    if triggers.is_empty() {
        "未启用".to_string()
    } else {
        triggers.join(" / ")
    }
}

fn redact_user_path(value: &str) -> String {
    let Ok(profile) = std::env::var("USERPROFILE") else {
        return value.to_string();
    };
    redact_path_with_profile(value, &profile)
}

fn redact_path_with_profile(value: &str, profile: &str) -> String {
    if profile.is_empty() {
        return value.to_string();
    }
    let lower_value = value.to_ascii_lowercase();
    let lower_profile = profile.to_ascii_lowercase();
    if lower_value == lower_profile {
        return "%USERPROFILE%".to_string();
    }
    if lower_value.starts_with(&lower_profile) {
        let suffix = &value[profile.len()..];
        if suffix.starts_with('\\') || suffix.starts_with('/') {
            return format!("%USERPROFILE%{}", suffix);
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::{enabled_trigger_summary, redact_path_with_profile};
    use crate::config::AppConfig;

    #[test]
    fn enabled_trigger_summary_lists_active_triggers() {
        let mut config = AppConfig {
            hotkey: "Ctrl+Q".to_string(),
            ..Default::default()
        };
        config.triggers.hotkey_enabled = true;
        config.triggers.right_alt_enabled = true;
        config.triggers.middle_mouse_enabled = false;

        assert_eq!(enabled_trigger_summary(&config), "CTRL+Q / 右 Alt");
    }

    #[test]
    fn enabled_trigger_summary_reports_no_active_trigger() {
        let mut config = AppConfig::default();
        config.triggers.hotkey_enabled = false;
        config.triggers.right_alt_enabled = false;
        config.triggers.middle_mouse_enabled = false;

        assert_eq!(enabled_trigger_summary(&config), "未启用");
    }

    #[test]
    fn redacts_exact_user_profile_path() {
        assert_eq!(
            redact_path_with_profile("C:\\Users\\Alice", "C:\\Users\\Alice"),
            "%USERPROFILE%"
        );
    }

    #[test]
    fn redacts_user_profile_child_path_case_insensitively() {
        assert_eq!(
            redact_path_with_profile(
                "C:\\Users\\Alice\\AppData\\Local\\VoxType\\logs\\voice_input.log",
                "c:\\users\\alice",
            ),
            "%USERPROFILE%\\AppData\\Local\\VoxType\\logs\\voice_input.log"
        );
    }

    #[test]
    fn does_not_redact_similar_user_profile_prefix() {
        assert_eq!(
            redact_path_with_profile("C:\\Users\\AliceBackup\\file.txt", "C:\\Users\\Alice"),
            "C:\\Users\\AliceBackup\\file.txt"
        );
    }
}
