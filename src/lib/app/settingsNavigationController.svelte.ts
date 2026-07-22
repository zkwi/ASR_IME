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

  function afterSectionRender(callback: () => void) {
    window.requestAnimationFrame(() => window.requestAnimationFrame(callback));
  }

  function contentScroller() {
    return document.querySelector<HTMLElement>(".content");
  }

  function settingsStickyOffset(scroller: HTMLElement) {
    const jumpNav = scroller.querySelector<HTMLElement>(".settings-jump-nav");
    return jumpNav ? jumpNav.offsetHeight + 18 : 18;
  }

  function scrollToSettingsPanel(targetId: string) {
    if (!options.isBrowser()) return;
    selectedSection = getSectionForSettingsPanel(targetId);
    afterSectionRender(() => {
      const scroller = contentScroller();
      const target = document.getElementById(targetId);
      if (!scroller || !target) return;

      const scrollerRect = scroller.getBoundingClientRect();
      const targetRect = target.getBoundingClientRect();
      const targetTop = targetRect.top - scrollerRect.top + scroller.scrollTop;
      scroller.scrollTo({
        top: Math.max(0, targetTop - settingsStickyOffset(scroller)),
        behavior: "smooth",
      });
    });
  }

  function scrollContentTop() {
    if (!options.isBrowser()) return;
    afterSectionRender(() => {
      contentScroller()?.scrollTo({ top: 0, behavior: "smooth" });
    });
  }

  function focusFirstValidationError(errors: ConfigValidationError[]) {
    const field = firstValidationField(errors);
    if (!field) return;
    scrollToSettingsPanel(settingsPanelForField(field));
    afterSectionRender(() => {
      const control = document.querySelector<HTMLElement>(`[data-config-field="${CSS.escape(field)}"]`);
      control?.focus();
    });
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
    const changed = selectedSection !== section;
    selectedSection = section;
    if (changed || (section === "ApiConfig" && options.requiresAsrAuth())) scrollContentTop();
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
