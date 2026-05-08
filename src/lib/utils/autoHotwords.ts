import type { CopyKey } from "$lib/i18n";
import type { AppConfig, AutoHotwordStatus, SelectableHotwordCandidate } from "$lib/types/app";
import { dedupeHotwords } from "$lib/utils/hotwords";

type Translate = (key: CopyKey, values?: Record<string, string>) => string;

export function showAutoHotwordDetails(config: AppConfig, fieldError: (field: string) => string) {
  return (
    config.auto_hotwords.enabled ||
    config.auto_hotwords.accepted_hotwords.length > 0 ||
    Boolean(fieldError("auto_hotwords.max_history_chars") || fieldError("auto_hotwords.max_candidates"))
  );
}

export function autoHotwordStatusText(status: AutoHotwordStatus | null, t: Translate) {
  if (!status) return t("autoHotwordsStatusUnknown");
  return t("autoHotwordsStatus", {
    entries: String(status.entry_count),
    chars: String(status.total_chars),
    max: String(status.max_history_chars),
  });
}

export function mapGeneratedHotwordCandidates(candidates: SelectableHotwordCandidate[] | Array<Omit<SelectableHotwordCandidate, "selected">>) {
  return candidates.map((item) => ({
    ...item,
    selected: true,
  }));
}

export function buildFinalPromptPreview(
  config: AppConfig,
  sampleText: string,
  hotwords: string[],
  labels: {
    dictionary: string;
    context: string;
    systemPrompt: string;
    userPromptTemplate: string;
    empty: string;
    summaryTitle: string;
    sceneContextSummary: string;
    recentContextPolicy: string;
    actualPromptTitle: string;
  },
) {
  let userPrompt = config.llm_post_edit.user_prompt_template.replace("{text}", sampleText);
  if (hotwords.length > 0) {
    userPrompt += `\n\n${labels.dictionary}\n${hotwords.join("\n")}`;
  }
  const promptContext = config.context.prompt_context.map((item) => item.text.trim()).filter(Boolean);
  const promptContextText = promptContext.length > 0 ? promptContext.map((item) => `- ${item}`).join("\n") : labels.empty;
  if (promptContext.length > 0) {
    userPrompt += `\n\n${labels.context}\n${promptContextText}`;
  }
  const summary = `${labels.summaryTitle}\n${labels.sceneContextSummary}\n${promptContextText}\n\n${labels.recentContextPolicy}`;
  const actualPrompt = `${labels.actualPromptTitle}\n\n${labels.systemPrompt}\n${config.llm_post_edit.system_prompt || labels.empty}\n\n${labels.userPromptTemplate}\n${userPrompt}`;
  return `${summary}\n\n${actualPrompt}`;
}

export function applySelectedAutoHotwords(
  config: AppConfig,
  candidates: SelectableHotwordCandidate[],
  existingEffectiveHotwords: string[],
) {
  const selected = candidates
    .filter((candidate) => candidate.selected)
    .map((candidate) => candidate.word.trim())
    .filter(Boolean);
  const merged = [...config.auto_hotwords.accepted_hotwords];
  const seen = new Set(existingEffectiveHotwords.map((item) => item.trim().toLocaleLowerCase()).filter(Boolean));
  let added = 0;
  for (const word of selected) {
    const key = word.toLocaleLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    merged.push(word);
    added += 1;
  }
  return {
    added,
    acceptedHotwords: dedupeHotwords(merged),
  };
}
