use crate::overlay;
use std::time::{Duration, Instant};
use tauri::AppHandle;

// 近期实测表明：速度优化应优先放在响应轮询和字幕节流，而不是缩短默认 200ms ASR 音频包。
// 豆包双向流式对 100-200ms 音频包更稳；这里的 50ms 只影响本地显示节流。
// 维护依据见 docs/asr-quality-latency-guardrails.md。
const PARTIAL_TEXT_MIN_INTERVAL: Duration = Duration::from_millis(50);

pub(crate) fn emit_partial_text(app: &AppHandle, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    overlay::update_text(app, text.to_string());
}

pub(super) struct PartialTextLimiter {
    last_emit_at: Option<Instant>,
    last_text: String,
    pending_text: Option<String>,
}

impl PartialTextLimiter {
    pub(super) fn new() -> Self {
        Self {
            last_emit_at: None,
            last_text: String::new(),
            pending_text: None,
        }
    }

    pub(super) fn emit_or_defer(&mut self, text: &str) -> Option<String> {
        let text = text.trim();
        if text.is_empty() || text == self.last_text {
            return None;
        }
        if self.can_emit_now() {
            return Some(self.mark_emitted(text.to_string()));
        }
        self.pending_text = Some(text.to_string());
        None
    }

    pub(super) fn emit_pending_if_ready(&mut self) -> Option<String> {
        if !self.can_emit_now() {
            return None;
        }
        let text = self.pending_text.take()?;
        if text.trim().is_empty() || text == self.last_text {
            return None;
        }
        Some(self.mark_emitted(text))
    }

    fn can_emit_now(&self) -> bool {
        self.last_emit_at
            .map(|last| last.elapsed() >= PARTIAL_TEXT_MIN_INTERVAL)
            .unwrap_or(true)
    }

    fn mark_emitted(&mut self, text: String) -> String {
        self.last_emit_at = Some(Instant::now());
        self.last_text = text.clone();
        self.pending_text = None;
        text
    }
}

pub(super) fn normalize_live_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::{PartialTextLimiter, PARTIAL_TEXT_MIN_INTERVAL};
    use std::time::{Duration, Instant};

    #[test]
    fn partial_text_limiter_allows_faster_live_caption_updates() {
        let mut limiter = PartialTextLimiter::new();

        assert_eq!(PARTIAL_TEXT_MIN_INTERVAL, Duration::from_millis(50));
        assert_eq!(limiter.emit_or_defer("第一段").as_deref(), Some("第一段"));
        assert!(limiter.emit_or_defer("第一段").is_none());
        assert!(limiter.emit_or_defer("").is_none());
    }

    #[test]
    fn partial_text_limiter_coalesces_fast_updates_instead_of_dropping_them() {
        let mut limiter = PartialTextLimiter::new();

        assert_eq!(limiter.emit_or_defer("第一段").as_deref(), Some("第一段"));
        assert!(limiter.emit_or_defer("第一段第二段").is_none());
        assert_eq!(limiter.pending_text.as_deref(), Some("第一段第二段"));
        limiter.last_emit_at = Some(Instant::now() - PARTIAL_TEXT_MIN_INTERVAL);

        assert_eq!(
            limiter.emit_pending_if_ready().as_deref(),
            Some("第一段第二段")
        );
        assert!(limiter.pending_text.is_none());
    }

    #[test]
    fn partial_text_limiter_keeps_latest_fast_update_for_live_caption() {
        let mut limiter = PartialTextLimiter::new();

        assert_eq!(limiter.emit_or_defer("第一段").as_deref(), Some("第一段"));
        assert!(limiter.emit_or_defer("第一段第二段").is_none());
        assert!(limiter.emit_or_defer("第一段第二段第三段").is_none());
        limiter.last_emit_at = Some(Instant::now() - PARTIAL_TEXT_MIN_INTERVAL);

        assert_eq!(
            limiter.emit_pending_if_ready().as_deref(),
            Some("第一段第二段第三段")
        );
    }
}
