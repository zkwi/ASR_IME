# Engineering Governance Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make VoxType releases repeatable by detecting locked build artifacts early, enforcing complete version/audit synchronization, and removing current documentation drift.

**Architecture:** Keep governance lightweight: PowerShell owns Windows release preflight, the existing Node governance script owns repository consistency, and Markdown remains the source of truth for maintenance guidance. Add focused self-tests before changing either script, then synchronize living documentation without rewriting historical audits.

**Tech Stack:** PowerShell 7/Windows PowerShell, Node.js ESM, npm scripts, Markdown, existing Rust/Tauri/Svelte checks.

## Global Constraints

- Do not change ASR, LLM, clipboard, hotkey, tray, logging, statistics, or configuration behavior.
- Do not introduce dependencies or a new test framework.
- Never print configuration values, credentials, recognized text, prompts, hotwords, OCR, or recent context.
- Historical release audits remain append-only except for obvious sensitive-data corrections.
- Release as `0.8.1` after all local and remote checks pass.

---

### Task 1: Release preflight regression coverage

**Files:**
- Create: `scripts/release-preflight.ps1`
- Create: `scripts/test-release-preflight.ps1`
- Modify: `scripts/ai-release-check.ps1`
- Modify: `scripts/ai-check.ps1`
- Modify: `package.json`

**Interfaces:**
- Produces: `Assert-ReleaseBuildArtifactWritable -Path PATH`, which returns silently for a missing/writable artifact and throws an actionable message for a locked artifact.
- Consumes: `src-tauri/target/debug/voxtype-desktop.exe` as the preflight target.

- [x] Write `test-release-preflight.ps1` with missing-file, writable-file, and exclusively locked-file cases.
- [x] Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-release-preflight.ps1`; expect failure because `release-preflight.ps1` does not exist.
- [x] Implement the minimal exclusive-open check and an error that names the path and asks the maintainer to close the running debug app.
- [x] Insert preflight before expensive release checks and add `test:release-preflight` to `ai:check`.
- [x] Re-run the focused test; expect all three cases to pass.

### Task 2: Version and release-audit governance

**Files:**
- Modify: `scripts/test-governance.mjs`
- Modify: `scripts/check-governance.mjs`

**Interfaces:**
- Produces: governance failures when the workspace package version is absent from `Cargo.lock`, when no `docs/audits/*release-VERSION-*.md` exists, or when `docs/README.md` does not link that audit.
- Consumes: existing `checkVersionConsistency(root, failures)` flow.

- [x] Extend the valid test fixture with `Cargo.lock`, a current release audit, and a `docs/README.md` link.
- [x] Add three negative fixture cases for Cargo.lock mismatch, missing audit, and missing index link.
- [x] Run `npm run test:governance`; expect failure because current governance does not detect the new cases.
- [x] Implement minimal Cargo.lock package-version parsing and release-audit/index checks.
- [x] Run `npm run test:governance`; expect all governance self-tests to pass.

### Task 3: Living documentation synchronization

**Files:**
- Modify: `README.md`
- Modify: `README.en.md`
- Modify: `AGENTS.md`
- Modify: `docs/maintenance-guide.md`
- Modify: `docs/code-style.md`
- Modify: `docs/README.md`
- Modify: relevant files under `docs/wiki/`
- Modify: `docs/plans/2026-07-13-code-quality-improvement-plan.md`
- Modify: comments in changed governance/preflight scripts only.

**Interfaces:**
- Produces: one shared test taxonomy: unit tests use synthetic/local data, provider connection tests send generated silence, and real recording tests require deliberate microphone use.
- Produces: current commands (`test:unit`, `test:release-preflight`, `ai:check`, `ai:release-check`) and release troubleshooting guidance.

- [x] Audit every living README/Wiki/maintenance document for obsolete commands and inaccurate current-state claims; do not rewrite point-in-time audit history.
- [x] Replace the obsolete `test:stats` command with `test:unit` and document release preflight behavior.
- [x] Add bilingual test-boundary guidance where users/maintainers choose online tests.
- [x] Mark every completed 0.8.0 plan item with its actual outcome; mark optional P2 items explicitly deferred and explain why.
- [x] Add the 0.8.0 and 0.8.1 audit links to the documentation index during release preparation.

### Task 4: Verification and 0.8.1 release

**Files:**
- Modify: `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `CHANGELOG.md`
- Create: `docs/audits/2026-07-13-release-0.8.1-engineering-governance-audit.md`

**Interfaces:**
- Produces: aligned version `0.8.1`, an NSIS installer, tag `v0.8.1`, and a GitHub Release.

- [x] Run focused self-tests, `npm run check:governance`, and `npm run ai:check`.
- [x] Synchronize all version files to `0.8.1`, add CHANGELOG and release audit records, then run `npm run ai:release-check`.
- [x] Build the NSIS installer, record byte size and SHA-256 in the audit, and re-run governance after the audit is final.
- [ ] Review `git diff`, run staged secret scanning, commit in Simplified Chinese, fetch/rebase safety-check, and push `main`.
- [ ] Tag `v0.8.1`, create the GitHub Release with the installer, verify the remote digest, and wait for GitHub CI success.

## Self-Review

- Spec coverage: README, English README, all living Wiki mirrors, maintenance rules, scripts, governance tests, comments, versioning, push, and release are covered.
- Placeholder scan: no deferred implementation placeholders are present; P2 business refactors are explicitly out of scope.
- Type consistency: PowerShell function and npm script names are identical across tasks and documentation.
