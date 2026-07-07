//! ASR provider facade.
//!
//! This module is intentionally small: it owns provider selection, shared
//! configuration gates, and the call shape used by the recording session.
//! Provider-specific protocol details stay in `asr_ws.rs` and `aliyun_asr.rs`.

use crate::config::{AppConfig, ASR_PROVIDER_ALIYUN_FUN};
use crate::session::SessionController;
use crate::{aliyun_asr, asr_ws};
use std::sync::mpsc::Receiver;
use tauri::AppHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AsrProviderKind {
    Doubao,
    AliyunFun,
}

/// Keeps provider entrypoints aligned without adding a trait/factory layer.
pub(crate) struct RecognitionInput {
    pub(crate) config: AppConfig,
    pub(crate) audio_rx: Receiver<Vec<u8>>,
    pub(crate) app: AppHandle,
    pub(crate) session: SessionController,
    pub(crate) generation: u64,
    pub(crate) screen_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AsrConfigurationError {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

pub(crate) fn active_provider(config: &AppConfig) -> AsrProviderKind {
    if config.asr.provider.trim() == ASR_PROVIDER_ALIYUN_FUN {
        AsrProviderKind::AliyunFun
    } else {
        AsrProviderKind::Doubao
    }
}

pub(crate) fn active_provider_label(config: &AppConfig) -> &'static str {
    match active_provider(config) {
        AsrProviderKind::Doubao => "豆包 ASR",
        AsrProviderKind::AliyunFun => "阿里云 FunASR",
    }
}

pub(crate) fn configuration_error(config: &AppConfig) -> Option<AsrConfigurationError> {
    match active_provider(config) {
        AsrProviderKind::Doubao => doubao_configuration_error(config),
        AsrProviderKind::AliyunFun => aliyun_configuration_error(config),
    }
}

pub(crate) fn start_configuration_error(
    config: &AppConfig,
    config_exists: bool,
) -> Option<AsrConfigurationError> {
    configuration_error(config).map(|_| {
        if config_exists {
            match active_provider(config) {
                AsrProviderKind::Doubao => AsrConfigurationError {
                    code: "ASR_AUTH_MISSING",
                    message: "ASR 未配置 app_key/access_key，请先在配置页填写豆包认证信息并保存。"
                        .to_string(),
                },
                AsrProviderKind::AliyunFun => AsrConfigurationError {
                    code: "ASR_AUTH_MISSING",
                    message:
                        "ASR 未配置阿里云 API Key 或 Workspace ID，请先在配置页填写认证信息并保存。"
                            .to_string(),
                },
            }
        } else {
            AsrConfigurationError {
                code: "CONFIG_MISSING",
                message: "未找到 config.toml。请先在配置页填写语音识别服务认证信息并保存，或复制 config.example.toml 为 config.toml 后手动编辑。"
                    .to_string(),
            }
        }
    })
}

pub(crate) fn worker_configuration_error(config: &AppConfig) -> Option<AsrConfigurationError> {
    configuration_error(config).map(|_| match active_provider(config) {
        AsrProviderKind::Doubao => AsrConfigurationError {
            code: "ASR_AUTH_MISSING",
            message: "ASR skipped: app_key/access_key is not configured.".to_string(),
        },
        AsrProviderKind::AliyunFun => AsrConfigurationError {
            code: "ASR_AUTH_MISSING",
            message: "阿里云 ASR 未配置 API Key、模型或 Workspace ID。".to_string(),
        },
    })
}

pub(crate) async fn test_connection(config: &AppConfig) -> Result<(), String> {
    match active_provider(config) {
        AsrProviderKind::Doubao => asr_ws::test_doubao_connection(config).await,
        AsrProviderKind::AliyunFun => aliyun_asr::test_connection(config).await,
    }
}

pub(crate) async fn recognize_stream(input: RecognitionInput) -> Result<String, String> {
    match active_provider(&input.config) {
        AsrProviderKind::Doubao => {
            let screen_context = input.screen_context;
            asr_ws::run_doubao_websocket_session(
                input.config,
                input.audio_rx,
                input.app,
                input.session,
                input.generation,
                screen_context.as_deref(),
            )
            .await
        }
        AsrProviderKind::AliyunFun => {
            let screen_context = input.screen_context;
            aliyun_asr::recognize_stream(
                input.config,
                input.audio_rx,
                input.app,
                input.session,
                input.generation,
                screen_context.as_deref(),
            )
            .await
        }
    }
}

fn doubao_configuration_error(config: &AppConfig) -> Option<AsrConfigurationError> {
    if config.auth.app_key.trim().is_empty() || config.auth.access_key.trim().is_empty() {
        return Some(AsrConfigurationError {
            code: "ASR_AUTH_MISSING",
            message: "请先填写豆包 App Key 和 Access Key。".to_string(),
        });
    }
    if config.auth.resource_id.trim().is_empty() {
        return Some(AsrConfigurationError {
            code: "ASR_AUTH_MISSING",
            message: "请先填写豆包 Resource ID。".to_string(),
        });
    }
    None
}

fn aliyun_configuration_error(config: &AppConfig) -> Option<AsrConfigurationError> {
    if config.aliyun_asr.api_key.trim().is_empty() {
        return Some(AsrConfigurationError {
            code: "ASR_AUTH_MISSING",
            message: "请先填写阿里云 ASR API Key。".to_string(),
        });
    }
    if config.aliyun_asr.model.trim().is_empty() {
        return Some(AsrConfigurationError {
            code: "ASR_AUTH_MISSING",
            message: "请先填写阿里云 ASR 模型名。".to_string(),
        });
    }
    if config.aliyun_asr.workspace_id.trim().is_empty()
        && config.aliyun_asr.websocket_url.trim().is_empty()
    {
        return Some(AsrConfigurationError {
            code: "ASR_AUTH_MISSING",
            message: "请先填写阿里云 ASR Workspace ID，或填写自定义 WebSocket 地址。".to_string(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ASR_PROVIDER_ALIYUN_FUN;

    #[test]
    fn doubao_is_current_default_provider() {
        let config = AppConfig::default();
        assert_eq!(active_provider(&config), AsrProviderKind::Doubao);
        assert_eq!(active_provider_label(&config), "豆包 ASR");
    }

    #[test]
    fn doubao_configuration_uses_existing_auth_fields() {
        let mut config = AppConfig::default();
        assert!(configuration_error(&config).is_some());
        assert_eq!(
            configuration_error(&config).unwrap().message,
            "请先填写豆包 App Key 和 Access Key。"
        );

        config.auth.app_key = "app".to_string();
        config.auth.access_key = "access".to_string();
        assert!(configuration_error(&config).is_none());

        config.auth.resource_id.clear();
        assert_eq!(
            configuration_error(&config).unwrap().message,
            "请先填写豆包 Resource ID。"
        );
    }

    #[test]
    fn aliyun_configuration_uses_only_active_provider_fields() {
        let mut config = AppConfig::default();
        config.auth.app_key = "doubao-app".to_string();
        config.auth.access_key = "doubao-access".to_string();
        assert!(configuration_error(&config).is_none());

        config.asr.provider = ASR_PROVIDER_ALIYUN_FUN.to_string();
        assert_eq!(active_provider(&config), AsrProviderKind::AliyunFun);
        assert_eq!(active_provider_label(&config), "阿里云 FunASR");
        assert_eq!(
            configuration_error(&config).unwrap().message,
            "请先填写阿里云 ASR API Key。"
        );

        config.aliyun_asr.api_key = "aliyun-key".to_string();
        config.aliyun_asr.workspace_id = "workspace".to_string();
        config.auth.app_key.clear();
        config.auth.access_key.clear();
        assert!(configuration_error(&config).is_none());
    }

    #[test]
    fn start_configuration_error_preserves_existing_user_messages() {
        let config = AppConfig::default();
        let existing = start_configuration_error(&config, true).unwrap();
        assert_eq!(existing.code, "ASR_AUTH_MISSING");
        assert!(existing.message.contains("豆包认证信息"));

        let missing_file = start_configuration_error(&config, false).unwrap();
        assert_eq!(missing_file.code, "CONFIG_MISSING");
        assert!(missing_file.message.contains("未找到 config.toml"));

        let mut aliyun = AppConfig::default();
        aliyun.asr.provider = ASR_PROVIDER_ALIYUN_FUN.to_string();
        let existing = start_configuration_error(&aliyun, true).unwrap();
        assert_eq!(existing.code, "ASR_AUTH_MISSING");
        assert!(existing.message.contains("阿里云"));
    }

    #[test]
    fn worker_configuration_error_preserves_existing_error_code() {
        let config = AppConfig::default();
        let err = worker_configuration_error(&config).unwrap();
        assert_eq!(err.code, "ASR_AUTH_MISSING");
        assert_eq!(
            err.message,
            "ASR skipped: app_key/access_key is not configured."
        );
    }
}
