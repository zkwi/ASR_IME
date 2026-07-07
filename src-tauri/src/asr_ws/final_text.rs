use super::errors::{ASR_CONNECTION_CLOSED_MESSAGE, ASR_FINAL_TIMEOUT_MESSAGE};
use crate::asr;
use std::time::Duration;

// 收到最终包后短暂 settle，给二遍修正一次补尾机会；不回退到直接接受中间结果。
const FINAL_PACKET_SETTLE: Duration = Duration::from_millis(300);

pub(super) fn upsert_definite_segment(
    segments: &mut Vec<asr::DefiniteSegment>,
    segment: asr::DefiniteSegment,
) -> bool {
    if let Some(existing) = segments
        .iter_mut()
        .find(|item| item.start_time == segment.start_time && item.end_time == segment.end_time)
    {
        let changed = existing.text != segment.text;
        *existing = segment;
        return changed;
    }

    segments.push(segment);
    true
}

pub(super) fn should_finish_final_packet_settle(elapsed: Option<Duration>) -> bool {
    elapsed
        .map(|elapsed| elapsed >= FINAL_PACKET_SETTLE)
        .unwrap_or(false)
}

pub(super) fn should_timeout_waiting_final(
    final_packet_received: bool,
    elapsed: Option<Duration>,
    timeout: Duration,
) -> bool {
    !final_packet_received && elapsed.map(|elapsed| elapsed >= timeout).unwrap_or(false)
}

pub(super) fn select_final_output_text(
    definitive_segments: &[asr::DefiniteSegment],
    final_packet_text: Option<&str>,
    _live_caption_text: &str,
    remove_trailing_period: bool,
) -> Result<String, String> {
    let Some(text) = final_packet_text else {
        return Err(ASR_FINAL_TIMEOUT_MESSAGE.to_string());
    };

    let final_text = asr::normalize_final_text(text, remove_trailing_period);
    if !definitive_segments.is_empty() {
        let mut segments = definitive_segments.to_vec();
        segments.sort_by_key(|item| (item.start_time, item.end_time));
        let text = segments
            .into_iter()
            .map(|item| item.text)
            .collect::<Vec<_>>()
            .join("");
        let definitive_text = asr::normalize_final_text(&text, remove_trailing_period);
        if final_text_matches_definitive_text(&final_text, &definitive_text) {
            return Ok(final_text);
        }
        return Ok(definitive_text);
    }

    Ok(final_text)
}

pub(super) fn missing_final_result_error(connection_closed_before_final: bool) -> String {
    if connection_closed_before_final {
        ASR_CONNECTION_CLOSED_MESSAGE.to_string()
    } else {
        ASR_FINAL_TIMEOUT_MESSAGE.to_string()
    }
}

// 最终输出仍必须来自豆包最终包；definite 分句用于稳定性，但不能压掉更完整的最终包。
// 0.1.102 的回归经验：final 包补齐尾字时，前文可能相对 definite 分句有小幅改写。
// 因此先接受严格包含，再用高重合度兜底；明显残缺或不相关的 final 包仍会被拒绝。
// 改这里时同步 docs/asr-quality-latency-guardrails.md，并先补 final_output_ 回归测试。
fn final_text_matches_definitive_text(final_text: &str, definitive_text: &str) -> bool {
    let compact_final = compact_for_final_prefix(final_text);
    let compact_definitive = compact_for_final_prefix(definitive_text);
    if compact_final.is_empty() || compact_definitive.is_empty() {
        return false;
    }

    let final_len = compact_final.chars().count();
    let definitive_len = compact_definitive.chars().count();
    if compact_final.contains(&compact_definitive) {
        return true;
    }

    let overlap = common_subsequence_len(&compact_final, &compact_definitive);
    if final_len > definitive_len {
        // 豆包 final 包可能补尾字的同时轻微改写前文；高重合且更长时应信任 final 包。
        return overlap * 100 >= definitive_len * 75;
    }

    // final 包有时会把前文改短但补齐尾字。只接受长度接近且高度重合的结果，
    // 避免把“完整”这类残缺 final 包覆盖到更完整的 definite 分句上。
    final_len * 100 >= definitive_len * 80 && overlap * 100 >= final_len * 85
}

