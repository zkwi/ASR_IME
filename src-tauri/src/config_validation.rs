use crate::config::{
    AppConfig, ConfigValidationError, ASR_PROVIDER_ALIYUN_FUN, ASR_PROVIDER_DOUBAO, MIN_UI_HEIGHT,
};
use crate::llm_request_adapter::is_valid_thinking_strategy;

/// 校验用户配置文件中会影响主链路的字段。
///
/// 保存配置前必须先调用该函数。这里拦截非法枚举、URL、音频参数和 LLM 必填项，
/// 避免错误值写入 `config.toml` 后在录音、粘贴或连接测试时才失败。
pub fn validate_config(config: &AppConfig) -> Result<(), Vec<ConfigValidationError>> {
    let mut errors = Vec::new();

    validate_hotkey(&mut errors, &config.hotkey);
    validate_allowed_value(
        &mut errors,
        "asr.provider",
        &config.asr.provider,
        &[ASR_PROVIDER_DOUBAO, ASR_PROVIDER_ALIYUN_FUN],
        "语音识别服务只能是 doubao 或 aliyun_fun。",
    );
    validate_u64_range(
        &mut errors,
        "asr.no_feedback_auto_stop_seconds",
        config.asr.no_feedback_auto_stop_seconds,
        0,
        300,
        "ASR 无反馈自动停止秒数需在 0 到 300 之间。",
    );
    validate_u32_range(
        &mut errors,
        "audio.sample_rate",
        config.audio.sample_rate,
        8_000,
        96_000,
        "采样率需在 8000 到 96000 之间。",
    );
    validate_u16_range(
        &mut errors,
        "audio.channels",
        config.audio.channels,
        1,
        2,
        "声道数只能填写 1 或 2。",
    );
    validate_u64_range(
        &mut errors,
        "audio.segment_ms",
        config.audio.segment_ms,
        20,
        2_000,
        "分片毫秒需在 20 到 2000 之间。",
    );
    validate_u64_range(
        &mut errors,
        "audio.max_record_seconds",
        config.audio.max_record_seconds,
        1,
        3_600,
        "最长录音秒数需在 1 到 3600 之间。",
    );
    validate_u64_range(
        &mut errors,
        "audio.stop_grace_ms",
        config.audio.stop_grace_ms,
        0,
        10_000,
        "停止收尾毫秒需在 0 到 10000 之间。",
    );
    validate_f32_range(
        &mut errors,
        "audio.input_gain_db",
        config.audio.input_gain_db,
        -12.0,
        24.0,
        "麦克风输入增益需在 -12 到 24 dB 之间。",
    );
    validate_u64_range(
        &mut errors,
        "typing.paste_delay_ms",
        config.typing.paste_delay_ms,
        0,
        5_000,
        "粘贴延迟需在 0 到 5000 毫秒之间。",
    );
    validate_u64_range(
        &mut errors,
        "typing.clipboard_restore_delay_ms",
        config.typing.clipboard_restore_delay_ms,
        0,
        10_000,
        "剪贴板恢复延迟需在 0 到 10000 毫秒之间。",
    );
    validate_u64_range(
        &mut errors,
        "typing.clipboard_snapshot_max_bytes",
        config.typing.clipboard_snapshot_max_bytes,
        0,
        256 * 1024 * 1024,
        "剪贴板快照大小上限需在 0 到 268435456 字节之间。",
    );
    validate_allowed_value(
        &mut errors,
        "typing.paste_method",
        &config.typing.paste_method,
        &["ctrl_v", "shift_insert", "clipboard_only"],
        "粘贴方式只能是 ctrl_v、shift_insert 或 clipboard_only。",
    );
    validate_allowed_value(
        &mut errors,
        "tray.close_behavior",
        &config.tray.close_behavior,
        &["close_to_tray", "direct_exit", "ask_every_time"],
        "关闭行为只能是 close_to_tray、direct_exit 或 ask_every_time。",
    );
    validate_url_scheme(
        &mut errors,
        "request.ws_url",
        &config.request.ws_url,
        &["ws://", "wss://"],
        "ASR WebSocket 地址必须以 ws:// 或 wss:// 开头。",
    );
    if !config.aliyun_asr.websocket_url.trim().is_empty() {
        validate_url_scheme(
            &mut errors,
            "aliyun_asr.websocket_url",
            &config.aliyun_asr.websocket_url,
            &["wss://"],
            "阿里云 ASR WebSocket 地址必须以 wss:// 开头。",
        );
    }
    validate_allowed_value(
        &mut errors,
        "aliyun_asr.region",
        &config.aliyun_asr.region,
        &["cn-beijing", "ap-southeast-1"],
        "阿里云 ASR 地域只能是 cn-beijing 或 ap-southeast-1。",
    );
    validate_u64_range(
        &mut errors,
        "aliyun_asr.max_sentence_silence",
        config.aliyun_asr.max_sentence_silence,
        200,
        6_000,
        "阿里云 ASR 断句静音需在 200 到 6000 毫秒之间。",
    );
    validate_allowed_value(
        &mut errors,
        "request.language",
        &config.request.language,
        &[
            "", "zh-CN", "en-US", "ja-JP", "id-ID", "es-MX", "pt-BR", "de-DE", "fr-FR", "ko-KR",
            "fil-PH", "ms-MY", "th-TH", "ar-SA", "it-IT", "bn-BD", "el-GR", "nl-NL", "ru-RU",
            "tr-TR", "vi-VN", "pl-PL", "ro-RO", "ne-NP", "uk-UA", "yue-CN",
        ],
        "识别语言只能选择文档支持的语言代码，或留空使用服务默认。",
    );
    if !config.llm_post_edit.base_url.trim().is_empty() {
        validate_url_scheme(
            &mut errors,
            "llm_post_edit.base_url",
            &config.llm_post_edit.base_url,
            &["http://", "https://"],
            "大模型 Base URL 必须以 http:// 或 https:// 开头。",
        );
    }
    validate_github_repo(
        &mut errors,
        "update.github_repo",
        &config.update.github_repo,
    );
    validate_usize_range(
        &mut errors,
        "auto_hotwords.max_history_chars",
        config.auto_hotwords.max_history_chars,
        1_000,
        20_000,
        "自动热词历史文本上限需在 1000 到 20000 字之间。",
    );
    validate_usize_range(
        &mut errors,
        "auto_hotwords.max_candidates",
        config.auto_hotwords.max_candidates,
        5,
        100,
        "自动热词候选数量需在 5 到 100 之间。",
    );
    validate_usize_range(
        &mut errors,
        "screen_context.max_chars",
        config.screen_context.max_chars,
        100,
        4_000,
        "屏幕 OCR 上下文字数上限需在 100 到 4000 之间。",
    );
    validate_u64_range(
        &mut errors,
        "screen_context.timeout_ms",
        config.screen_context.timeout_ms,
        100,
        3_000,
        "屏幕 OCR 等待时间需在 100 到 3000 毫秒之间。",
    );
    if !matches!(
        config.screen_context.capture_scope.as_str(),
        "screen" | "window"
    ) {
        push_validation_error(
            &mut errors,
            "screen_context.capture_scope",
            "屏幕 OCR 识别范围只能是 screen 或 window。",
        );
    }
    validate_f64_range(
        &mut errors,
        "request.final_result_timeout_seconds",
        config.request.final_result_timeout_seconds,
        1.0,
        120.0,
        "最终结果等待时间需在 1 到 120 秒之间。",
    );
    if let Some(value) = config.request.accelerate_score {
        validate_i64_range(
            &mut errors,
            "request.accelerate_score",
            value,
            0,
            20,
            "首字返回加速率需在 0 到 20 之间。",
        );
    }
    validate_f64_range(
        &mut errors,
        "ui.opacity",
        config.ui.opacity,
        0.05,
        1.0,
        "悬浮窗透明度需大于 0 且不超过 1。",
    );
    validate_u32_range(
        &mut errors,
        "ui.width",
        config.ui.width,
        160,
        1_200,
        "悬浮窗宽度需在 160 到 1200 之间。",
    );
    validate_u32_range(
        &mut errors,
        "ui.height",
        config.ui.height,
        MIN_UI_HEIGHT,
        400,
        "悬浮窗高度需在 52 到 400 之间。",
    );
    validate_hex_color(
        &mut errors,
        "ui.background_color",
        &config.ui.background_color,
        "悬浮字幕背景色需使用 #RRGGBB 格式。",
    );
    validate_hex_color(
        &mut errors,
        "ui.text_color",
        &config.ui.text_color,
        "悬浮字幕文字色需使用 #RRGGBB 格式。",
    );
    validate_f64_range(
        &mut errors,
        "llm_post_edit.timeout_seconds",
        config.llm_post_edit.timeout_seconds,
        1.0,
        300.0,
        "大模型超时时间需在 1 到 300 秒之间。",
    );
    validate_usize_range(
        &mut errors,
        "llm_post_edit.min_chars",
        config.llm_post_edit.min_chars,
        0,
        10_000,
        "大模型最少润色字数需在 0 到 10000 之间。",
    );
    validate_usize_range(
        &mut errors,
        "llm_post_edit.screen_context_max_chars",
        config.llm_post_edit.screen_context_max_chars,
        0,
        2_000,
        "大模型屏幕 OCR 参考字数上限需在 0 到 2000 之间。",
    );
    validate_usize_range(
        &mut errors,
        "llm_post_edit.screen_context_max_lines",
        config.llm_post_edit.screen_context_max_lines,
        0,
        100,
        "大模型屏幕 OCR 参考行数上限需在 0 到 100 之间。",
    );
    validate_usize_range(
        &mut errors,
        "llm_post_edit.recent_context_max_chars",
        config.llm_post_edit.recent_context_max_chars,
        0,
        2_000,
        "大模型最近上下文字数上限需在 0 到 2000 之间。",
    );
    validate_usize_range(
        &mut errors,
        "llm_post_edit.reference_hotwords_limit",
        config.llm_post_edit.reference_hotwords_limit,
        0,
        500,
        "大模型热词参考数量上限需在 0 到 500 之间。",
    );
    if !is_valid_thinking_strategy(&config.llm_post_edit.thinking_strategy) {
        push_validation_error(
            &mut errors,
            "llm_post_edit.thinking_strategy",
            "思考模式适配策略只能选择 auto、dashscope_enable_thinking、thinking_disabled、openrouter_reasoning_low、openrouter_reasoning_minimal 或 omit。",
        );
    }
    if config.llm_post_edit.enabled {
        if config.llm_post_edit.api_key.trim().is_empty() {
            push_validation_error(
                &mut errors,
                "llm_post_edit.api_key",
                "启用大模型润色时必须填写 API Key。",
            );
        }
        if config.llm_post_edit.base_url.trim().is_empty() {
            push_validation_error(
                &mut errors,
                "llm_post_edit.base_url",
                "启用大模型润色时必须填写 Base URL。",
            );
        }
        if config.llm_post_edit.model.trim().is_empty() {
            push_validation_error(
                &mut errors,
                "llm_post_edit.model",
                "启用大模型润色时必须填写模型名。",
            );
        }
        if !config.llm_post_edit.user_prompt_template.contains("{text}") {
            push_validation_error(
                &mut errors,
                "llm_post_edit.user_prompt_template",
                "User Prompt 模板必须包含 {text}。",
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_hotkey(errors: &mut Vec<ConfigValidationError>, value: &str) {
    let parts = value
        .split('+')
        .map(|part| part.trim().to_ascii_lowercase())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        push_validation_error(errors, "hotkey", "热键不能为空。");
        return;
    }
    let modifiers = &parts[..parts.len() - 1];
    if !modifiers.iter().any(|part| {
        matches!(
            part.as_str(),
            "ctrl" | "control" | "alt" | "shift" | "win" | "meta"
        )
    }) {
        push_validation_error(errors, "hotkey", "热键至少需要一个修饰键。");
        return;
    }
    if modifiers.iter().any(|part| {
        !matches!(
            part.as_str(),
            "ctrl" | "control" | "alt" | "shift" | "win" | "meta"
        )
    }) {
        push_validation_error(errors, "hotkey", "热键包含不支持的修饰键。");
        return;
    }
    let key = parts.last().map(String::as_str).unwrap_or_default();
    let supported_key = matches!(key, "space" | "enter" | "tab")
        || (key.len() == 1 && key.chars().all(|ch| ch.is_ascii_alphanumeric()))
        || matches!(
            key,
            "f1" | "f2" | "f3" | "f4" | "f5" | "f6" | "f7" | "f8" | "f9" | "f10" | "f11" | "f12"
        );
    if !supported_key {
        push_validation_error(errors, "hotkey", "热键按键暂不支持。");
    }
}

fn validate_u32_range(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: u32,
    min: u32,
    max: u32,
    message: &str,
) {
    if value < min || value > max {
        push_validation_error(errors, field, message);
    }
}

fn validate_u16_range(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: u16,
    min: u16,
    max: u16,
    message: &str,
) {
    if value < min || value > max {
        push_validation_error(errors, field, message);
    }
}

fn validate_u64_range(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: u64,
    min: u64,
    max: u64,
    message: &str,
) {
    if value < min || value > max {
        push_validation_error(errors, field, message);
    }
}

fn validate_usize_range(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: usize,
    min: usize,
    max: usize,
    message: &str,
) {
    if value < min || value > max {
        push_validation_error(errors, field, message);
    }
}

fn validate_i64_range(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: i64,
    min: i64,
    max: i64,
    message: &str,
) {
    if value < min || value > max {
        push_validation_error(errors, field, message);
    }
}

fn validate_f64_range(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: f64,
    min: f64,
    max: f64,
    message: &str,
) {
    if !value.is_finite() || value < min || value > max {
        push_validation_error(errors, field, message);
    }
}

fn validate_f32_range(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: f32,
    min: f32,
    max: f32,
    message: &str,
) {
    if !value.is_finite() || value < min || value > max {
        push_validation_error(errors, field, message);
    }
}

fn validate_hex_color(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: &str,
    message: &str,
) {
    let value = value.trim();
    let valid = value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|ch| ch.is_ascii_hexdigit());
    if !valid {
        push_validation_error(errors, field, message);
    }
}

fn validate_allowed_value(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: &str,
    allowed: &[&str],
    message: &str,
) {
    if !allowed.contains(&value.trim()) {
        push_validation_error(errors, field, message);
    }
}

fn validate_url_scheme(
    errors: &mut Vec<ConfigValidationError>,
    field: &str,
    value: &str,
    allowed_prefixes: &[&str],
    message: &str,
) {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty()
        || !allowed_prefixes
            .iter()
            .any(|prefix| value.starts_with(prefix))
    {
        push_validation_error(errors, field, message);
    }
}

fn validate_github_repo(errors: &mut Vec<ConfigValidationError>, field: &str, value: &str) {
    let parts = value.trim().split('/').collect::<Vec<_>>();
    let valid = parts.len() == 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(is_github_repo_char));
    if !valid {
        push_validation_error(errors, field, "GitHub 仓库必须使用 owner/repo 格式。");
    }
}

fn is_github_repo_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.'
}

fn push_validation_error(errors: &mut Vec<ConfigValidationError>, field: &str, message: &str) {
    errors.push(ConfigValidationError {
        field: field.to_string(),
        message: message.to_string(),
    });
}

pub(crate) fn format_validation_errors(errors: Vec<ConfigValidationError>) -> String {
    let summary = errors
        .iter()
        .map(|error| format!("{}: {}", error.field, error.message))
        .collect::<Vec<_>>()
        .join("; ");
    format!("配置存在不合法字段，请修改后再保存。{}", summary)
}
