import type { AppConfig, AppSnapshot, StatsSnapshot, UsageStats } from "$lib/types/app";
import type { CopyKey } from "$lib/i18n";

export const defaultLlmSystemPrompt = `你是语音输入助手。

场景：用户通过语音输入文字，语音识别（ASR）将语音转为文本后交给你处理。
你的输出将直接粘贴到用户的光标位置。永远只输出处理后的文本，不要与用户对话。如果无需处理，原样输出。

任务：
1. 修正明显的语音识别错误
2. 在不改变原意的前提下，对必要文本进行轻度润色、轻度改写或重写，使表达更清晰自然
3. 删除无意义的口头语、语气词和明显重复
4. 当文本较长、层次较多或明显属于口述长句时，可以主动分段、分行、分点整理，让结构更清晰
5. 不要扩写，不要新增事实，不要改变用户立场和语气，不要编造任何内容
6. 保留专有名词、数字、百分比、金融和编程术语
7. 如果原文本身已经简洁清楚，就尽量少改
8. 自动去掉结尾的句号
9. 最终只返回处理后的文本，不要输出任何解释、标题或多余内容`;

export const defaultLlmUserPromptTemplate = `以下是用户通过语音转写得到的文本，请按要求直接输出处理后的最终文本：

{text}

处理要求：
- 如果文本较短且表达清楚，尽量少改
- 如果文本较长、信息点较多、层次较乱，优先进行结构化整理，可按语义分段、分行、分点
- 如果存在明显识别错误、口头语、重复、语序混乱，可做必要的轻度改写，使其更清晰自然
- 不要输出解释，不要输出标题，不要输出任何额外说明`;

export function emptyUsage(): UsageStats {
  return {
    session_count: 0,
    total_seconds: 0,
    total_chars: 0,
    total_minutes_int: 0,
    avg_chars_per_minute: 0,
  };
}

export const fallbackConfig: AppConfig = {
  hotkey: "ctrl+q",
  auth: { app_key: "", access_key: "", resource_id: "volc.seedasr.sauc.duration" },
  audio: {
    sample_rate: 16000,
    channels: 1,
    segment_ms: 200,
    max_record_seconds: 300,
    stop_grace_ms: 500,
    silence_auto_stop_seconds: 30,
    silence_level_threshold: 0.03,
    mute_system_volume_while_recording: false,
    input_device: null,
  },
  request: {
    ws_url: "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async",
    model_name: "bigmodel",
    enable_nonstream: true,
    enable_itn: true,
    enable_punc: true,
    enable_ddc: true,
    show_utterances: true,
    result_type: "full",
    enable_accelerate_text: false,
    accelerate_score: 0,
    end_window_size: 800,
    force_to_speech_time: null,
    final_result_timeout_seconds: 15,
  },
  context: {
    enable_recent_context: false,
    recent_context_rounds: 5,
    hotwords: [],
    prompt_context: [],
    recent_context: [],
  },
  triggers: { hotkey_enabled: true, middle_mouse_enabled: false, right_alt_enabled: false },
  typing: {
    paste_delay_ms: 120,
    paste_method: "ctrl_v",
    remove_trailing_period: true,
    restore_clipboard_after_paste: true,
    clipboard_restore_delay_ms: 800,
    clipboard_snapshot_max_bytes: 8 * 1024 * 1024,
    clipboard_open_retry_count: 5,
    clipboard_open_retry_interval_ms: 50,
  },
  startup: { launch_on_startup: false },
  update: { auto_check_on_startup: true, github_repo: "zkwi/VoxType" },
  auto_hotwords: {
    enabled: false,
    accepted_hotwords: [],
    max_history_chars: 5000,
    max_candidates: 30,
    ignored_hotwords: [],
  },
  llm_post_edit: {
    enabled: false,
    min_chars: 40,
    base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    api_key: "",
    model: "qwen3.5-plus",
    timeout_seconds: 30,
    enable_thinking: false,
    system_prompt: defaultLlmSystemPrompt,
    user_prompt_template: defaultLlmUserPromptTemplate,
  },
  ui: {
    width: 350,
    height: 64,
    margin_bottom: 52,
    opacity: 0.9,
    scroll_interval_ms: 1200,
    background_color: "#176ee6",
    text_color: "#ffffff",
  },
  tray: {
    show_startup_message: true,
    startup_message_timeout_ms: 6000,
    close_behavior: "close_to_tray",
    close_to_tray_notice_shown: false,
  },
  debug: { print_transcript_to_console: false },
};

export const fallbackSnapshot: AppSnapshot = {
  hotkey: "ctrl+q",
  current_version: "0.1.16",
};

export const emptyStats: StatsSnapshot = {
  path: "voice_input_stats.jsonl",
  recent_24h: emptyUsage(),
  recent_7d: emptyUsage(),
  by_day: [],
  history: [],
};

export const defaultOverlayText = "正在录音...";
export const overlayLineHeight = 1.18;
export const chineseTypingCharsPerMinute = 50;
export const micBars = [0, 1, 2, 3, 4, 5];
export const overlayMeterBars = [0, 1, 2, 3];
export const overlayColorPresets: { label: CopyKey; background: string; text: string }[] = [
  { label: "overlayPresetBlue", background: "#176ee6", text: "#ffffff" },
  { label: "overlayPresetDark", background: "#111827", text: "#f8fafc" },
  { label: "overlayPresetLight", background: "#f8fafc", text: "#111827" },
  { label: "overlayPresetAmber", background: "#92400e", text: "#fff7ed" },
];
export const overlayOpacityPresets = [0.6, 0.75, 0.9, 1] as const;
export const setupStatusCacheKey = "voxtype-setup-status-v1";
export const autoSaveDelayMs = 700;