fn common_subsequence_len(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    if right_chars.is_empty() {
        return 0;
    }
    let mut dp = vec![0; right_chars.len() + 1];
    for left_char in left.chars() {
        let mut previous = 0;
        for (index, right_char) in right_chars.iter().enumerate() {
            let saved = dp[index + 1];
            dp[index + 1] = if left_char == *right_char {
                previous + 1
            } else {
                dp[index + 1].max(dp[index])
            };
            previous = saved;
        }
    }
    dp[right_chars.len()]
}

fn compact_for_final_prefix(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace() && !ch.is_ascii_punctuation() && !is_cjk_punctuation(*ch))
        .collect()
}

fn is_cjk_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '。' | '，'
            | '、'
            | '；'
            | '：'
            | '？'
            | '！'
            | '“'
            | '”'
            | '‘'
            | '’'
            | '（'
            | '）'
            | '《'
            | '》'
            | '【'
            | '】'
    )
}

#[cfg(test)]
mod tests {
    use super::{
        missing_final_result_error, select_final_output_text, should_finish_final_packet_settle,
        should_timeout_waiting_final, upsert_definite_segment,
    };
    use crate::asr::DefiniteSegment;
    use crate::asr_ws::errors::{ASR_CONNECTION_CLOSED_MESSAGE, ASR_FINAL_TIMEOUT_MESSAGE};
    use std::time::Duration;

