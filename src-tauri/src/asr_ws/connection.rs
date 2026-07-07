use super::audio_stream::silent_test_audio;
use super::errors::{
    friendly_asr_connection_error, friendly_asr_service_error, is_success_code,
    ASR_CONNECT_TIMEOUT_MESSAGE,
};
use crate::{app_log, asr, config::AppConfig, protocol};
use futures_util::{SinkExt, StreamExt};
use std::future::Future;
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};
use tokio_tungstenite::tungstenite::Message;

const ASR_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

pub(crate) async fn test_doubao_connection(config: &AppConfig) -> Result<(), String> {
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

    let (mut websocket, _) = tokio::time::timeout(ASR_CONNECT_TIMEOUT, connect_async(request))
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

pub(super) async fn await_asr_connect<F, T, E>(
    connect_future: F,
    timeout: Duration,
) -> Result<T, String>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    tokio::time::timeout(timeout, connect_future)
        .await
        .map_err(|_| ASR_CONNECT_TIMEOUT_MESSAGE.to_string())?
        .map_err(|err| {
            let detail = err.to_string();
            let message = friendly_asr_connection_error(&detail);
            app_log::warn(format!(
                "连接 ASR WebSocket 失败: {}; user_message={}",
                detail, message
            ));
            message
        })
}

#[cfg(test)]
mod tests {
    use super::await_asr_connect;
    use crate::asr_ws::errors::{classify_asr_error, ASR_CONNECT_TIMEOUT_MESSAGE};
    use std::time::Duration;

    #[test]
    fn formal_connect_timeout_uses_specific_error_code() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let result = runtime.block_on(await_asr_connect(
            async {
                tokio::time::sleep(Duration::from_millis(25)).await;
                Ok::<(), &str>(())
            },
            Duration::from_millis(1),
        ));

        let error = result.unwrap_err();
        assert_eq!(error, ASR_CONNECT_TIMEOUT_MESSAGE);
        assert_eq!(classify_asr_error(&error), "ASR_CONNECT_TIMEOUT");
    }

    #[test]
    fn formal_connect_failure_uses_specific_error_code() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        let result = runtime.block_on(await_asr_connect(
            async { Err::<(), _>("dns lookup failed") },
            Duration::from_secs(1),
        ));

        let error = result.unwrap_err();
        assert!(error.contains("无法连接"));
        assert_eq!(classify_asr_error(&error), "ASR_CONNECT_FAILED");
    }
}
