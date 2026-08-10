# VoxType OSS Readiness and 0.12.0 Release Design

## Goal

Make VoxType understandable and trustworthy to a first-time open-source reviewer within one to three minutes, without adding product features or changing the voice-input runtime workflow.

## Approved Scope

The user selected the OSS-complete option: improve repository presentation, project status, roadmap, architecture, contribution paths, CI, dependency/security governance, and public project metadata; then audit, verify, publish, and release the result.

## Design

### Repository first impression

The default Chinese README will start with an English-first bilingual identity line, a concise privacy-conscious positioning statement, live release/CI/CodeQL/license badges, a primary screenshot, value points, download and quick-start links, and project-status/maintainer links. `README.en.md` will mirror the same information architecture in English. Existing detailed configuration and troubleshooting content remains available below the new first screen so useful documentation is not discarded.

### Open-source project identity

Add a root `ROADMAP.md` with Now/Next/Later priorities, contribution labels, explicit non-goals, and security-sensitive work that requires prior design discussion. Add a root `ARCHITECTURE.md` as an English-first contributor entry point that maps the Windows/Tauri/Svelte modules and the guarded runtime data flow, while keeping `docs/architecture.md` as the detailed Chinese maintenance document. Name the current maintainer in the README and CODEOWNERS without implying a commercial support SLA.

### Contributor workflow

Keep the existing issue forms and pull-request template, but make the contribution entry points easier to discover and align their checklists with the public CI jobs. Create a small set of real, bounded GitHub Issues carrying `good first issue`, `help wanted`, documentation, testing, and security labels. Issues must describe acceptance criteria and must not invent user demand or promise unsupported platforms.

### Engineering and security credibility

Restructure CI so ordinary branch pushes and pull requests show separate quality, dependency-security, and Windows release-build jobs. Pin every reusable GitHub Action to a full commit SHA and let Dependabot maintain those pins. Add CodeQL advanced scanning for JavaScript/TypeScript, Rust, and GitHub Actions with read-only repository permissions plus the minimum `security-events: write` permission.

The audit will cover checked-in secrets, local-data exclusions, log/diagnostic redaction, Tauri CSP/capabilities, external command and update execution, dependency advisories, unsafe Win32 boundaries, release scripts, version synchronization, GitHub repository settings, public screenshots, and current Actions results. Fix only findings that are directly reproducible and low risk in this release. Larger runtime-security work becomes a tracked roadmap item.

### Reliability fixes

Remove the wall-clock upper-bound assertion from the existing OCR timeout test. The test will continue to exercise the real empty-channel timeout path; only the scheduler-sensitive `<100ms` assertion is removed. Update `nanoid` from 3.3.16 to 3.3.18 in the npm lockfile and `event-listener` from 5.4.1 to 5.4.2 in the Cargo lockfile to resolve current advisories without changing direct dependency declarations.

### Release

Release as `0.12.0`. The change is a minor release because it materially changes public project presentation and engineering/security governance, even though the end-user voice-input workflow is unchanged. Synchronize all five version-bearing files, `CHANGELOG.md`, the release audit, and the documentation index. Build the NSIS installer, publish the commit and tag, wait for remote CI/CodeQL, and create a GitHub Release with installer assets.

## Non-goals

- No new voice-input feature, ASR provider, LLM behavior, or UI setting.
- No changes to ASR packet timing, final-result selection, OCR timeout defaults, clipboard behavior, hotkeys, tray behavior, statistics, or privacy defaults.
- No broad refactor of Svelte or Rust modules.
- No new runtime dependency.
- No claim that cloud ASR or optional LLM processing is local-only.
- No cryptographic update-signing system in this release; that requires a dedicated design and release-key lifecycle.

## Verification

- Targeted Rust OCR timeout test passes repeatedly without a scheduler-dependent upper bound.
- `npm audit` reports zero known vulnerabilities and `cargo audit` has no unacknowledged vulnerability that applies to the supported Windows runtime.
- `npm run ai:release-check` and the production `npx tauri build` complete successfully.
- Governance checks confirm version parity, release-audit indexing, Markdown links, screenshots, Wiki mirrors, and i18n keys.
- Git-visible and staged secret scans pass.
- Remote CI and CodeQL complete successfully on the published commit.
- GitHub shows the `v0.12.0` release, Windows installer assets, updated repository metadata, and the new contribution Issues.
