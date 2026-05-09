<script lang="ts">
  import type { Snippet } from "svelte";
  import type { CopyKey, Language } from "$lib/i18n";
  import type { Section } from "$lib/types/app";
  import {
    BarChart3,
    Download,
    House,
    KeyRound,
    LockKeyhole,
    Maximize2,
    Mic,
    Minus,
    Settings,
    Sparkles,
    X as XIcon,
  } from "lucide-svelte";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;
  type MaybeAsync = void | Promise<void>;
  type ConfigSaveState = "idle" | "pending" | "saving" | "saved";

  type Props = {
    children?: Snippet;
    uiCompact: boolean;
    selectedSection: Section;
    language: Language;
    recording: boolean;
    configSaveState: ConfigSaveState;
    inputStatus: string;
    inputStatusLabel: string;
    inputStatusDesc: string;
    micBars: number[];
    snapshotHotkey: string;
    requiresAsrAuth: boolean;
    t: Translate;
    formatHotkey: (value: string) => string;
    micStatusText: () => string;
    sidebarMicStatusText: () => string;
    micBarHeight: (index: number) => string;
    micBarOpacity: (index: number) => string;
    onSelectSection: (section: Section) => void;
    onSetLanguage: (language: string) => void;
    onClose: () => MaybeAsync;
    onMinimize: () => MaybeAsync;
    onToggleMaximize: () => MaybeAsync;
  };

  let {
    children,
    uiCompact,
    selectedSection,
    language,
    recording,
    configSaveState,
    inputStatus,
    inputStatusLabel,
    inputStatusDesc,
    micBars,
    snapshotHotkey,
    requiresAsrAuth,
    t,
    formatHotkey,
    micStatusText,
    sidebarMicStatusText,
    micBarHeight,
    micBarOpacity,
    onSelectSection,
    onSetLanguage,
    onClose,
    onMinimize,
    onToggleMaximize,
  }: Props = $props();

  const navItems = [
    { id: "Home", icon: House },
    { id: "Hotwords", icon: Sparkles },
    { id: "ApiConfig", icon: KeyRound },
    { id: "Options", icon: Settings },
    { id: "Privacy", icon: LockKeyhole },
    { id: "History", icon: BarChart3 },
  ] as const;

  const navLabelKeys: Record<Section, CopyKey> = {
    Home: "navHome",
    Hotwords: "navHotwords",
    ApiConfig: "navApiConfig",
    Options: "navOptions",
    Privacy: "navPrivacy",
    History: "navHistory",
  };

  function configSaveStatusText() {
    if (configSaveState === "pending") return t("settingsSavePending");
    if (configSaveState === "saving") return t("settingsSaving");
    return t("settingsSaved");
  }
</script>

