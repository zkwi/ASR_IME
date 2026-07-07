pub(super) const ASR_CONNECT_TIMEOUT_MESSAGE: &str = "连接豆包 ASR 超时，请检查网络或代理后重试。";
pub(super) const ASR_FINAL_TIMEOUT_MESSAGE: &str = "等待豆包 ASR 最终结果超时，请检查网络后重试。";
pub(super) const ASR_CONNECTION_CLOSED_MESSAGE: &str =
    "豆包 ASR 连接已结束，但未返回完整最终结果。请重试，或检查网络稳定性。";

pub(super) fn classify_asr_error(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if error.contains("认证")
        || error.contains("权限")
        || error.contains("App Key")
        || error.contains("Access Key")
        || error.contains("Resource ID")
        || error.contains("API Key")
        || error.contains("Workspace")
        || error.contains("Bearer")
        || error.contains("阿里云 ASR 认证")
    {
        "ASR_AUTH_MISSING"
    } else if error == ASR_CONNECT_TIMEOUT_MESSAGE {
        "ASR_CONNECT_TIMEOUT"
    } else if error == ASR_FINAL_TIMEOUT_MESSAGE || error.contains("最终结果超时") {
        "ASR_FINAL_TIMEOUT"
    } else if error == ASR_CONNECTION_CLOSED_MESSAGE
        || error.contains("连接已结束")
        || error.contains("连接已关闭")
        || error.contains("连接已中断")
        || lower.contains("connection closed")
        || lower.contains("connection reset")
        || lower.contains("already closed")
        || lower.contains("reset without closing handshake")
        || lower.contains("reset by peer")
        || lower.contains("broken pipe")
        || lower.contains("unexpected eof")
    {
        "ASR_CONNECTION_CLOSED"
    } else if error.contains("无法连接豆包 ASR")
        || error.contains("连接豆包 ASR 失败")
        || error.contains("无法连接阿里云 ASR")
        || error.contains("连接阿里云 ASR 失败")
        || lower.contains("dns")
        || lower.contains("resolve")
        || lower.contains("connect")
        || lower.contains("proxy")
        || lower.contains("tls")
    {
        "ASR_CONNECT_FAILED"
    } else if error.contains("超时") || lower.contains("timeout") || lower.contains("timed out") {
        "ASR_FINAL_TIMEOUT"
    } else {
        "ASR_NETWORK_FAILED"
    }
}

pub(super) fn is_success_code(code: i32) -> bool {
    code == 0 || code == 20_000_000
}

pub(super) fn friendly_asr_connection_error(error: &str) -> String {
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

pub(super) fn friendly_asr_service_error(code: i32) -> String {
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
        classify_asr_error, friendly_asr_connection_error, friendly_asr_service_error,
        is_success_code, ASR_FINAL_TIMEOUT_MESSAGE,
    };
    use crate::asr_ws::final_text::missing_final_result_error;

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
    fn final_timeout_and_connection_closed_use_distinct_error_codes() {
        assert_eq!(
            classify_asr_error(ASR_FINAL_TIMEOUT_MESSAGE),
            "ASR_FINAL_TIMEOUT"
        );
        assert_eq!(
            missing_final_result_error(false),
            ASR_FINAL_TIMEOUT_MESSAGE.to_string()
        );
        assert_eq!(
            classify_asr_error(&missing_final_result_error(true)),
            "ASR_CONNECTION_CLOSED"
        );
        assert_eq!(
            classify_asr_error("豆包 ASR 连接已中断: connection closed"),
            "ASR_CONNECTION_CLOSED"
        );
    }
}
