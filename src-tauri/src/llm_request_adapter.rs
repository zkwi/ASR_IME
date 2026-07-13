use serde_json::{json, Value};

pub const STRATEGY_AUTO: &str = "auto";
pub const STRATEGY_DASHSCOPE_ENABLE_THINKING: &str = "dashscope_enable_thinking";
pub const STRATEGY_THINKING_DISABLED: &str = "thinking_disabled";
pub const STRATEGY_OPENROUTER_REASONING_LOW: &str = "openrouter_reasoning_low";
pub const STRATEGY_OPENROUTER_REASONING_MINIMAL: &str = "openrouter_reasoning_minimal";
pub const STRATEGY_OMIT: &str = "omit";

const ALLOWED_STRATEGIES: &[&str] = &[
    STRATEGY_AUTO,
    STRATEGY_DASHSCOPE_ENABLE_THINKING,
    STRATEGY_THINKING_DISABLED,
    STRATEGY_OPENROUTER_REASONING_LOW,
    STRATEGY_OPENROUTER_REASONING_MINIMAL,
    STRATEGY_OMIT,
];

pub fn is_valid_thinking_strategy(strategy: &str) -> bool {
    ALLOWED_STRATEGIES.contains(&strategy)
}

pub fn is_thinking_only_model(base_url: &str, model: &str) -> bool {
    if !is_dashscope(base_url) {
        return false;
    }

    matches!(
        model.trim().to_ascii_lowercase().as_str(),
        "qwen3.7-max-preview" | "qwen3.7-max-2026-05-17"
    )
}

pub fn effective_thinking_strategy(base_url: &str, model: &str, configured: &str) -> &'static str {
    if is_dashscope(base_url) && configured == STRATEGY_OMIT {
        return STRATEGY_DASHSCOPE_ENABLE_THINKING;
    }
    if configured != STRATEGY_AUTO {
        return normalize_thinking_strategy(configured);
    }
    preferred_auto_strategy(base_url, model)
}

pub fn thinking_strategy_candidates(
    base_url: &str,
    model: &str,
    configured: &str,
) -> Vec<&'static str> {
    if configured != STRATEGY_AUTO {
        let normalized = effective_thinking_strategy(base_url, model, configured);
        if normalized == STRATEGY_OMIT || normalized == preferred_auto_strategy(base_url, model) {
            return vec![normalized];
        }
    }

    let base_url = base_url.to_ascii_lowercase();
    let model = model.to_ascii_lowercase();
    if base_url.contains("dashscope.aliyuncs.com") {
        return vec![STRATEGY_DASHSCOPE_ENABLE_THINKING];
    }
    if base_url.contains("api.deepseek.com") || base_url.contains("api.xiaomimimo.com") {
        return vec![
            STRATEGY_THINKING_DISABLED,
            STRATEGY_DASHSCOPE_ENABLE_THINKING,
            STRATEGY_OMIT,
        ];
    }
    if base_url.contains("openrouter.ai") {
        if model.starts_with("stepfun/") {
            return vec![
                STRATEGY_OPENROUTER_REASONING_LOW,
                STRATEGY_OPENROUTER_REASONING_MINIMAL,
                STRATEGY_DASHSCOPE_ENABLE_THINKING,
            ];
        }
        return vec![
            STRATEGY_OPENROUTER_REASONING_MINIMAL,
            STRATEGY_OPENROUTER_REASONING_LOW,
            STRATEGY_DASHSCOPE_ENABLE_THINKING,
        ];
    }

    vec![
        STRATEGY_DASHSCOPE_ENABLE_THINKING,
        STRATEGY_THINKING_DISABLED,
        STRATEGY_OMIT,
    ]
}

pub fn apply_thinking_strategy(
    body: &mut Value,
    base_url: &str,
    model: &str,
    enable_thinking: bool,
    configured_strategy: &str,
) -> &'static str {
    if enable_thinking {
        body["enable_thinking"] = json!(true);
        return STRATEGY_DASHSCOPE_ENABLE_THINKING;
    }

    let strategy = effective_thinking_strategy(base_url, model, configured_strategy);
    match strategy {
        STRATEGY_DASHSCOPE_ENABLE_THINKING => {
            body["enable_thinking"] = json!(false);
        }
        STRATEGY_THINKING_DISABLED => {
            body["thinking"] = json!({ "type": "disabled" });
        }
        STRATEGY_OPENROUTER_REASONING_LOW => {
            body["reasoning_effort"] = json!("low");
        }
        STRATEGY_OPENROUTER_REASONING_MINIMAL => {
            body["reasoning"] = json!({ "effort": "minimal", "exclude": true });
        }
        STRATEGY_OMIT | STRATEGY_AUTO => {}
        _ => {}
    }
    strategy
}

