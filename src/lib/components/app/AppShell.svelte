<script lang="ts">
  import type { Snippet } from "svelte";
  import type { CopyKey, Language } from "$lib/i18n";
  import type { Section } from "$lib/types/app";
  import {
    BarChart3,
    Download,
    Gauge,
    Maximize2,
    Mic,
    Minus,
    Settings,
    ShieldCheck,
    Sparkles,
    X as XIcon,
  } from "lucide-svelte";

  type Translate = (key: CopyKey, values?: Record<string, string>) => string;
  type MaybeAsync = void | Promise<void>;

  type Props = {
    children?: Snippet;
    uiCompact: boolean;
    selectedSection: Section;
    language: Language;
    recording: boolean;
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
    { id: "Home", icon: Gauge },
    { id: "Hotwords", icon: Sparkles },
    { id: "ApiConfig", icon: ShieldCheck },
    { id: "Options", icon: Settings },
    { id: "History", icon: BarChart3 },
  ] as const;

  const navLabelKeys: Record<Section, CopyKey> = {
    Home: "navHome",
    Hotwords: "navHotwords",
    ApiConfig: "navApiConfig",
    Options: "navOptions",
    History: "navHistory",
  };
</script>

<div class:ui-compact={uiCompact} class="app-frame">
  <header class="window-titlebar" data-tauri-drag-region>
    <div class="window-title" data-tauri-drag-region>
      <span class="window-title-mark"><Mic size={12} strokeWidth={2.6} /></span>
      <strong data-tauri-drag-region>{t("appTitle")}</strong>
      <span data-tauri-drag-region>VoxType</span>
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
          <button
            class:active={selectedSection === item.id}
            onclick={() => onSelectSection(item.id)}
          >
            <Icon size={17} />
            <span>{t(navLabelKeys[item.id])}</span>
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

  .window-title > span:last-child {
    min-width: 0;
    overflow: hidden;
    color: var(--text-secondary);
    text-overflow: ellipsis;
    white-space: nowrap;
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
    overflow: hidden;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
  }

  .ui-compact .sidebar {
    gap: 11px;
    padding: 14px 16px;
  }

  nav {
    display: grid;
    gap: 8px;
  }

  .ui-compact nav {
    gap: 6px;
  }

  nav button {
    display: flex;
    align-items: center;
    width: 100%;
    min-height: 42px;
    margin: 0;
    padding: 0 14px;
    gap: 12px;
    color: var(--text-main);
    border-radius: var(--radius-md);
    font-size: 15px;
    font-weight: 500;
    transition: all 160ms ease;
  }

  nav button span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ui-compact nav button {
    min-height: 38px;
    padding: 0 12px;
    font-size: 14px;
  }

  nav button:hover {
    color: var(--primary);
    background: var(--primary-light);
  }

  nav button.active {
    color: #ffffff;
    background: var(--primary);
    box-shadow: 0 12px 28px rgba(47, 128, 237, 0.24);
  }

  .language-control {
    display: grid;
    gap: 10px;
    margin: 6px 0 0;
  }

  .ui-compact .language-control {
    gap: 8px;
    margin-top: 4px;
  }

  .language-control span {
    color: var(--text-secondary);
    font-size: 14px;
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
    border-radius: 10px;
    font-size: 15px;
  }

  .ui-compact .language-control select {
    min-height: 34px;
    font-size: 14px;
  }

  .bridge-card {
    display: grid;
    gap: 7px;
    margin: auto 0 0;
    padding: 12px;
    min-width: 0;
    overflow: hidden;
    background: #ffffff;
    border: 1px solid var(--border);
    border-radius: 14px;
    box-shadow: var(--shadow-card);
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
</style>
