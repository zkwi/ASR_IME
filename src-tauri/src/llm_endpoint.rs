pub fn chat_completions_endpoint(configured_url: &str) -> String {
    let url = configured_url.trim().trim_end_matches('/');
    if url.ends_with("/chat/completions") {
        return url.to_string();
    }
    if has_no_path(url) {
        return format!("{}/v1/chat/completions", url);
    }
    format!("{}/chat/completions", url)
}

fn has_no_path(url: &str) -> bool {
    let Some((_, rest)) = url.split_once("://") else {
        return !url.contains('/');
    };
    !rest.contains('/')
}

#[cfg(test)]
mod tests {
    use super::chat_completions_endpoint;

    #[test]
    fn normalizes_deepseek_equivalent_urls() {
        let expected = "https://api.deepseek.com/v1/chat/completions";

        assert_eq!(
            chat_completions_endpoint("https://api.deepseek.com"),
            expected
        );
        assert_eq!(
            chat_completions_endpoint("https://api.deepseek.com/v1/"),
            expected
        );
        assert_eq!(
            chat_completions_endpoint("https://api.deepseek.com/v1/chat/completions"),
            expected
        );
    }

    #[test]
    fn keeps_provider_specific_v1_paths() {
        assert_eq!(
            chat_completions_endpoint("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_endpoint("https://openrouter.ai/api/v1"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
    }
}