<div class:ui-compact={uiCompact} class="app-frame">
  <header class="window-titlebar" data-tauri-drag-region>
    <div class="window-title" data-tauri-drag-region>
      <span class="window-title-mark"><Mic size={12} strokeWidth={2.6} /></span>
      <strong data-tauri-drag-region>{t("appTitle")}</strong>
      <span class="window-product-name" data-tauri-drag-region>VoxType</span>
      {#if configSaveState !== "idle"}
        <span
          class:pending={configSaveState === "pending"}
          class:saved={configSaveState === "saved"}
          class:saving={configSaveState === "saving"}
          class="save-status"
          aria-live="polite"
          data-tauri-drag-region
        >
          <span class="save-dot" data-tauri-drag-region></span>
          <span class="save-text" data-tauri-drag-region>{configSaveStatusText()}</span>
        </span>
      {/if}
    </div>
    <div class="window-controls">
      <button class="tray-action" aria-label={t("minimizeToTray")} title={t("minimizeToTray")} onclick={onClose}>
        <Download size={15} />
        <span>{t("minimizeToTray")}</span>
      </button>
      <button aria-label={t("windowMinimize")} title={t("windowMinimize")} onclick={onMinimize}><Minus size={13} /></button>
      <button aria-label={t("windowMaximizeRestore")} title={t("windowMaximizeRestore")} onclick={onToggleMaximize}><Maximize2 size={12} /></button>
      <button class="close" aria-label={t("windowClose")} title={t("windowClose")} onclick={onClose}><XIcon size={14} /></button>
    </div>
  </header>

  <main class="shell">
    <aside class="sidebar">
      <nav aria-label={t("mainSections")}>
        {#each navItems as item}
          {@const Icon = item.icon}
          {@const label = t(navLabelKeys[item.id])}
          <button
            aria-current={selectedSection === item.id ? "page" : undefined}
            class:active={selectedSection === item.id}
            title={label}
            onclick={() => onSelectSection(item.id)}
          >
            <span class="nav-icon" aria-hidden="true"><Icon size={17} strokeWidth={2.25} /></span>
            <span class="nav-text">{label}</span>
          </button>
        {/each}
      </nav>

      <label class="language-control">
        <span>{t("language")}</span>
        <select value={language} onchange={(event) => onSetLanguage(event.currentTarget.value)}>
          <option value="zh-CN">简体中文</option>
          <option value="zh-TW">繁體中文</option>
          <option value="en">English</option>
        </select>
      </label>

      <section class:error={inputStatus === "error"} class:listening={recording} class="bridge-card">
        <div class="bridge-top">
          <span class="pulse" class:recording class:error={inputStatus === "error"}></span>
          <span>{inputStatusLabel}</span>
        </div>
        <p>{inputStatusDesc}</p>
        <div class:active={recording} class="mic-line">
          <span title={micStatusText()}>{sidebarMicStatusText()}</span>
          {#if recording}
            {#each micBars as bar}
              <i style:height={micBarHeight(bar)} style:opacity={micBarOpacity(bar)}></i>
            {/each}
          {/if}
        </div>
        <div class="shortcut-line">{t("sidebarShortcut", { hotkey: formatHotkey(snapshotHotkey) })}</div>
      </section>
    </aside>

    <section
      class:overview-content={selectedSection === "Home"}
      class:setup-required={requiresAsrAuth}
      class:session-error={inputStatus === "error"}
      class="content"
    >
      {#if selectedSection === "History"}
        <header class="topbar">
          <div>
            <h2>{t(navLabelKeys[selectedSection])}</h2>
          </div>
        </header>
      {/if}

      {@render children?.()}
    </section>
  </main>
</div>

<style>
  .app-frame {
    position: relative;
    display: grid;
    grid-template-rows: 48px minmax(0, 1fr);
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background: var(--bg-page);
  }

  .app-frame.ui-compact {
    grid-template-rows: 44px minmax(0, 1fr);
  }

  .window-titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 18px;
    background: #ffffff;
    border-bottom: 1px solid var(--border);
    box-shadow: 0 1px 0 rgba(15, 23, 42, 0.02);
    user-select: none;
    -webkit-app-region: drag;
  }

  .ui-compact .window-titlebar {
    height: 44px;
    padding: 0 16px;
  }

  .window-title {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    overflow: hidden;
    color: var(--text-main);
    font-size: 15px;
    font-weight: 400;
    text-transform: none;
  }

  .window-title strong {
    min-width: 0;
    overflow: hidden;
    font-size: 16px;
    font-weight: 700;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .window-product-name {
    min-width: 0;
    overflow: hidden;
    color: var(--text-secondary);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .save-status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    max-width: 150px;
    min-height: 24px;
    padding: 0 8px;
    flex: 0 1 auto;
    color: #245b93;
    background: rgba(240, 247, 255, 0.96);
    border: 1px solid rgba(47, 128, 237, 0.22);
    border-radius: 999px;
    font-size: 12px;
    font-weight: 800;
    line-height: 1;
  }

  .save-dot {
    width: 7px;
    height: 7px;
    flex: 0 0 7px;
    background: var(--primary);
    border-radius: 999px;
  }

  .save-text {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .save-status.pending {
    color: #854d0e;
    background: rgba(255, 251, 235, 0.96);
    border-color: rgba(245, 158, 11, 0.3);
  }

  .save-status.pending .save-dot {
    background: #f59e0b;
  }

  .save-status.saved {
    color: #047857;
    background: rgba(236, 253, 245, 0.96);
    border-color: rgba(16, 185, 129, 0.26);
  }

  .save-status.saved .save-dot {
    background: #10b981;
  }

  .save-status.saving .save-dot {
    animation: save-pulse 900ms ease-in-out infinite;
  }

  .window-title-mark {
    display: grid;
    width: 28px;
    height: 28px;
    flex: 0 0 auto;
    place-items: center;
    color: #ffffff;
    background: linear-gradient(135deg, var(--gradient-start), var(--gradient-end));
    border: 0;
    border-radius: 10px;
    box-shadow: 0 6px 16px rgba(47, 128, 237, 0.24);
  }

  .window-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 0 0 auto;
    -webkit-app-region: no-drag;
  }

  .window-controls button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid transparent;
    border-radius: 10px;
    transition: all 160ms ease;
    -webkit-app-region: no-drag;
  }

  .ui-compact .window-controls button {
    width: 30px;
    height: 30px;
  }

  .ui-compact .window-controls .tray-action {
    width: auto;
    padding: 0 10px;
    font-size: 13px;
  }

  .window-controls button:hover {
    color: var(--text-main);
    background: #f1f5f9;
    border-color: var(--border);
  }

  .window-controls .tray-action {
    display: inline-flex;
    width: auto;
    max-width: 170px;
    gap: 9px;
    padding: 0 12px;
    color: var(--text-secondary);
    background: #fbfdff;
    border-color: var(--border);
    font-size: 14px;
    font-weight: 500;
  }

  .window-controls .tray-action span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .window-controls button.close:hover {
    color: #ffffff;
    background: var(--danger);
    border-color: var(--danger);
  }

  .shell {
    display: grid;
    grid-template-columns: 230px minmax(0, 1fr);
    min-height: 0;
    overflow: hidden;
    background: var(--bg-page);
  }

  .ui-compact .shell {
    grid-template-columns: 212px minmax(0, 1fr);
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
    min-height: 0;
    padding: 18px 20px;
    overflow-x: hidden;
    overflow-y: auto;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    scrollbar-color: #cbd8e7 transparent;
    scrollbar-width: thin;
  }

  .sidebar::-webkit-scrollbar {
    width: 8px;
  }

  .sidebar::-webkit-scrollbar-thumb {
    background: #cbd8e7;
    border: 2px solid var(--bg-sidebar);
    border-radius: 999px;
  }

  .ui-compact .sidebar {
    gap: 11px;
    padding: 14px 16px;
  }

  nav {
    display: grid;
    gap: 7px;
  }

  .ui-compact nav {
    gap: 6px;
  }

  nav button {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    min-height: 42px;
    margin: 0;
    padding: 0 12px;
    gap: 10px;
    color: #334155;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    font-size: 15px;
    font-weight: 600;
    text-align: left;
    transition: color 160ms ease, background-color 160ms ease, border-color 160ms ease, box-shadow 160ms ease;
  }

  nav button::before {
    position: absolute;
    left: 4px;
    width: 3px;
    height: 18px;
    content: "";
    background: currentColor;
    border-radius: 999px;
    opacity: 0;
    transform: scaleY(0.65);
    transition: opacity 160ms ease, transform 160ms ease;
  }

  .nav-icon {
    display: grid;
    width: 28px;
    height: 28px;
    flex: 0 0 28px;
    place-items: center;
    color: #5b6f88;
    background: rgba(255, 255, 255, 0.62);
    border: 1px solid rgba(203, 216, 231, 0.7);
    border-radius: 9px;
    transition: color 160ms ease, background-color 160ms ease, border-color 160ms ease;
  }

  .nav-icon :global(svg) {
    width: 17px;
    height: 17px;
  }

  .nav-text {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ui-compact nav button {
    min-height: 38px;
    padding: 0 10px;
    gap: 9px;
    font-size: 14px;
  }

  .ui-compact .nav-icon {
    width: 26px;
    height: 26px;
    flex-basis: 26px;
  }

  nav button:hover {
    color: var(--primary);
    background: rgba(234, 243, 255, 0.78);
    border-color: rgba(47, 128, 237, 0.18);
  }

  nav button:hover .nav-icon {
    color: var(--primary);
    background: #ffffff;
    border-color: rgba(47, 128, 237, 0.24);
  }

  nav button:focus-visible {
    outline: 2px solid rgba(47, 128, 237, 0.32);
    outline-offset: 2px;
  }

  nav button.active {
    color: #ffffff;
    background: linear-gradient(135deg, var(--primary), var(--primary-hover));
    box-shadow: 0 10px 22px rgba(47, 128, 237, 0.22);
    font-weight: 800;
  }

  nav button.active::before {
    opacity: 0.9;
    transform: scaleY(1);
  }

  nav button.active .nav-icon {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.18);
    border-color: rgba(255, 255, 255, 0.24);
  }

  .language-control {
    display: grid;
    gap: 8px;
    margin: 6px 0 0;
  }

  .ui-compact .language-control {
    gap: 8px;
    margin-top: 4px;
  }

  .language-control span {
    color: #516b8a;
    font-size: 13px;
    font-weight: 700;
    text-transform: none;
  }

  .language-control select {
    width: 100%;
    min-height: 38px;
    padding: 0 12px;
    color: var(--text-main);
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 12px;
    font-size: 15px;
    box-shadow: 0 1px 0 rgba(15, 23, 42, 0.02);
    transition: border-color 160ms ease, box-shadow 160ms ease;
  }

  .language-control select:focus {
    border-color: rgba(47, 128, 237, 0.45);
    outline: 2px solid rgba(47, 128, 237, 0.16);
    outline-offset: 2px;
  }

  .ui-compact .language-control select {
    min-height: 34px;
    font-size: 14px;
  }

  .bridge-card {
    display: grid;
    gap: 7px;
    margin: auto 0 0;
    padding: 13px;
    min-width: 0;
    overflow: hidden;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.045);
  }

  .ui-compact .bridge-card {
    gap: 6px;
    padding: 10px;
  }

  .bridge-card.listening {
    border-color: rgba(47, 128, 237, 0.28);
  }

  .bridge-card.error {
    border-color: rgba(239, 68, 68, 0.28);
  }

  .bridge-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin: 0;
    min-width: 0;
    color: var(--text-main);
    font-size: 15px;
    font-weight: 800;
  }

  .bridge-top span:last-child {
    order: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .bridge-card p {
    margin: 0;
    min-width: 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.35;
    display: -webkit-box;
    overflow-wrap: anywhere;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .pulse {
    order: 2;
    width: 8px;
    height: 8px;
    flex: 0 0 8px;
    align-self: center;
    margin: 0 6px 0 auto;
    background: var(--success);
    border-radius: 999px;
  }

  .pulse.recording {
    background: var(--primary);
    box-shadow: 0 0 0 6px rgba(47, 128, 237, 0.14);
  }

  .pulse.error {
    background: var(--danger);
    box-shadow: 0 0 0 6px rgba(239, 68, 68, 0.12);
  }

  .mic-line {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    min-height: 28px;
    padding-top: 8px;
    color: var(--text-secondary);
    border-top: 1px solid var(--border);
    font-size: 12px;
  }

  .shortcut-line {
    min-width: 0;
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.35;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mic-line span {
    margin-right: auto;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mic-line i {
    display: block;
    width: 4px;
    background: var(--success);
    border-radius: 999px;
    transform-origin: bottom center;
    transition: height 90ms ease, opacity 90ms ease, background-color 160ms ease;
  }

  .mic-line.active i {
    background: var(--primary);
  }

  .content {
    min-width: 0;
    min-height: 0;
    padding: 16px 20px;
    overflow: auto;
    overflow-x: hidden;
    background: var(--bg-page);
  }

  .content::-webkit-scrollbar {
    width: 10px;
  }

  .content::-webkit-scrollbar-thumb {
    background: #cbd8e7;
    border: 3px solid var(--bg-page);
    border-radius: 999px;
  }

  .content > header {
    width: min(100%, 1120px);
    margin-left: auto;
    margin-right: auto;
  }

  .topbar {
    display: flex;
    align-items: flex-end;
    min-width: 0;
    margin-bottom: 12px;
  }

  .topbar h2 {
    margin: 0;
    color: var(--text-main);
    font-size: 24px;
    font-weight: 800;
    line-height: 1.2;
    letter-spacing: 0;
  }

  .ui-compact .topbar {
    margin-bottom: 10px;
  }

  .ui-compact .topbar h2 {
    font-size: 22px;
  }

  .ui-compact .content {
    padding: 14px 16px;
  }

  .content.overview-content {
    display: grid;
    grid-auto-rows: max-content;
    gap: 14px;
    align-content: start;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .ui-compact .content.overview-content {
    gap: 12px;
  }

  @media (max-width: 920px) {
    .shell {
      grid-template-columns: 210px minmax(0, 1fr);
    }

    .content {
      padding: 16px;
    }

    .content.overview-content {
      overflow: auto;
    }
  }

  @media (max-width: 760px) {
    .save-status {
      max-width: 112px;
    }
  }

  @keyframes save-pulse {
    0%,
    100% {
      opacity: 0.45;
      transform: scale(0.82);
    }

    50% {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>
