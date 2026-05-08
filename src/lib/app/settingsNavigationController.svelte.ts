import type { Section } from "$lib/types/app";
import {
  fieldAdvancedSection,
  fieldRequiresAdvancedSettings,
  settingsPanelForField,
  type AdvancedSection,
} from "$lib/utils/settingsFields";
import { sectionForSettingsPanel as getSectionForSettingsPanel } from "$lib/utils/appRouting";
import type { ConfigValidationError } from "$lib/types/app";
import { firstValidationField } from "$lib/utils/config";

type SettingsNavigationControllerOptions = {
  isBrowser: () => boolean;
  requiresAsrAuth: () => boolean;
};

export function createSettingsNavigationController(options: SettingsNavigationControllerOptions) {
  let selectedSection = $state<Section>("Home");
  let llmApiConfigVisible = $state(false);
  let advancedVisible = $state<Record<AdvancedSection, boolean>>({
    Hotwords: true,
    ApiConfig: true,
    Options: true,
  });

  function isAdvancedVisible(section: AdvancedSection) {
    return advancedVisible[section];
  }

  function toggleAdvanced(section: AdvancedSection) {
    advancedVisible = {
      ...advancedVisible,
      [section]: !advancedVisible[section],
    };
  }

  function showAdvanced(section: AdvancedSection) {
    advancedVisible = {
      ...advancedVisible,
      [section]: true,
    };
  }

  function scrollToSettingsPanel(targetId: string) {
    if (!options.isBrowser()) return;
    selectedSection = getSectionForSettingsPanel(targetId);
    if (targetId === "settings-llm-api") llmApiConfigVisible = true;
    window.setTimeout(() => {
      document.getElementById(targetId)?.scrollIntoView({ block: "start", behavior: "smooth" });
    }, 50);
  }

  function focusFirstValidationError(errors: ConfigValidationError[]) {
    const field = firstValidationField(errors);
    if (!field) return;
    if (fieldRequiresAdvancedSettings(field)) showAdvanced(fieldAdvancedSection(field));
    if (field.startsWith("llm_post_edit.")) llmApiConfigVisible = true;
    scrollToSettingsPanel(settingsPanelForField(field));
  }

  function focusAsrAuthSettings() {
    scrollToSettingsPanel("settings-auth");
  }

  function openLlmApiSettings() {
    llmApiConfigVisible = true;
    scrollToSettingsPanel("settings-llm-api");
  }

  function selectSection(section: Section) {
    selectedSection = section;
    if (section === "ApiConfig" && options.requiresAsrAuth()) scrollToSettingsPanel("settings-auth");
  }

  return {
    get selectedSection() { return selectedSection; },
    set selectedSection(value: Section) { selectedSection = value; },
    get llmApiConfigVisible() { return llmApiConfigVisible; },
    set llmApiConfigVisible(value: boolean) { llmApiConfigVisible = value; },
    isAdvancedVisible,
    toggleAdvanced,
    showAdvanced,
    scrollToSettingsPanel,
    focusFirstValidationError,
    focusAsrAuthSettings,
    openLlmApiSettings,
    selectSection,
  };
}
