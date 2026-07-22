type FocusTrapOptions = {
  initialFocus?: string;
  onEscape?: () => void;
};

const focusableSelector = [
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "[tabindex]:not([tabindex='-1'])",
].join(",");

export function wrappedFocusIndex(current: number, direction: -1 | 1, count: number) {
  if (count <= 1) return 0;
  return (current + direction + count) % count;
}

export function focusTrap(node: HTMLElement, options: FocusTrapOptions = {}) {
  const returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;

  function focusableElements() {
    return Array.from(node.querySelectorAll<HTMLElement>(focusableSelector)).filter(
      (element) => !element.hasAttribute("hidden") && element.getAttribute("aria-hidden") !== "true",
    );
  }

  function focusInitial() {
    const preferred = options.initialFocus
      ? node.querySelector<HTMLElement>(options.initialFocus)
      : null;
    (preferred ?? focusableElements()[0] ?? node).focus();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && options.onEscape) {
      event.preventDefault();
      event.stopPropagation();
      options.onEscape();
      return;
    }
    if (event.key !== "Tab") return;
    const elements = focusableElements();
    if (elements.length === 0) {
      event.preventDefault();
      node.focus();
      return;
    }
    const currentIndex = elements.indexOf(document.activeElement as HTMLElement);
    if (currentIndex === -1) {
      event.preventDefault();
      elements[event.shiftKey ? elements.length - 1 : 0].focus();
      return;
    }
    const nextIndex = wrappedFocusIndex(currentIndex, event.shiftKey ? -1 : 1, elements.length);
    if (nextIndex === currentIndex + (event.shiftKey ? -1 : 1)) return;
    event.preventDefault();
    elements[nextIndex].focus();
  }

  node.addEventListener("keydown", handleKeydown);
  window.requestAnimationFrame(focusInitial);

  return {
    destroy() {
      node.removeEventListener("keydown", handleKeydown);
      if (returnFocus?.isConnected) window.requestAnimationFrame(() => returnFocus.focus());
    },
  };
}
