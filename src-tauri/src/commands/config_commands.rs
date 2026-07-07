use super::{AppSnapshot, ConfigSaveError, ConnectionTestResult, SetupStatus, SetupWarning};
use crate::{
    app_log, asr_provider, audio, autostart, config, hotkey, llm_post_edit, overlay,
    screen_context, setup_guide, tray,
};
use config::{AppConfig, LoadedConfig};
use tauri::AppHandle;

#[tauri::command]
pub(crate) fn get_app_snapshot() -> Result<AppSnapshot, String> {
    let loaded = config::load_config()?;

    Ok(AppSnapshot {
        hotkey: loaded.data.hotkey,
        current_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub(crate) fn get_setup_status() -> Result<SetupStatus, String> {
    let loaded = config::load_config()?;
    let has_audio_device = audio::list_input_devices()
        .map(|devices| !devices.is_empty())
        .unwrap_or(false);

    Ok(build_setup_status(loaded.data, has_audio_device))
}

fn build_setup_status(data: AppConfig, has_audio_device: bool) -> SetupStatus {
    let asr_configuration_error = asr_provider::configuration_error(&data);
    let missing_auth = asr_configuration_error.is_some();
    let mut warnings = Vec::new();

    if missing_auth {
        warnings.push(SetupWarning {
            code: "ASR_AUTH_MISSING".to_string(),
            level: "blocking".to_string(),
            title: "ASR 密钥未填写".to_string(),
            message: asr_configuration_error
                .map(|error| error.message)
                .unwrap_or_else(|| "填写当前语音识别服务认证信息后才能开始语音识别。".to_string()),
            action: "asr_auth".to_string(),
        });
    }
    if !has_audio_device {
        warnings.push(SetupWarning {
            code: "MIC_DEVICE_NOT_FOUND".to_string(),
            level: "blocking".to_string(),
            title: "未检测到麦克风".to_string(),
            message: "请接入或启用麦克风，然后重新检查设备。".to_string(),
            action: "audio".to_string(),
        });
    }
    let any_trigger_enabled = data.triggers.hotkey_enabled
        || data.triggers.middle_mouse_enabled
        || data.triggers.right_alt_enabled;
    if !any_trigger_enabled {
        warnings.push(SetupWarning {
            code: "TRIGGER_DISABLED".to_string(),
            level: "blocking".to_string(),
            title: "触发方式未开启".to_string(),
            message: "请至少开启主快捷键、右 Alt 或鼠标中键中的一种。".to_string(),
            action: "hotkey".to_string(),
        });
    }
    SetupStatus {
        ready: !missing_auth && has_audio_device && any_trigger_enabled,
        missing_auth,
        has_audio_device,
        hotkey: data.hotkey,
        paste_method: data.typing.paste_method,
        privacy_recent_context_enabled: data.context.enable_recent_context,
        warnings,
    }
}

#[tauri::command]
pub(crate) fn load_app_config() -> Result<LoadedConfig, String> {
    match config::load_config() {
        Ok(loaded) => {
            app_log::info(format!("配置加载完成: exists={}", loaded.exists));
            Ok(loaded)
        }
        Err(err) => {
            app_log::warn(format!("配置加载失败: {}", err));
            Err(err)
        }
    }
}

#[tauri::command]
pub(crate) fn get_config_migration_candidate(
) -> Result<Option<config::ConfigMigrationCandidate>, String> {
    Ok(config::config_migration_candidate())
}

#[tauri::command]
pub(crate) fn migrate_config_to_default_path() -> Result<LoadedConfig, String> {
    let loaded = config::migrate_legacy_config_to_default_path()?;
    app_log::info(format!("配置迁移检查完成: exists={}", loaded.exists));
    Ok(loaded)
}

#[tauri::command]
pub(crate) fn save_app_config(
    app: AppHandle,
    config: AppConfig,
) -> Result<LoadedConfig, ConfigSaveError> {
    let previous_config = crate::config::load_config().ok().map(|loaded| loaded.data);
    let save_mode = match config_save_mode(previous_config.as_ref(), &config) {
        Ok(mode) => mode,
        Err(blocking_errors) => {
            app_log::warn(format!(
                "配置保存失败: validation_errors={}",
                blocking_errors.len()
            ));
            return Err(ConfigSaveError {
                message: "配置存在不合法字段，请修改后再保存。".to_string(),
                errors: blocking_errors,
            });
        }
    };
    if save_mode == ConfigSaveMode::Unchecked {
        app_log::warn("配置存在未改动的隐藏高级字段错误，已保留原值并继续保存。");
    }
    let side_effects = config_side_effects(previous_config.as_ref(), &config);
    if hotkey_registration_test_needed(previous_config.as_ref(), &config) {
        if let Err(err) = hotkey::can_register_hotkey(&config.hotkey) {
            app_log::warn(format!(
                "配置保存失败: hotkey register test failed: {}",
                err
            ));
            return Err(ConfigSaveError {
                message: format!("快捷键注册测试失败：{}", err),
                errors: vec![crate::config::ConfigValidationError {
                    field: "hotkey".to_string(),
                    message: "该快捷键可能已被其他程序占用，请换一个。".to_string(),
                }],
            });
        }
    }
    let save_result = match save_mode {
        ConfigSaveMode::Strict => crate::config::save_config(config),
        ConfigSaveMode::Unchecked => crate::config::save_config_without_validation(config),
    };
    match save_result {
        Ok(loaded) => {
            apply_config_side_effects(app.clone(), &loaded, side_effects);
            app_log::info(format!(
                "配置保存完成: hotkey_enabled={}, middle_mouse_enabled={}, right_alt_enabled={}, hotkey_restart_scheduled={}, launch_on_startup={}, autostart_sync_scheduled={}, update_auto_check={}, update_repo={}, llm_enabled={}, close_behavior={}",
                loaded.data.triggers.hotkey_enabled,
                loaded.data.triggers.middle_mouse_enabled,
                loaded.data.triggers.right_alt_enabled,
                side_effects.restart_hotkey,
                loaded.data.startup.launch_on_startup,
                side_effects.apply_autostart,
                loaded.data.update.auto_check_on_startup,
                loaded.data.update.github_repo,
                loaded.data.llm_post_edit.enabled,
                loaded.data.tray.close_behavior
            ));
            Ok(loaded)
        }
        Err(err) => {
            app_log::warn(format!("配置保存失败: {}", err));
            Err(ConfigSaveError {
                message: err,
                errors: Vec::new(),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigSaveMode {
    Strict,
    Unchecked,
}

fn config_save_mode(
    previous_config: Option<&AppConfig>,
    next_config: &AppConfig,
) -> Result<ConfigSaveMode, Vec<crate::config::ConfigValidationError>> {
    match crate::config::validate_config(next_config) {
        Ok(()) => Ok(ConfigSaveMode::Strict),
        Err(errors) => {
            let blocking_errors = blocking_validation_errors(errors, previous_config, next_config);
            if blocking_errors.is_empty() {
                Ok(ConfigSaveMode::Unchecked)
            } else {
                Err(blocking_errors)
            }
        }
    }
}

fn blocking_validation_errors(
    errors: Vec<crate::config::ConfigValidationError>,
    previous_config: Option<&AppConfig>,
    next_config: &AppConfig,
) -> Vec<crate::config::ConfigValidationError> {
    errors
        .into_iter()
        .filter(|error| {
            !unchanged_hidden_config_field(previous_config, next_config, error.field.as_str())
        })
        .collect()
}

fn unchanged_hidden_config_field(
    previous_config: Option<&AppConfig>,
    next_config: &AppConfig,
    field: &str,
) -> bool {
    let Some(previous_config) = previous_config else {
        return false;
    };
    match field {
        "asr.no_feedback_auto_stop_seconds" => {
            previous_config.asr.no_feedback_auto_stop_seconds
                == next_config.asr.no_feedback_auto_stop_seconds
        }
        "audio.sample_rate" => previous_config.audio.sample_rate == next_config.audio.sample_rate,
        "audio.channels" => previous_config.audio.channels == next_config.audio.channels,
        "audio.segment_ms" => previous_config.audio.segment_ms == next_config.audio.segment_ms,
        "audio.max_record_seconds" => {
            previous_config.audio.max_record_seconds == next_config.audio.max_record_seconds
        }
        "audio.stop_grace_ms" => {
            previous_config.audio.stop_grace_ms == next_config.audio.stop_grace_ms
        }
        "request.final_result_timeout_seconds" => {
            previous_config.request.final_result_timeout_seconds
                == next_config.request.final_result_timeout_seconds
        }
        "llm_post_edit.timeout_seconds" => {
            previous_config.llm_post_edit.timeout_seconds
                == next_config.llm_post_edit.timeout_seconds
        }
        "llm_post_edit.screen_context_max_chars" => {
            previous_config.llm_post_edit.screen_context_max_chars
                == next_config.llm_post_edit.screen_context_max_chars
        }
        "llm_post_edit.screen_context_max_lines" => {
            previous_config.llm_post_edit.screen_context_max_lines
                == next_config.llm_post_edit.screen_context_max_lines
        }
        "llm_post_edit.recent_context_max_chars" => {
            previous_config.llm_post_edit.recent_context_max_chars
                == next_config.llm_post_edit.recent_context_max_chars
        }
        "llm_post_edit.reference_hotwords_limit" => {
            previous_config.llm_post_edit.reference_hotwords_limit
                == next_config.llm_post_edit.reference_hotwords_limit
        }
        "ui.width" => previous_config.ui.width == next_config.ui.width,
        "ui.height" => previous_config.ui.height == next_config.ui.height,
        "ui.background_color" => {
            previous_config.ui.background_color == next_config.ui.background_color
        }
        "ui.text_color" => previous_config.ui.text_color == next_config.ui.text_color,
        "typing.paste_delay_ms" => {
            previous_config.typing.paste_delay_ms == next_config.typing.paste_delay_ms
        }
        "typing.clipboard_snapshot_max_bytes" => {
            previous_config.typing.clipboard_snapshot_max_bytes
                == next_config.typing.clipboard_snapshot_max_bytes
        }
        "request.ws_url" => previous_config.request.ws_url == next_config.request.ws_url,
        "update.github_repo" => {
            previous_config.update.github_repo == next_config.update.github_repo
        }
        "auto_hotwords.max_history_chars" => {
            previous_config.auto_hotwords.max_history_chars
                == next_config.auto_hotwords.max_history_chars
        }
        "auto_hotwords.max_candidates" => {
            previous_config.auto_hotwords.max_candidates == next_config.auto_hotwords.max_candidates
        }
        "screen_context.max_chars" => {
            previous_config.screen_context.max_chars == next_config.screen_context.max_chars
        }
        "screen_context.timeout_ms" => {
            previous_config.screen_context.timeout_ms == next_config.screen_context.timeout_ms
        }
        "screen_context.capture_scope" => {
            previous_config.screen_context.capture_scope == next_config.screen_context.capture_scope
        }
        _ => false,
    }
}

#[tauri::command]
pub(crate) async fn test_asr_config(config: AppConfig) -> Result<ConnectionTestResult, String> {
    let provider_label = asr_provider::active_provider_label(&config);
    app_log::info(format!("用户开始测试{}配置。", provider_label));
    match asr_provider::test_connection(&config).await {
        Ok(()) => {
            app_log::info(format!("{}配置测试成功。", provider_label));
            Ok(ConnectionTestResult::message(format!(
                "{}测试成功，当前 Key 可用。",
                provider_label
            )))
        }
        Err(err) => {
            app_log::warn(format!("{}配置测试失败: {}", provider_label, err));
            Err(err)
        }
    }
}

#[tauri::command]
pub(crate) async fn test_llm_config(config: AppConfig) -> Result<ConnectionTestResult, String> {
    app_log::info("用户开始测试大模型配置。");
    match llm_post_edit::test_connection(&config).await {
        Ok(result) => {
            app_log::info(format!(
                "大模型配置测试成功: elapsed_ms={}, thinking_strategy={}",
                result.elapsed_ms, result.thinking_strategy
            ));
            Ok(ConnectionTestResult::with_llm_result(
                "大模型测试成功，当前 API Key 可用。",
                result.elapsed_ms,
                result.thinking_strategy,
            ))
        }
        Err(err) => {
            app_log::warn(format!("大模型配置测试失败: {}", err));
            Err(err)
        }
    }
}

#[tauri::command]
pub(crate) fn test_screen_context(
    config: AppConfig,
) -> Result<screen_context::ScreenContextTestResult, String> {
    app_log::info("用户开始测试屏幕 OCR 上下文。");
    match screen_context::test_capture_on_worker(&config.screen_context) {
        Ok(result) => {
            app_log::info(format!(
                "屏幕 OCR 上下文测试完成: chars={}, elapsed_ms={}, language={}, image={}x{}",
                result.text_chars,
                result.elapsed_ms,
                result.selected_language.as_deref().unwrap_or("unknown"),
                result.image_width,
                result.image_height
            ));
            Ok(result)
        }
        Err(err) => {
            app_log::warn(format!("屏幕 OCR 上下文测试失败: {}", err));
            Err(err)
        }
    }
}

#[tauri::command]
pub(crate) fn open_setup_guide(app: AppHandle) -> Result<(), String> {
    app_log::info("用户打开配置指南。");
    setup_guide::open(&app).map_err(|err| {
        app_log::warn(format!("打开配置指南失败: {}", err));
        err
    })
}

#[tauri::command]
pub(crate) fn open_doubao_asr_docs(app: AppHandle) -> Result<(), String> {
    app_log::info("用户打开豆包 ASR 帮助文档。");
    setup_guide::open_doubao_asr_docs(&app).map_err(|err| {
        app_log::warn(format!("打开豆包 ASR 帮助文档失败: {}", err));
        err
    })
}

#[tauri::command]
pub(crate) fn open_aliyun_asr_docs(app: AppHandle) -> Result<(), String> {
    app_log::info("用户打开阿里云 ASR 帮助文档。");
    setup_guide::open_aliyun_asr_docs(&app).map_err(|err| {
        app_log::warn(format!("打开阿里云 ASR 帮助文档失败: {}", err));
        err
    })
}

#[tauri::command]
pub(crate) fn set_tray_language(app: AppHandle, language: String) -> Result<(), String> {
    tray::set_language(&app, &language)
}

#[tauri::command]
pub(crate) fn update_close_preference(
    close_behavior: Option<String>,
    close_to_tray_notice_shown: Option<bool>,
) -> Result<LoadedConfig, String> {
    let mut loaded = config::load_config()?;
    if let Some(behavior) = close_behavior {
        loaded.data.tray.close_behavior = normalize_close_behavior(&behavior).to_string();
    }
    if let Some(shown) = close_to_tray_notice_shown {
        loaded.data.tray.close_to_tray_notice_shown = shown;
    }
    let saved = config::save_config(loaded.data)?;
    app_log::info(format!(
        "关闭行为配置已更新: close_behavior={}, notice_shown={}",
        saved.data.tray.close_behavior, saved.data.tray.close_to_tray_notice_shown
    ));
    Ok(saved)
}

fn normalize_close_behavior(value: &str) -> &str {
    match value {
        "direct_exit" => "direct_exit",
        "ask_every_time" => "ask_every_time",
        _ => "close_to_tray",
    }
}

fn hotkey_equal(left: &str, right: &str) -> bool {
    let normalize = |value: &str| {
        value
            .split('+')
            .map(|part| part.trim().to_ascii_lowercase())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("+")
    };
    normalize(left) == normalize(right)
}

fn hotkey_registration_test_needed(previous: Option<&AppConfig>, next: &AppConfig) -> bool {
    if !next.triggers.hotkey_enabled {
        return false;
    }
    previous
        .map(|previous| {
            !previous.triggers.hotkey_enabled || !hotkey_equal(&previous.hotkey, &next.hotkey)
        })
        .unwrap_or(true)
}

fn hotkey_runtime_update_needed(previous: Option<&AppConfig>, next: &AppConfig) -> bool {
    previous
        .map(|previous| {
            previous.triggers.hotkey_enabled != next.triggers.hotkey_enabled
                || !hotkey_equal(&previous.hotkey, &next.hotkey)
        })
        .unwrap_or(true)
}

fn autostart_update_needed(previous: Option<&AppConfig>, next: &AppConfig) -> bool {
    previous
        .map(|previous| previous.startup.launch_on_startup != next.startup.launch_on_startup)
        .unwrap_or(true)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ConfigSideEffects {
    restart_hotkey: bool,
    apply_autostart: bool,
}

fn config_side_effects(previous: Option<&AppConfig>, next: &AppConfig) -> ConfigSideEffects {
    ConfigSideEffects {
        restart_hotkey: hotkey_runtime_update_needed(previous, next),
        apply_autostart: autostart_update_needed(previous, next),
    }
}

fn apply_config_side_effects(app: AppHandle, loaded: &LoadedConfig, effects: ConfigSideEffects) {
    hotkey::refresh_trigger_config_from(&loaded.data.triggers);
    overlay::update_config(&app, &loaded.data.ui);
    if effects.restart_hotkey {
        restart_hotkey_in_background(app.clone());
    }
    if effects.apply_autostart {
        apply_autostart_in_background(loaded.data.startup.clone());
    }
}

fn restart_hotkey_in_background(app: AppHandle) {
    std::thread::spawn(move || {
        if let Err(err) = hotkey::restart_global_hotkey_thread(app) {
            app_log::warn(format!("配置已保存，但快捷键重新注册未确认完成: {}", err));
        }
    });
}

fn apply_autostart_in_background(startup: config::StartupConfig) {
    std::thread::spawn(move || {
        if let Err(err) = autostart::apply(&startup) {
            app_log::warn(format!("配置已保存，但开机自启动后台同步失败: {}", err));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{
        autostart_update_needed, blocking_validation_errors, build_setup_status, config_save_mode,
        config_side_effects, hotkey_registration_test_needed, hotkey_runtime_update_needed,
        ConfigSaveMode, ConfigSideEffects,
    };
    use crate::config::{AppConfig, ConfigValidationError};

    #[test]
    fn setup_status_blocks_missing_auth_audio_and_triggers() {
        let mut config = AppConfig::default();
        config.auth.app_key.clear();
        config.auth.access_key.clear();
        config.triggers.hotkey_enabled = false;
        let status = build_setup_status(config, false);

        assert!(!status.ready);
        assert!(status.missing_auth);
        assert!(!status.has_audio_device);
        assert_eq!(status.warnings.len(), 3);
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.code == "ASR_AUTH_MISSING" && warning.level == "blocking"));
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.code == "MIC_DEVICE_NOT_FOUND" && warning.level == "blocking"));
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.code == "TRIGGER_DISABLED" && warning.level == "blocking"));
    }

    #[test]
    fn setup_status_keeps_soft_options_non_blocking() {
        let mut config = AppConfig::default();
        config.auth.app_key = "app".to_string();
        config.auth.access_key = "access".to_string();
        config.context.enable_recent_context = false;
        config.triggers.middle_mouse_enabled = false;
        config.triggers.right_alt_enabled = false;

        let status = build_setup_status(config, true);

        assert!(status.ready);
        assert!(status.warnings.is_empty());
    }

    #[test]
    fn setup_status_is_ready_when_auth_audio_and_trigger_are_available() {
        let mut config = AppConfig::default();
        config.auth.app_key = "app".to_string();
        config.auth.access_key = "access".to_string();
        config.triggers.hotkey_enabled = true;

        let status = build_setup_status(config, true);

        assert!(status.ready);
        assert!(!status.missing_auth);
        assert!(status.has_audio_device);
        assert_eq!(status.hotkey, "ctrl+q");
    }

    #[test]
    fn hotkey_registration_test_is_needed_when_enabled_with_same_text() {
        let mut previous = AppConfig::default();
        previous.triggers.hotkey_enabled = false;
        previous.hotkey = "Ctrl+Q".to_string();
        let mut next = previous.clone();
        next.triggers.hotkey_enabled = true;

        assert!(hotkey_registration_test_needed(Some(&previous), &next));
    }

    #[test]
    fn hotkey_registration_test_is_skipped_when_still_disabled() {
        let mut previous = AppConfig::default();
        previous.triggers.hotkey_enabled = false;
        let mut next = previous.clone();
        next.hotkey = "Ctrl+Shift+Q".to_string();

        assert!(!hotkey_registration_test_needed(Some(&previous), &next));
    }

    #[test]
    fn hotkey_runtime_update_is_needed_when_hotkey_changes() {
        let mut previous = AppConfig::default();
        let mut next = previous.clone();
        next.hotkey = "Ctrl+Shift+Q".to_string();

        assert!(hotkey_runtime_update_needed(Some(&previous), &next));
        assert!(config_side_effects(Some(&previous), &next).restart_hotkey);

        previous.hotkey = next.hotkey.clone();
        next.triggers.right_alt_enabled = true;
        assert!(!hotkey_runtime_update_needed(Some(&previous), &next));
    }

    #[test]
    fn hotkey_runtime_update_is_skipped_for_unrelated_settings() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.auth.app_key = "new".to_string();

        assert!(!hotkey_runtime_update_needed(Some(&previous), &next));
        assert!(!config_side_effects(Some(&previous), &next).restart_hotkey);
    }

    #[test]
    fn autostart_update_is_needed_when_startup_changes() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.startup.launch_on_startup = !previous.startup.launch_on_startup;

        assert!(autostart_update_needed(Some(&previous), &next));
        assert!(config_side_effects(Some(&previous), &next).apply_autostart);
    }

    #[test]
    fn autostart_update_is_skipped_for_unrelated_settings() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.auth.access_key = "new".to_string();

        assert!(!autostart_update_needed(Some(&previous), &next));
        assert!(!config_side_effects(Some(&previous), &next).apply_autostart);
    }

    #[test]
    fn config_side_effects_are_empty_for_unrelated_settings() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.llm_post_edit.enabled = !previous.llm_post_edit.enabled;

        let effects = config_side_effects(Some(&previous), &next);

        assert!(!effects.restart_hotkey);
        assert!(!effects.apply_autostart);
    }

    #[test]
    fn config_side_effects_detect_hotkey_and_autostart_independently() {
        let previous = AppConfig::default();
        let mut hotkey_next = previous.clone();
        hotkey_next.hotkey = "Ctrl+Shift+Q".to_string();
        assert_eq!(
            config_side_effects(Some(&previous), &hotkey_next),
            ConfigSideEffects {
                restart_hotkey: true,
                apply_autostart: false,
            }
        );

        let mut autostart_next = previous.clone();
        autostart_next.startup.launch_on_startup = !previous.startup.launch_on_startup;
        assert_eq!(
            config_side_effects(Some(&previous), &autostart_next),
            ConfigSideEffects {
                restart_hotkey: false,
                apply_autostart: true,
            }
        );
    }

    #[test]
    fn unchanged_hidden_config_validation_errors_do_not_block_visible_saves() {
        let mut previous = AppConfig::default();
        previous.audio.sample_rate = 0;

        let mut next = previous.clone();
        next.ui.width += 10;

        let blocking = blocking_validation_errors(
            vec![validation_error("audio.sample_rate")],
            Some(&previous),
            &next,
        );

        assert!(blocking.is_empty());
    }

    #[test]
    fn unchanged_hidden_config_validation_errors_use_unchecked_save_mode() {
        let mut previous = AppConfig::default();
        previous.audio.sample_rate = 0;

        let mut next = previous.clone();
        next.ui.width += 10;

        assert_eq!(
            config_save_mode(Some(&previous), &next).unwrap(),
            ConfigSaveMode::Unchecked
        );
    }

    #[test]
    fn changed_hidden_config_validation_errors_still_block_saves() {
        let previous = AppConfig::default();
        let mut next = previous.clone();
        next.audio.sample_rate = 0;

        let blocking = blocking_validation_errors(
            vec![validation_error("audio.sample_rate")],
            Some(&previous),
            &next,
        );

        assert_eq!(blocking.len(), 1);
        assert_eq!(blocking[0].field, "audio.sample_rate");
    }

    #[test]
    fn visible_config_validation_errors_still_block_even_when_unchanged() {
        let mut previous = AppConfig::default();
        previous.request.language = "auto".to_string();

        let next = previous.clone();

        let blocking = blocking_validation_errors(
            vec![validation_error("request.language")],
            Some(&previous),
            &next,
        );

        assert_eq!(blocking.len(), 1);
        assert_eq!(blocking[0].field, "request.language");
    }

    fn validation_error(field: &str) -> ConfigValidationError {
        ConfigValidationError {
            field: field.to_string(),
            message: "invalid".to_string(),
        }
    }
}
