<style>
  :global(:root) {
    --blue-50: #eef8ff;
    --blue-100: #d9efff;
    --blue-200: #c7e6fb;
    --blue-500: #2f8cff;
    --blue-600: #176ee6;
    --blue-700: #1158bb;
    --ink: #17202b;
    --muted: #6f8194;
    --line: #e2eaf3;
    --panel: #ffffff;
    --canvas: #f6f8fb;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(html),
  :global(body) {
    margin: 0;
    min-width: 320px;
    width: 100%;
    height: 100%;
    overflow: hidden;
    color: var(--ink);
    background: #edf3fa;
    font-family: "Segoe UI", "Microsoft YaHei UI", "Microsoft YaHei", sans-serif;
    font-size: 16px;
    letter-spacing: 0;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
  }

  :global(html[data-voxtype-mode="overlay"]),
  :global(body[data-voxtype-mode="overlay"]),
  :global(html:has(.overlay-root)),
  :global(body:has(.overlay-root)) {
    min-width: 0;
    width: 100vw;
    height: 100vh;
    overflow: hidden !important;
    background: transparent;
  }

  :global(html[data-voxtype-mode="toast"]),
  :global(body[data-voxtype-mode="toast"]),
  :global(html:has(.toast-root)),
  :global(body:has(.toast-root)) {
    min-width: 0;
    width: 100vw;
    height: 100vh;
    overflow: hidden !important;
    background: transparent;
    font-size: 14px;
  }

  :global(body[data-voxtype-mode="overlay"]::-webkit-scrollbar),
  :global(body[data-voxtype-mode="overlay"] *::-webkit-scrollbar),
  :global(body:has(.overlay-root)::-webkit-scrollbar),
  :global(body:has(.overlay-root) *::-webkit-scrollbar),
  :global(body[data-voxtype-mode="toast"]::-webkit-scrollbar),
  :global(body[data-voxtype-mode="toast"] *::-webkit-scrollbar),
  :global(body:has(.toast-root)::-webkit-scrollbar),
  :global(body:has(.toast-root) *::-webkit-scrollbar) {
    width: 0;
    height: 0;
    display: none;
  }

  :global(button),
  :global(input),
  :global(textarea),
  :global(select) {
    font: inherit;
    transition: all 160ms ease;
  }

  :global(button) {
    cursor: pointer;
    border: 0;
    background: transparent;
  }

  :global(:root) {
    --primary: #2f80ed;
    --primary-hover: #256fe0;
    --primary-light: #eaf3ff;
    --gradient-start: #2f80ed;
    --gradient-end: #4f6ff0;
    --text-main: #0f172a;
    --text-secondary: #5f7188;
    --text-muted: #8ea0b5;
    --bg-page: #f7fafe;
    --bg-sidebar: #eef5fc;
    --bg-card: #ffffff;
    --border: #dbe6f2;
    --border-strong: #c4d2e3;
    --success: #10b981;
    --warning: #f59e0b;
    --danger: #ef4444;
    --radius-sm: 8px;
    --radius-md: 12px;
    --radius-lg: 16px;
    --radius-xl: 24px;
    --shadow-card: 0 10px 26px rgba(15, 23, 42, 0.045);
    --shadow-soft: 0 4px 12px rgba(15, 23, 42, 0.055);
  }

  :global(html[data-voxtype-mode="main"]),
  :global(body[data-voxtype-mode="main"]),
  :global(html:not([data-voxtype-mode]):not(:has(.overlay-root)):not(:has(.toast-root))),
  :global(body:not([data-voxtype-mode]):not(:has(.overlay-root)):not(:has(.toast-root))) {
    background: var(--bg-page);
    font-family: "Segoe UI", "Microsoft YaHei UI", "Microsoft YaHei", "PingFang SC", "SF Pro Display", "Noto Sans CJK SC", sans-serif;
  }

  :global(.settings-stack),
  :global(.privacy-stack),
  :global(.history-page) {
    display: grid;
    gap: 16px;
    max-width: 1040px;
  }

  :global(.privacy-stack),
  :global(.history-page) {
    max-width: 1120px;
  }

  :global(.settings-group) {
    display: grid;
    gap: 12px;
  }

  :global(.settings-group-heading),
  :global(.section-heading) {
    display: grid;
    gap: 4px;
  }

  :global(.settings-group-heading) {
    padding: 0 2px;
  }

  :global(.settings-group-heading h3),
  :global(.section-heading h3),
  :global(.privacy-heading h3) {
    margin: 0;
    color: var(--text-main);
    font-weight: 800;
    line-height: 1.25;
  }

  :global(.settings-group-heading h3),
  :global(.privacy-heading h3) {
    font-size: 20px;
  }

  :global(.section-heading h3) {
    margin-bottom: 6px;
    font-size: 16px;
  }

  :global(.settings-group-heading p),
  :global(.section-heading p),
  :global(.privacy-heading p) {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.45;
  }

  :global(.form-panel) {
    display: grid;
    gap: 16px;
    min-width: 0;
    padding: 18px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 16px;
    box-shadow: none;
  }

  :global(.form-panel[id^="settings-"]) {
    scroll-margin-top: 86px;
  }

  :global(.form-panel label) {
    display: grid;
    align-content: start;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 14px;
  }

  :global(.section-heading.with-actions) {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  :global(.section-heading.with-actions > div),
  :global(.section-heading-copy) {
    min-width: 0;
  }

  :global(.section-heading-copy) {
    flex: 1 1 320px;
  }

  :global(.form-grid),
  :global(.toggle-grid) {
    display: grid;
    align-items: start;
    gap: 16px 14px;
  }

  :global(.form-grid) {
    grid-template-columns: repeat(auto-fit, minmax(min(320px, 100%), 1fr));
  }

  :global(.toggle-grid) {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 18px;
  }

  :global(.check) {
    display: inline-flex !important;
    align-items: center;
    flex: 0 1 auto;
    gap: 10px;
    min-width: 0;
    width: fit-content;
    max-width: 100%;
    min-height: 38px;
    color: var(--text-main);
    font-weight: 700;
    line-height: 1.35;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  :global(.check input) {
    flex: 0 0 auto;
    width: 18px;
    min-height: 18px;
    accent-color: var(--primary);
  }

  :global(.check-copy) {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  :global(.check-copy span) {
    color: var(--text-main);
    font-size: 14px;
    font-weight: 700;
    white-space: normal;
  }

  :global(.check-copy small) {
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 700;
    line-height: 1.25;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  :global(.settings-inline-actions) {
    display: flex;
    flex: 0 0 auto;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: flex-end;
    min-width: 0;
  }

  :global(.test-button),
  :global(.link-button) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    min-height: 36px;
    padding: 0 14px;
    color: var(--primary);
    background: var(--primary-light);
    border: 1px solid rgba(47, 128, 237, 0.18);
    border-radius: 12px;
    font-size: 13px;
    font-weight: 800;
    line-height: 1.2;
    white-space: normal;
    overflow-wrap: anywhere;
  }

  :global(.test-button) {
    flex: 0 0 auto;
    min-width: 96px;
  }

  :global(.link-button) {
    min-width: 0;
  }

  :global(.test-button:disabled),
  :global(.link-button:disabled) {
    cursor: wait;
    opacity: 0.68;
  }

  :global(.form-panel input:not([type="checkbox"]):not([type="radio"])),
  :global(.form-panel select),
  :global(.form-panel textarea) {
    width: 100%;
    min-width: 0;
    min-height: 38px;
    padding: 0 12px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 10px;
    text-overflow: ellipsis;
  }

  :global(.form-panel textarea) {
    min-height: 84px;
    padding: 10px 12px;
    resize: vertical;
  }

  :global(.form-panel input:not([type="checkbox"]):not([type="radio"]):focus),
  :global(.form-panel select:focus),
  :global(.form-panel textarea:focus),
  :global(.form-panel button:focus-visible) {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(47, 128, 237, 0.14);
  }

  :global(.field-hint) {
    margin: 8px 0 0;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.45;
  }

  :global(.field-invalid input:not([type="checkbox"]):not([type="radio"])),
  :global(.field-invalid select),
  :global(.field-invalid textarea) {
    border-color: var(--danger);
    background: #fff7f7;
  }

  :global(.field-error) {
    color: var(--danger);
    font-size: 12px;
    line-height: 1.35;
  }

  @media (max-width: 920px) {
    :global(.section-heading.with-actions) {
      display: grid;
      grid-template-columns: 1fr;
      align-items: stretch;
    }

    :global(.test-button) {
      width: 100%;
    }

    :global(.form-grid) {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    :global(.form-grid) {
      grid-template-columns: 1fr;
    }

    :global(.toggle-grid) {
      display: grid;
      grid-template-columns: 1fr;
    }

    :global(.check) {
      width: 100%;
    }
  }

  @media (max-width: 620px) {
    :global(.form-panel) {
      padding: 14px;
    }
  }
</style>
