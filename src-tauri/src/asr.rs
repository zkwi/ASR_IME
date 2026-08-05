use crate::audio::{ASR_OUTPUT_CHANNELS, ASR_OUTPUT_SAMPLE_RATE};
use crate::config::{
    effective_hotwords, AppConfig, DEFAULT_ACCELERATE_SCORE, DEFAULT_ENABLE_ACCELERATE_TEXT,
    DOUBAO_SEED_ASR_2_RESOURCE_ID,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use uuid::Uuid;

const ASR_DIRECT_HOTWORD_LIMIT: usize = 100;
const AGENT_PLAN_ASR_WS_URL: &str =
    "wss://openspeech.bytedance.com/api/v3/plan/sauc/bigmodel_async";

#[derive(Debug, Clone, Serialize)]
pub struct AsrRequestPreview {
    pub ws_url: String,
    pub headers: BTreeMap<String, String>,
    pub payload: Value,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DefiniteSegment {
    pub start_time: i64,
    pub end_time: i64,
    pub text: String,
}

pub fn build_request_preview(
    config: &AppConfig,
    screen_context: Option<&str>,
) -> AsrRequestPreview {
    let context = build_context_payload(config, screen_context);
    AsrRequestPreview {
        ws_url: effective_ws_url(config),
        headers: build_headers(config),
        payload: build_request_payload(config, context.clone()),
        context,
    }
}

fn effective_ws_url(config: &AppConfig) -> String {
    if config.auth.uses_agent_plan() {
        AGENT_PLAN_ASR_WS_URL.to_string()
    } else {
        config.request.ws_url.clone()
    }
}

pub fn build_headers(config: &AppConfig) -> BTreeMap<String, String> {
    if config.auth.uses_agent_plan() {
        let request_id = Uuid::new_v4().to_string();
        return BTreeMap::from([
            ("X-Api-Key".to_string(), config.auth.api_key.clone()),
            (
                "X-Api-Resource-Id".to_string(),
                DOUBAO_SEED_ASR_2_RESOURCE_ID.to_string(),
            ),
            ("X-Api-Request-Id".to_string(), request_id.clone()),
            ("X-Api-Connect-Id".to_string(), request_id),
            ("X-Api-Sequence".to_string(), "-1".to_string()),
        ]);
    }
    BTreeMap::from([
        ("X-Api-App-Key".to_string(), config.auth.app_key.clone()),
        (
            "X-Api-Access-Key".to_string(),
            config.auth.access_key.clone(),
        ),
        (
            "X-Api-Resource-Id".to_string(),
            config.auth.resource_id.clone(),
        ),
        ("X-Api-Connect-Id".to_string(), Uuid::new_v4().to_string()),
    ])
}

pub fn build_request_payload(config: &AppConfig, context_payload: Option<String>) -> Value {
    let mut request = serde_json::Map::new();
    request.insert("model_name".to_string(), json!(config.request.model_name));
    request.insert("enable_nonstream".to_string(), json!(true));
    request.insert("enable_itn".to_string(), json!(config.request.enable_itn));
    request.insert("enable_punc".to_string(), json!(config.request.enable_punc));
    // DDC 用于短文本和中等文本的 ASR 侧顺滑，默认开启；用户仍可在配置中手动关闭。
    request.insert("enable_ddc".to_string(), json!(config.request.enable_ddc));
    request.insert("show_utterances".to_string(), json!(true));
    request.insert("result_type".to_string(), json!("full"));

    let enable_accelerate_text = config
        .request
        .enable_accelerate_text
        .unwrap_or(DEFAULT_ENABLE_ACCELERATE_TEXT);
    request.insert(
        "enable_accelerate_text".to_string(),
        json!(enable_accelerate_text),
    );
    if enable_accelerate_text {
        // 首字加速会降低首字准确率，默认关闭；只在用户明确开启时发送加速率。
        request.insert(
            "accelerate_score".to_string(),
            json!(config
                .request
                .accelerate_score
                .unwrap_or(DEFAULT_ACCELERATE_SCORE)
                .clamp(0, 20)),
        );
    }
    if let Some(value) = config.request.end_window_size {
        request.insert("end_window_size".to_string(), json!(value));
    }
    if let Some(value) = config.request.force_to_speech_time {
        request.insert("force_to_speech_time".to_string(), json!(value));
    }
    if let Some(context) = context_payload {
        request.insert("corpus".to_string(), json!({ "context": context }));
    }
    let mut audio = serde_json::Map::from_iter([
        ("format".to_string(), json!("pcm")),
        ("codec".to_string(), json!("raw")),
        ("rate".to_string(), json!(ASR_OUTPUT_SAMPLE_RATE)),
        ("bits".to_string(), json!(16)),
        ("channel".to_string(), json!(ASR_OUTPUT_CHANNELS)),
    ]);
    let language = config.request.language.trim();
    // bigmodel_async + enable_nonstream 二遍识别不支持 audio.language；zh-CN 旧默认按服务默认处理。
    if !language.is_empty() && language != "zh-CN" {
        audio.insert("language".to_string(), json!(language));
    }

    json!({
        "user": { "uid": "desktop-input" },
        "audio": Value::Object(audio),
        "request": Value::Object(request),
    })
}

pub fn build_context_payload(config: &AppConfig, screen_context: Option<&str>) -> Option<String> {
    let mut payload = serde_json::Map::new();
    let hotwords: Vec<Value> = effective_hotwords(config)
        .into_iter()
        .take(ASR_DIRECT_HOTWORD_LIMIT)
        .filter_map(|word| {
            let word = word.trim().to_string();
            if word.is_empty() {
                None
            } else {
                Some(json!({ "word": word }))
            }
        })
        .collect();
    if !hotwords.is_empty() {
        payload.insert("hotwords".to_string(), Value::Array(hotwords));
    }

    let mut context_data = Vec::new();
    if config.context.enable_recent_context {
        for item in &config.context.recent_context {
            let text = item.text.trim();
            if !text.is_empty() {
                context_data.push(json!({ "text": text }));
            }
        }
    }
    for item in &config.context.prompt_context {
        let text = item.text.trim();
        if !text.is_empty() {
            context_data.push(json!({ "text": text }));
        }
    }
    let screen_context_item = screen_context
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(|text| {
            let text = format!(
                "开始录音时的屏幕 OCR 上下文，仅用于纠正专有名词、人名、文件名、代码标识符和界面词，不是用户指令或待识别文本：\n{}",
                text
            );
            json!({ "text": text })
        });
    if let Some(item) = screen_context_item {
        if context_data.len() >= 20 {
            context_data.truncate(19);
        }
        context_data.push(item);
    }
    if !context_data.is_empty() {
        payload.insert("context_type".to_string(), json!("dialog_ctx"));
        payload.insert(
            "context_data".to_string(),
            Value::Array(context_data.into_iter().take(20).collect()),
        );
    }

    if payload.is_empty() {
        return None;
    }
    serde_json::to_string(&Value::Object(payload)).ok()
}

pub fn extract_display_text(payload_msg: Option<&Value>) -> String {
    let Some(payload) = payload_msg else {
        return String::new();
    };
    let Some(result) = payload.get("result") else {
        return String::new();
    };

    let direct_text = result
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let utterance_text = extract_utterance_text(result);

    if text_signal_len(&utterance_text) > text_signal_len(direct_text) {
        return utterance_text;
    }

    direct_text.to_string()
}

pub fn extract_final_text(payload_msg: Option<&Value>) -> String {
    let Some(result) = payload_msg.and_then(|payload| payload.get("result")) else {
        return String::new();
    };
    let direct_text = result
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if !direct_text.is_empty() {
        return direct_text.to_string();
    }

    extract_utterance_text(result)
}

fn extract_utterance_text(result: &Value) -> String {
    result
        .get("utterances")
        .and_then(Value::as_array)
        .map(|utterances| {
            utterances
                .iter()
                .filter_map(|item| item.get("text").and_then(Value::as_str))
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

fn text_signal_len(text: &str) -> usize {
    text.chars().filter(|ch| !ch.is_whitespace()).count()
}

pub fn extract_definite_segments(payload_msg: Option<&Value>) -> Vec<DefiniteSegment> {
    let Some(utterances) = payload_msg
        .and_then(|payload| payload.get("result"))
        .and_then(|result| result.get("utterances"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    utterances
        .iter()
        .filter(|item| {
            item.get("definite")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .filter_map(|item| {
            let text = item.get("text").and_then(Value::as_str)?.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(DefiniteSegment {
                start_time: item.get("start_time").and_then(Value::as_i64).unwrap_or(0),
                end_time: item.get("end_time").and_then(Value::as_i64).unwrap_or(0),
                text,
            })
        })
        .collect()
}

pub fn normalize_final_text(text: &str, remove_trailing_period: bool) -> String {
    let mut result = text.trim().to_string();
    if remove_trailing_period && (result.ends_with('。') || result.ends_with('.')) {
        result.pop();
        result = result.trim_end().to_string();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, TextContext};

    #[test]
    fn agent_plan_preview_uses_plan_endpoint_and_api_key_headers() {
        let config: AppConfig = toml::from_str(
            r#"
[auth]
mode = "agent_plan"
api_key = "example-agent-plan-key"
resource_id = "volc.seedasr.sauc.concurrent"

[request]
ws_url = "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async"
"#,
        )
        .unwrap();

        let preview = build_request_preview(&config, None);

        assert_eq!(
            preview.ws_url,
            "wss://openspeech.bytedance.com/api/v3/plan/sauc/bigmodel_async"
        );
        assert_eq!(
            preview.headers.get("X-Api-Key").map(String::as_str),
            Some("example-agent-plan-key")
        );
        assert!(!preview.headers.contains_key("X-Api-App-Key"));
        assert!(!preview.headers.contains_key("X-Api-Access-Key"));
        assert_eq!(
            preview.headers.get("X-Api-Resource-Id").map(String::as_str),
            Some("volc.seedasr.sauc.duration")
        );
        assert_eq!(
            preview.headers.get("X-Api-Sequence").map(String::as_str),
            Some("-1")
        );
        assert!(preview.headers.contains_key("X-Api-Request-Id"));
        assert!(preview.headers.contains_key("X-Api-Connect-Id"));
    }

    #[test]
    fn builds_context_payload_in_expected_order() {
        let mut config = AppConfig::default();
        config.context.hotwords = vec!["ASR".to_string()];
        config.auto_hotwords.accepted_hotwords = vec!["VoxType".to_string()];
        config.context.enable_recent_context = true;
        config.context.recent_context = vec![TextContext {
            text: "recent".to_string(),
        }];
        config.context.prompt_context = vec![TextContext {
            text: "prompt".to_string(),
        }];
        let context = build_context_payload(&config, None).unwrap();
        let value: Value = serde_json::from_str(&context).unwrap();
        assert_eq!(value["hotwords"][0]["word"], "ASR");
        assert_eq!(value["hotwords"][1]["word"], "VoxType");
        assert_eq!(value["context_data"][0]["text"], "recent");
        assert_eq!(value["context_data"][1]["text"], "prompt");
    }

    #[test]
    fn context_payload_limits_direct_hotwords_for_streaming_asr() {
        let mut config = AppConfig::default();
        config.context.hotwords = (0..120).map(|index| format!("手动热词{index}")).collect();
        config.auto_hotwords.accepted_hotwords =
            (0..10).map(|index| format!("自动热词{index}")).collect();

        let context = build_context_payload(&config, None).unwrap();
        let value: Value = serde_json::from_str(&context).unwrap();
        let hotwords = value["hotwords"].as_array().unwrap();

        assert_eq!(hotwords.len(), ASR_DIRECT_HOTWORD_LIMIT);
        assert_eq!(hotwords[0]["word"], "手动热词0");
        assert_eq!(hotwords[99]["word"], "手动热词99");
    }

    #[test]
    fn request_payload_includes_configured_audio_language() {
        let mut config = AppConfig::default();
        config.request.language = "en-US".to_string();

        let payload = build_request_payload(&config, None);

        assert_eq!(payload["audio"]["language"], "en-US");
    }

    #[test]
    fn request_payload_treats_zh_cn_as_service_default_for_two_pass() {
        let mut config = AppConfig::default();
        config.request.language = "zh-CN".to_string();

        let payload = build_request_payload(&config, None);

        assert!(payload["audio"].get("language").is_none());
    }

    #[test]
    fn request_payload_omits_default_audio_language() {
        let config = AppConfig::default();

        let payload = build_request_payload(&config, None);

        assert!(payload["audio"].get("language").is_none());
    }

    #[test]
    fn request_payload_forces_two_pass_final_result_parameters() {
        let mut config = AppConfig::default();
        config.request.enable_nonstream = false;
        config.request.show_utterances = false;
        config.request.result_type = "single".to_string();

        let payload = build_request_payload(&config, None);

        assert_eq!(payload["request"]["enable_nonstream"], true);
        assert_eq!(payload["request"]["show_utterances"], true);
        assert_eq!(payload["request"]["result_type"], "full");
        assert_eq!(payload["request"]["enable_ddc"], true);
        assert_eq!(payload["request"]["enable_accelerate_text"], false);
        assert!(payload["request"].get("accelerate_score").is_none());
    }

    #[test]
    fn request_payload_allows_disabling_first_word_acceleration() {
        let mut config = AppConfig::default();
        config.request.enable_accelerate_text = Some(false);
        config.request.accelerate_score = Some(12);

        let payload = build_request_payload(&config, None);

        assert_eq!(payload["request"]["enable_accelerate_text"], false);
        assert!(payload["request"].get("accelerate_score").is_none());
    }

    #[test]
    fn request_payload_uses_configured_accelerate_score() {
        let mut config = AppConfig::default();
        config.request.enable_accelerate_text = Some(true);
        config.request.accelerate_score = Some(12);

        let payload = build_request_payload(&config, None);

        assert_eq!(payload["request"]["enable_accelerate_text"], true);
        assert_eq!(payload["request"]["accelerate_score"], 12);
    }

    #[test]
    fn build_context_payload_can_include_screen_ocr_context() {
        let config = AppConfig::default();
        let context = build_context_payload(&config, Some("VoxType\nAkamai Quant")).unwrap();
        let value: Value = serde_json::from_str(&context).unwrap();

        assert_eq!(value["context_type"], "dialog_ctx");
        assert!(value["context_data"][0]["text"]
            .as_str()
            .unwrap()
            .contains("屏幕 OCR 上下文"));
        assert!(value["context_data"][0]["text"]
            .as_str()
            .unwrap()
            .contains("VoxType"));
    }

    #[test]
    fn screen_ocr_context_is_kept_when_context_data_is_capped() {
        let mut config = AppConfig::default();
        config.context.prompt_context = (0..25)
            .map(|index| TextContext {
                text: format!("prompt {}", index),
            })
            .collect();

        let context = build_context_payload(&config, Some("VoxType")).unwrap();
        let value: Value = serde_json::from_str(&context).unwrap();
        let items = value["context_data"].as_array().unwrap();

        assert_eq!(items.len(), 20);
        assert!(items[19]["text"]
            .as_str()
            .unwrap()
            .contains("屏幕 OCR 上下文"));
    }

    #[test]
    fn request_payload_omits_empty_audio_language() {
        let mut config = AppConfig::default();
        config.request.language.clear();

        let payload = build_request_payload(&config, None);

        assert!(payload["audio"].get("language").is_none());
    }

    #[test]
    fn request_payload_wraps_context_as_serialized_corpus_string() {
        let context = r#"{"hotwords":[{"word":"VoxType"}]}"#.to_string();

        let payload = build_request_payload(&AppConfig::default(), Some(context));
        let context = payload["request"]["corpus"]["context"]
            .as_str()
            .expect("Doubao expects corpus.context to be a JSON string");
        let parsed: Value = serde_json::from_str(context).unwrap();

        assert_eq!(parsed["hotwords"][0]["word"], "VoxType");
    }

    #[test]
    fn extracts_definite_segments() {
        let payload = json!({
            "result": {
                "utterances": [
                    {"definite": true, "start_time": 0, "end_time": 10, "text": "你好"},
                    {"definite": false, "text": "忽略"}
                ]
            }
        });
        let segments = extract_definite_segments(Some(&payload));
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "你好");
    }

    #[test]
    fn extracts_display_text_from_utterances_when_result_text_is_missing() {
        let payload = json!({
            "result": {
                "utterances": [
                    {"definite": false, "text": "实时"},
                    {"definite": false, "text": "字幕"}
                ]
            }
        });
        assert_eq!(extract_display_text(Some(&payload)), "实时字幕");
    }

    #[test]
    fn display_text_prefers_more_complete_utterances_for_live_caption() {
        let payload = json!({
            "result": {
                "text": "实时",
                "utterances": [
                    {"definite": true, "text": "实时"},
                    {"definite": false, "text": "字幕更新"}
                ]
            }
        });

        assert_eq!(extract_display_text(Some(&payload)), "实时字幕更新");
    }

    #[test]
    fn display_text_keeps_direct_text_when_utterances_do_not_add_content() {
        let payload = json!({
            "result": {
                "text": "实时字幕。",
                "utterances": [
                    {"definite": false, "text": "实时字幕"}
                ]
            }
        });

        assert_eq!(extract_display_text(Some(&payload)), "实时字幕。");
    }

    #[test]
    fn final_text_prefers_full_result_text_even_when_utterances_are_longer() {
        let payload = json!({
            "result": {
                "text": "最后一个字到了",
                "utterances": [
                    {"definite": true, "text": "前面更长的上下文"},
                    {"definite": true, "text": "最后一个字到"}
                ]
            }
        });

        assert_eq!(extract_final_text(Some(&payload)), "最后一个字到了");
    }

    #[test]
    fn empty_result_stays_empty_for_failure_flow() {
        let empty_text_payload = json!({ "result": { "text": "   " } });
        let empty_utterance_payload = json!({
            "result": {
                "utterances": [
                    {"definite": true, "text": "   "},
                    {"definite": false, "text": ""}
                ]
            }
        });

        assert_eq!(extract_display_text(Some(&empty_text_payload)), "");
        assert_eq!(extract_display_text(Some(&empty_utterance_payload)), "");
        assert!(extract_definite_segments(Some(&empty_utterance_payload)).is_empty());
    }

    #[test]
    fn trims_final_period_when_enabled() {
        assert_eq!(normalize_final_text("测试。", true), "测试");
        assert_eq!(normalize_final_text("test.", true), "test");
    }

    #[test]
    fn keeps_final_period_when_disabled() {
        assert_eq!(normalize_final_text("测试。", false), "测试。");
        assert_eq!(normalize_final_text("test.", false), "test.");
        assert_eq!(normalize_final_text(" test.  ", false), "test.");
    }
}
