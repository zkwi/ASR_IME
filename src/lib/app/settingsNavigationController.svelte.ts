import type { Section } from "$lib/types/app";
import { settingsPanelForField } from "$lib/utils/settingsFields";
import { sectionForSettingsPanel as getSectionForSettingsPanel } from "$lib/utils/appRouting";
import type { ConfigValidationError } from "$lib/types/app";
import { firstValidationField } from "$lib/utils/config";

type SettingsNavigationControllerOptions = {
  isBrowser: () => boolean;
  requiresAsrAuth: () => boolean;
};

export function createSettingsNavigationController(options: SettingsNavigationControllerOptions) {
  let selectedSection = $state<Section>("Home");

  function scrollToSettingsPanel(targetId: string) {
    if (!options.isBrowser()) return;
    selectedSection = getSectionForSettingsPanel(targetId);
    window.setTimeout(() => {
      document.getElementById(targetId)?.scrollIntoView({ block: "start", behavior: "smooth" });
    }, 50);
  }

  function scrollContentTop() {
    if (!options.isBrowser()) return;
    window.setTimeout(() => {
      document.querySelector<HTMLElement>(".content")?.scrollTo({ top: 0, behavior: "smooth" });
    }, 50);
  }

  function focusFirstValidationError(errors: ConfigValidationError[]) {
    const field = firstValidationField(errors);
    if (!field) return;
    scrollToSettingsPanel(settingsPanelForField(field));
  }

  function focusAsrAuthSettings() {
    scrollToSettingsPanel("settings-auth");
  }

  function showApiConfigIntro() {
    selectedSection = "ApiConfig";
    scrollContentTop();
  }

  function openLlmApiSettings() {
    scrollToSettingsPanel("settings-llm-api");
  }

  function selectSection(section: Section) {
    selectedSection = section;
    if (section === "ApiConfig" && options.requiresAsrAuth()) scrollContentTop();
  }

  return {
    get selectedSection() { return selectedSection; },
    set selectedSection(value: Section) { selectedSection = value; },
    scrollToSettingsPanel,
    focusFirstValidationError,
    focusAsrAuthSettings,
    showApiConfigIntro,
    openLlmApiSettings,
    selectSection,
  };
}