pub fn remove_thinking_controls(body: &mut Value) {
    if let Some(object) = body.as_object_mut() {
        object.remove("enable_thinking");
        object.remove("thinking");
        object.remove("reasoning_effort");
        object.remove("reasoning");
    }
}

pub fn disabled_thinking_controls(body: &Value) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if body.get("enable_thinking").and_then(Value::as_bool) == Some(false) {
        fields.push("enable_thinking");
    }
    if body
        .get("thinking")
        .and_then(|item| item.get("type"))
        .and_then(Value::as_str)
        == Some("disabled")
    {
        fields.push("thinking");
    }
    if matches!(
        body.get("reasoning_effort").and_then(Value::as_str),
        Some("none" | "minimal" | "low")
    ) {
        fields.push("reasoning_effort");
    }
    if body.get("reasoning").is_some() {
        fields.push("reasoning");
    }
    fields
}

pub fn should_retry_without_unsupported_thinking(
    base_url: &str,
    body: &Value,
    error: &str,
) -> bool {
    let controls = disabled_thinking_controls(body);
    if controls.is_empty() {
        return false;
    }
    if base_url.contains("dashscope.aliyuncs.com")
        && controls.len() == 1
        && controls[0] == "enable_thinking"
    {
        return false;
    }
    let lower = error.to_ascii_lowercase();
    if lower.contains("restricted to true")
        || lower.contains("mandatory")
        || lower.contains("cannot be disabled")
    {
        return false;
    }
    controls.iter().any(|field| lower.contains(field))
        && (lower.contains("unknown")
            || lower.contains("unrecognized")
            || lower.contains("unsupported")
            || lower.contains("not support")
            || lower.contains("unexpected")
            || lower.contains("extra")
            || lower.contains("additional")
            || lower.contains("invalid_request_error"))
}

fn preferred_auto_strategy(base_url: &str, model: &str) -> &'static str {
    let base_url = base_url.to_ascii_lowercase();
    let model = model.to_ascii_lowercase();
    if base_url.contains("dashscope.aliyuncs.com") {
        return STRATEGY_DASHSCOPE_ENABLE_THINKING;
    }
    if base_url.contains("api.deepseek.com") || base_url.contains("api.xiaomimimo.com") {
        return STRATEGY_THINKING_DISABLED;
    }
    if base_url.contains("openrouter.ai") && model.starts_with("stepfun/") {
        return STRATEGY_OPENROUTER_REASONING_LOW;
    }
    if base_url.contains("openrouter.ai") {
        return STRATEGY_OPENROUTER_REASONING_MINIMAL;
    }
    STRATEGY_DASHSCOPE_ENABLE_THINKING
}

fn is_dashscope(base_url: &str) -> bool {
    base_url
        .to_ascii_lowercase()
        .contains("dashscope.aliyuncs.com")
}

