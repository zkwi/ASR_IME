# VoxType maintenance guide

This document is for day-to-day maintenance and small release iterations. The goal is to help maintainers quickly decide which module a change belongs in, and which boundaries must not be casually crossed.

## Locate the path before you change code

VoxType's core path is:

```text
trigger recording -> microphone capture -> ASR provider -> optional LLM polish -> clipboard output -> stats and local history
```

Before editing, decide whether the change touches any of these areas:

- ASR provider, audio chunking, final-packet selection, empty-recognition handling.
- LLM trigger conditions, prompt assembly, recent context, or screen OCR reference text.
- Clipboard write, auto-paste, original-clipboard restore.
- Logs, diagnostic reports, statistics, local transcript history.
- Hotkeys, tray, window-close behavior.

If you touch these areas, read [ASR quality and latency guardrails](asr-quality-latency-guardrails.md) first, then decide the test scope.

## ASR provider boundaries

`src-tauri/src/asr_provider.rs` is the unified entry point. It is responsible for only three things:

- Resolve the current provider.
- Run start-up configuration gates.
- Hand recording-session parameters to the concrete provider.

Doubao protocol details live under `src-tauri/src/asr_ws/`. Alibaba Cloud FunASR protocol details live in `aliyun_asr.rs`. Do not push provider-specific WebSocket payloads, event parsing, or final-text selection logic back into `asr_provider.rs`.

The Doubao ASR directory is split by responsibility: `worker.rs` only orchestrates ASR, LLM, and output; `session.rs` handles the Doubao WebSocket session loop; `audio_stream.rs` handles the audio queue and send pacing; `connection.rs` handles connection tests and handshake; `final_text.rs` handles final-text selection; `partial_text.rs` handles live-caption throttling; `output.rs` handles final output events and side effects; `errors.rs` handles error classification. When adding Doubao behavior, put it in the matching module first — do not pile logic back into `mod.rs`.

When adding a provider, prefer the current lightweight dispatch style. Only introduce a trait or heavier abstraction once provider count and shared behavior are clearly duplicated.

## Final-text gates

Live captions and final text must be handled separately:

- Doubao: intermediate packets only update captions; final output waits for the final packet and second-pass sentence selection.
- Alibaba Cloud: `result-generated` only updates captions; final output must wait for `task-finished`.

No intermediate text may trigger LLM polish, paste, success stats, recent context, or automatic hotword history. An empty final text must enter the failure path.

## Settings page maintenance

The settings page should stay approachable for ordinary users, with advanced parameters available for repair:

- High-frequency, required fields are shown directly.
- Low-frequency, protocol, compatibility, or troubleshooting fields prefer a collapsed section.
- Advanced sections that are enabled, non-default, or have validation errors must auto-expand.
- Field-validation navigation is owned by `src/lib/utils/settingsFields.ts`; panel ids must match the component `id` values.
- Reuse `src/lib/components/common/AdvancedSettings.svelte` for collapsible panels — do not reimplement the same DOM/CSS on each page.
- Reuse `src/lib/components/common/ActionPanel.svelte` for "description + metadata + action buttons" cards; pages keep only business buttons and state checks.

When adding a settings field, follow the configuration-sync checklist in `AGENTS.md`. Do not change only Rust or only the frontend.

## Privacy and diagnostics

By default, never write any of the following into logs, diagnostic reports, release audits, or screenshots:

- Real credentials / API keys.
- Recognition transcript body.
- Hotwords, prompts, or recent-context body text.
- Screen OCR body text.
- Windows username paths.

Statistics store non-body metrics only. Even when recent context and automatic hotword history are enabled, they may only enter their own local data files — never write them back into `config.toml`.

## Pre-release checks

Day-to-day changes:

```powershell
npm run test:unit
npm run ai:check
```

Before a release:

```powershell
npm run ai:release-check
npx tauri build
```

`ai:release-check` first confirms the debug EXE is not locked by a running VoxType instance, then covers the day-to-day checks, npm audit, Rust audit, clippy, and a Tauri debug build. If the preflight reports a file lock, close the debug app from this session and retry — do not wait until the final Tauri build to debug it. GitHub Actions CI reuses the same entry point; if local release checks fail, do not push a release branch.

Treat test evidence as three separate layers — do not mix them:

- Unit/governance tests use synthetic or local temporary data only and must not call providers.
- API-settings ASR connection tests use real credentials and send a program-generated short silence packet to the selected service, but do not open the microphone.
- Real recording regressions capture and send live microphone audio; only record them as completed when the change truly touches capture or the full ASR main path and a maintainer explicitly ran them.

Release version numbers should reflect impact:

- patch: pure maintenance, docs, small fixes, narrow copy changes.
- minor: user-visible features, clear UX adjustments, default-policy changes.
- major: breaking compatibility or requiring users to relearn the core workflow.

On release, keep `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `CHANGELOG.md`, `docs/audits/`, and the current release-audit entry in `docs/README.md` in sync.

## Configuration sync, secrets scan, and rollback

Before merging maintainer changes that touch settings or release metadata:

```powershell
npm run check:governance
npm run scan:secrets
```

`check:governance` validates docs/governance consistency used by CI. `scan:secrets` must stay clean — never commit transcripts, credentials, hotwords, prompts, OCR text, recent context, logs, statistics dumps, or Windows username paths.

If a release build or publish is bad:

1. Stop distributing the bad artifact (GitHub Release / installer channel).
2. Prefer a forward fix release over rewriting published tags.
3. Record the incident under `docs/audits/` and link it from `docs/README.md`.
4. Re-run `npm run ai:release-check` on the fix branch before tagging again.

Chinese original: [maintenance-guide.md](maintenance-guide.md).