    #[test]
    fn final_output_uses_definite_segments() {
        let segments = vec![DefiniteSegment {
            text: "二遍完整结果。".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text =
            select_final_output_text(&segments, Some("初步结果。"), "初步结果。", true).unwrap();

        assert_eq!(text, "二遍完整结果");
    }

    #[test]
    fn final_output_uses_last_package_text_when_it_extends_definite_segments() {
        let segments = vec![DefiniteSegment {
            text: "按下结束键之后，最后说话".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("按下结束键之后，最后说话还是没有被识别下来。"),
            "按下结束键之后，最后说话还是没有被识别下来。",
            true,
        )
        .unwrap();

        assert_eq!(text, "按下结束键之后，最后说话还是没有被识别下来");
    }

    #[test]
    fn final_output_uses_last_package_text_when_it_adds_missing_head() {
        let segments = vec![DefiniteSegment {
            text: "头两个字可能被截断".to_string(),
            start_time: 200,
            end_time: 1200,
        }];

        let text = select_final_output_text(
            &segments,
            Some("开头的头两个字可能被截断。"),
            "开头的头两个字可能被截断。",
            true,
        )
        .unwrap();

        assert_eq!(text, "开头的头两个字可能被截断");
    }

    #[test]
    fn final_output_uses_last_package_text_when_punctuation_differs() {
        let segments = vec![DefiniteSegment {
            text: "按下结束键之后，最后说话".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("按下结束键之后最后说话还是没有被识别下来。"),
            "按下结束键之后最后说话还是没有被识别下来。",
            true,
        )
        .unwrap();

        assert_eq!(text, "按下结束键之后最后说话还是没有被识别下来");
    }

    #[test]
    fn final_output_uses_last_package_text_when_tail_is_added_after_minor_rewrite() {
        let segments = vec![DefiniteSegment {
            text: "我觉得这个配置可以".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("我觉得这项配置可以生效。"),
            "我觉得这项配置可以生效。",
            true,
        )
        .unwrap();

        assert_eq!(text, "我觉得这项配置可以生效");
    }

    #[test]
    fn final_output_uses_last_package_text_when_tail_is_added_after_shorter_rewrite() {
        let segments = vec![DefiniteSegment {
            text: "最后一个字没有识别到".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("最后字没有识别到了。"),
            "最后字没有识别到了。",
            true,
        )
        .unwrap();

        assert_eq!(text, "最后字没有识别到了");
    }

    #[test]
    fn final_output_keeps_definite_segments_when_last_package_is_too_short() {
        let segments = vec![DefiniteSegment {
            text: "完整的二遍分句还有后半段".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(&segments, Some("完整"), "完整", false).unwrap();

        assert_eq!(text, "完整的二遍分句还有后半段");
    }

    #[test]
    fn final_output_keeps_definite_segments_when_last_package_does_not_cover_them() {
        let segments = vec![DefiniteSegment {
            text: "完整的二遍分句".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(&segments, Some("不相关最终包"), "不相关最终包", false)
            .unwrap();

        assert_eq!(text, "完整的二遍分句");
    }

    #[test]
    fn final_output_keeps_definite_segments_when_longer_last_package_is_unrelated() {
        let segments = vec![DefiniteSegment {
            text: "完整的二遍分句".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let text = select_final_output_text(
            &segments,
            Some("这是一个完全不相关但是更长的最终包"),
            "这是一个完全不相关但是更长的最终包",
            false,
        )
        .unwrap();

        assert_eq!(text, "完整的二遍分句");
    }

    #[test]
    fn definite_segment_update_replaces_earlier_text_for_same_time_range() {
        let mut segments = vec![DefiniteSegment {
            text: "早期结果".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        assert!(upsert_definite_segment(
            &mut segments,
            DefiniteSegment {
                text: "最终修正结果".to_string(),
                start_time: 0,
                end_time: 1000,
            },
        ));

        let text = select_final_output_text(&segments, Some(""), "", false).unwrap();
        assert_eq!(text, "最终修正结果");
    }

    #[test]
    fn final_output_rejects_definite_segments_without_last_package() {
        let segments = vec![DefiniteSegment {
            text: "缺少尾部的二遍结果".to_string(),
            start_time: 0,
            end_time: 1000,
        }];

        let err = select_final_output_text(&segments, None, "实时字幕", false).unwrap_err();

        assert!(err.contains("最终结果"));
    }

    #[test]
    fn final_output_rejects_interim_text_without_last_package() {
        let err = select_final_output_text(&[], None, "初步结果", false).unwrap_err();

        assert!(err.contains("最终结果"));
    }

    #[test]
    fn final_output_uses_last_package_text_without_definite_segments() {
        let text = select_final_output_text(&[], Some("最终结果"), "最终结果", false).unwrap();

        assert_eq!(text, "最终结果");
    }

    #[test]
    fn final_packet_settle_waits_before_finishing() {
        assert!(!should_finish_final_packet_settle(Some(
            Duration::from_millis(250)
        )));
        assert!(should_finish_final_packet_settle(Some(
            Duration::from_millis(300)
        )));
        assert!(!should_finish_final_packet_settle(None));
    }

    #[test]
    fn final_timeout_only_applies_before_last_package_arrives() {
        assert!(should_timeout_waiting_final(
            false,
            Some(Duration::from_millis(600)),
            Duration::from_millis(500),
        ));
        assert!(!should_timeout_waiting_final(
            true,
            Some(Duration::from_millis(600)),
            Duration::from_millis(500),
        ));
        assert!(!should_timeout_waiting_final(
            false,
            None,
            Duration::from_millis(500),
        ));
    }

    #[test]
    fn empty_last_package_stays_empty_for_failure_flow() {
        let text = select_final_output_text(&[], Some("   "), "初步结果", false).unwrap();

        assert_eq!(text, "");
    }

    #[test]
    fn missing_final_result_error_keeps_distinct_messages() {
        assert_eq!(
            missing_final_result_error(false),
            ASR_FINAL_TIMEOUT_MESSAGE.to_string()
        );
        assert_eq!(
            missing_final_result_error(true),
            ASR_CONNECTION_CLOSED_MESSAGE.to_string()
        );
    }
}