fn normalize_thinking_strategy(strategy: &str) -> &'static str {
    match strategy {
        STRATEGY_DASHSCOPE_ENABLE_THINKING => STRATEGY_DASHSCOPE_ENABLE_THINKING,
        STRATEGY_THINKING_DISABLED => STRATEGY_THINKING_DISABLED,
        STRATEGY_OPENROUTER_REASONING_LOW => STRATEGY_OPENROUTER_REASONING_LOW,
        STRATEGY_OPENROUTER_REASONING_MINIMAL => STRATEGY_OPENROUTER_REASONING_MINIMAL,
        STRATEGY_OMIT => STRATEGY_OMIT,
        _ => STRATEGY_AUTO,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_thinking_strategy, disabled_thinking_controls, effective_thinking_strategy,
        is_thinking_only_model, is_valid_thinking_strategy,
        should_retry_without_unsupported_thinking, thinking_strategy_candidates, STRATEGY_AUTO,
        STRATEGY_DASHSCOPE_ENABLE_THINKING, STRATEGY_OMIT, STRATEGY_OPENROUTER_REASONING_LOW,
        STRATEGY_OPENROUTER_REASONING_MINIMAL, STRATEGY_THINKING_DISABLED,
    };
    use serde_json::json;

    #[test]
    fn validates_known_strategy_names() {
        assert!(is_valid_thinking_strategy(STRATEGY_AUTO));
        assert!(is_valid_thinking_strategy(STRATEGY_THINKING_DISABLED));
        assert!(!is_valid_thinking_strategy("reasoning_none"));
    }

    #[test]
    fn recognizes_dashscope_thinking_only_models() {
        let base_url = "https://dashscope.aliyuncs.com/compatible-mode/v1";
        assert!(is_thinking_only_model(base_url, "qwen3.7-max-2026-05-17"));
        assert!(is_thinking_only_model(base_url, "qwen3.7-max-preview"));
        assert!(!is_thinking_only_model(base_url, "qwen3.7-max"));
        assert!(!is_thinking_only_model(
            "https://api.example.com/v1",
            "qwen3.7-max-2026-05-17"
        ));
    }

    #[test]
    fn chooses_provider_specific_auto_strategy() {
        assert_eq!(
            effective_thinking_strategy(
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
                "qwen3.7-max",
                STRATEGY_AUTO,
            ),
            STRATEGY_DASHSCOPE_ENABLE_THINKING
        );
        assert_eq!(
            effective_thinking_strategy(
                "https://api.deepseek.com",
                "deepseek-v4-pro",
                STRATEGY_AUTO,
            ),
            STRATEGY_THINKING_DISABLED
        );
        assert_eq!(
            effective_thinking_strategy(
                "https://api.xiaomimimo.com/v1",
                "mimo-v2.5-pro",
                STRATEGY_AUTO,
            ),
            STRATEGY_THINKING_DISABLED
        );
        assert_eq!(
            effective_thinking_strategy(
                "https://openrouter.ai/api/v1",
                "stepfun/step-3.7-flash",
                STRATEGY_AUTO,
            ),
            STRATEGY_OPENROUTER_REASONING_LOW
        );
        assert_eq!(
            effective_thinking_strategy(
                "https://openrouter.ai/api/v1",
                "x-ai/grok-build-0.1",
                STRATEGY_AUTO,
            ),
            STRATEGY_OPENROUTER_REASONING_MINIMAL
        );
    }

    #[test]
    fn builds_candidate_sets_for_connection_test() {
        assert_eq!(
            thinking_strategy_candidates(
                "https://dashscope.aliyuncs.com/compatible-mode/v1",
                "qwen3.7-max",
                STRATEGY_AUTO
            ),
            vec![STRATEGY_DASHSCOPE_ENABLE_THINKING]
        );
        assert_eq!(
            thinking_strategy_candidates(
                "https://api.deepseek.com",
                "deepseek-v4-pro",
                STRATEGY_AUTO
            ),
            vec![
                STRATEGY_THINKING_DISABLED,
                STRATEGY_DASHSCOPE_ENABLE_THINKING,
                STRATEGY_OMIT
            ]
        );
        assert_eq!(
            thinking_strategy_candidates(
                "https://openrouter.ai/api/v1",
                "stepfun/step-3.7-flash",
                STRATEGY_AUTO
            )[0],
            STRATEGY_OPENROUTER_REASONING_LOW
        );
    }

    #[test]
    fn applies_disabled_strategy_to_request_body() {
        let mut body = json!({});
        let used = apply_thinking_strategy(
            &mut body,
            "https://api.deepseek.com",
            "deepseek-v4-pro",
            false,
            STRATEGY_AUTO,
        );
        assert_eq!(used, STRATEGY_THINKING_DISABLED);
        assert_eq!(body["thinking"], json!({ "type": "disabled" }));

        let mut body = json!({});
        let used = apply_thinking_strategy(
            &mut body,
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "qwen3.7-max",
            false,
            STRATEGY_AUTO,
        );
        assert_eq!(used, STRATEGY_DASHSCOPE_ENABLE_THINKING);
        assert_eq!(body["enable_thinking"], json!(false));

        let mut body = json!({});
        let used = apply_thinking_strategy(
            &mut body,
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "qwen3.7-max",
            false,
            STRATEGY_OMIT,
        );
        assert_eq!(used, STRATEGY_DASHSCOPE_ENABLE_THINKING);
        assert_eq!(body["enable_thinking"], json!(false));
    }

    #[test]
    fn retries_only_for_unsupported_disabled_controls() {
        let body = json!({ "thinking": { "type": "disabled" } });
        assert!(disabled_thinking_controls(&body).contains(&"thinking"));
        assert!(should_retry_without_unsupported_thinking(
            "https://api.example.com/v1",
            &body,
            "400 unknown field: thinking"
        ));
        assert!(!should_retry_without_unsupported_thinking(
            "https://openrouter.ai/api/v1",
            &json!({ "reasoning_effort": "none" }),
            "Reasoning is mandatory for this endpoint and cannot be disabled."
        ));
    }
}
