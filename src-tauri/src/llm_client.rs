use crate::{
    app_log,
    llm_endpoint::chat_completions_endpoint,
    llm_request_adapter::{remove_thinking_controls, should_retry_without_unsupported_thinking},
};
use serde_json::{json, Value};
use std::time::Duration;

pub struct ChatJsonResult {
    pub value: Value,
    pub retried_without_thinking: bool,
}

pub fn build_client(timeout: Duration) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder().timeout(timeout).build()
}

pub fn base_chat_body(model: &str, system_prompt: &str, user_prompt: &str) -> Value {
    json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ]
    })
}

pub async fn send_chat_with_thinking_fallback(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    body: &Value,
    parse_error_context: &str,
) -> Result<ChatJsonResult, String> {
    let response = send_chat_request(client, base_url, api_key, body).await?;
    match parse_chat_response(response, parse_error_context).await {
        Ok(value) => Ok(ChatJsonResult {
            value,
            retried_without_thinking: false,
        }),
        Err(err) if should_retry_without_unsupported_thinking(base_url, body, &err) => {
            app_log::info(
                "LLM provider rejected disabled thinking controls, retrying without them",
            );
            let mut retry_body = body.clone();
            remove_thinking_controls(&mut retry_body);
            let response = send_chat_request(client, base_url, api_key, &retry_body).await?;
            let value = parse_chat_response(response, parse_error_context).await?;
            Ok(ChatJsonResult {
                value,
                retried_without_thinking: true,
            })
        }
        Err(err) => Err(err),
    }
}

pub fn extract_message_content(value: &Value) -> String {
    extract_message_string_field(value, "content")
}

pub fn extract_reasoning_content(value: &Value) -> String {
    let legacy = extract_message_string_field(value, "reasoning_content");
    if legacy.is_empty() {
        extract_message_string_field(value, "reasoning")
    } else {
        legacy
    }
}

pub fn response_was_truncated(value: &Value) -> bool {
    value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("finish_reason"))
        .and_then(Value::as_str)
        .is_some_and(|reason| matches!(reason, "length" | "max_tokens"))
}

fn extract_message_string_field(value: &Value, field: &str) -> String {
    value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get(field))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

async fn send_chat_request(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    body: &Value,
) -> Result<reqwest::Response, String> {
    client
        .post(chat_completions_endpoint(base_url))
        .bearer_auth(api_key)
        .json(body)
        .send()
        .await
        .map_err(|err| format!("调用 LLM 失败: {}", err))
}

async fn parse_chat_response(
    response: reqwest::Response,
    parse_error_context: &str,
) -> Result<Value, String> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("LLM 返回状态码: {}; {}", status, body));
    }
    let value: Value = response
        .json()
        .await
        .map_err(|err| format!("{}: {}", parse_error_context, err))?;
    if let Some(error) = value.get("error") {
        return Err(format!("LLM 返回错误: {}", error));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{
        base_chat_body, extract_message_content, extract_reasoning_content, response_was_truncated,
    };
    use serde_json::json;

    #[test]
    fn extracts_content_and_reasoning_from_first_choice() {
        let value = json!({
            "choices": [{
                "message": {
                    "content": "final text",
                    "reasoning_content": "private reasoning"
                }
            }]
        });

        assert_eq!(extract_message_content(&value), "final text");
        assert_eq!(extract_reasoning_content(&value), "private reasoning");
    }

    #[test]
    fn extracts_openrouter_reasoning_field() {
        let value = json!({
            "choices": [{
                "message": {
                    "content": "",
                    "reasoning": "gateway reasoning"
                }
            }]
        });

        assert_eq!(extract_reasoning_content(&value), "gateway reasoning");
    }

    #[test]
    fn detects_token_limit_truncation_only() {
        assert!(response_was_truncated(&json!({
            "choices": [{"finish_reason": "length"}]
        })));
        assert!(response_was_truncated(&json!({
            "choices": [{"finish_reason": "max_tokens"}]
        })));
        assert!(!response_was_truncated(&json!({
            "choices": [{"finish_reason": "stop"}]
        })));
    }

    #[test]
    fn builds_openai_compatible_message_body() {
        let body = base_chat_body("test-model", "system prompt", "user prompt");

        assert_eq!(body["model"], "test-model");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "system prompt");
        assert_eq!(body["messages"][1]["role"], "user");
        assert_eq!(body["messages"][1]["content"], "user prompt");
    }
}
