# VoxType OSS Readiness and 0.12.0 Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Present VoxType as a mature, maintainable open-source project, close reproducible CI/dependency gaps, and publish the verified 0.12.0 release.

**Architecture:** Preserve the Windows voice-input runtime and improve only repository-facing documentation, GitHub governance, CI/security automation, one flaky test assertion, and lockfile-resolved dependencies. Existing local scripts remain the source of truth for release verification, while GitHub Actions exposes quality, security, release-build, and CodeQL results as separate public signals.

**Tech Stack:** Markdown, GitHub Actions, Dependabot, CodeQL, Node.js/npm, Rust/Cargo, SvelteKit, Tauri 2, PowerShell, GitHub CLI.

## Global Constraints

- Do not change the ASR → optional LLM → clipboard → paste runtime workflow.
- Do not change configuration fields or defaults.
- Do not add runtime dependencies.
- Do not expose real keys, transcripts, hotwords, prompts, OCR text, recent-context text, logs, statistics files, or Windows usernames.
- Keep English and Chinese public entry points aligned.
- Release version is `0.12.0` and must be synchronized across all version-bearing files.

---

### Task 1: Establish the audit baseline

**Files:**
- Read: repository root, `.github/`, `src-tauri/src/`, `src/`, `scripts/`, and current GitHub repository settings
- Create: `docs/superpowers/specs/2026-08-10-oss-readiness-release-design.md`
- Create: `docs/superpowers/plans/2026-08-10-oss-readiness-release-plan.md`

**Interfaces:**
- Consumes: current `main` at `v0.11.0`, existing local release scripts, GitHub CLI authentication
- Produces: approved scope, finding inventory, and file-level implementation sequence

- [ ] Inspect `git status`, recent commits/tags, releases, open PRs/Issues, workflows, repository security settings, and community profile.
- [ ] Run `npm run ai:check`, `npm run audit:npm`, and `npm run audit:rust` to capture fresh baseline evidence.
- [ ] Inspect secrets, logs, diagnostics, update execution, Tauri CSP/capabilities, unsafe modules, public screenshots, and release scripts.
- [ ] Record reproducible findings and separate low-risk release fixes from roadmap work.

### Task 2: Rebuild the public project entry points

**Files:**
- Modify: `README.md`
- Modify: `README.en.md`
- Create: `ROADMAP.md`
- Create: `ARCHITECTURE.md`
- Modify: `CONTRIBUTING.md`
- Modify: `SECURITY.md`
- Modify: `docs/README.md`
- Create: `.github/CODEOWNERS`

**Interfaces:**
- Consumes: existing screenshots, Wiki URLs, detailed `docs/architecture.md`, support and security policies
- Produces: reviewer-first README screens and stable contributor navigation

- [ ] Add bilingual positioning, release/CI/CodeQL/license badges, screenshot, value points, download, quick start, status, maintainer, and governance links to both README first screens.
- [ ] Write the Now/Next/Later roadmap with explicit non-goals and issue-label routes.
- [ ] Write the English-first architecture entry point with the guarded data flow and module ownership map.
- [ ] Make contribution and security reporting guidance accessible to English-speaking reviewers without weakening privacy warnings.
- [ ] Add `@zkwi` as repository CODEOWNER and index the new documents.
- [ ] Run `npm run check:governance` and fix any broken local links.

### Task 3: Expose and harden engineering checks

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `.github/workflows/codeql.yml`
- Modify: `.github/dependabot.yml`
- Modify: `.github/PULL_REQUEST_TEMPLATE.md`
- Modify: `.github/ISSUE_TEMPLATE/bug_report.yml`
- Modify: `.github/ISSUE_TEMPLATE/feature_request.yml`
- Modify: `.github/ISSUE_TEMPLATE/config.yml`

**Interfaces:**
- Consumes: `npm run ai:check`, `npm run audit:npm`, `npm run audit:rust`, Cargo clippy, Tauri debug build
- Produces: separate public quality/security/release-build checks and CodeQL results

- [ ] Pin checkout, Node, Rust toolchain, Rust cache, cargo-audit installer, and CodeQL actions to verified full commit SHAs with version comments.
- [ ] Split CI into quality, dependency-audit, and Windows release-build jobs while avoiding duplicate tag-only runs.
- [ ] Add weekly and push/PR CodeQL matrices for JavaScript/TypeScript, Rust, and Actions using `security-extended` queries.
- [ ] Keep workflow permissions least-privileged and add timeouts/concurrency controls.
- [ ] Align PR and issue privacy/check wording with the published contributor workflow.
- [ ] Validate workflow YAML syntax locally and run governance checks.

### Task 4: Fix reproducible audit findings

**Files:**
- Modify: `src-tauri/src/screen_context.rs`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.lock`

**Interfaces:**
- Consumes: GitHub Actions failure at the OCR timeout upper-bound assertion, npm advisory GHSA-2v37-7h3g-55p8, RustSec RUSTSEC-2026-0221
- Produces: deterministic timeout test and patched transitive dependency locks

- [ ] Use the existing failed GitHub Actions run as the red test evidence for the scheduler-sensitive assertion.
- [ ] Remove only the `<100ms` wall-clock assertion and retain the real timeout return assertion.
- [ ] Run `cargo test screen_context::tests::wait_for_context_times_out_without_result -- --exact` repeatedly.
- [ ] Refresh only `nanoid` to 3.3.18 in npm resolution and verify `npm audit` returns zero vulnerabilities.
- [ ] Refresh only `event-listener` to 5.4.2 in Cargo resolution and verify RUSTSEC-2026-0221 no longer appears.
- [ ] Run `npm run ai:check` after the targeted fixes.

### Task 5: Prepare the 0.12.0 release record

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `CHANGELOG.md`
- Create: `docs/audits/2026-08-10-release-0.12.0-oss-readiness-audit.md`
- Modify: `docs/README.md`

**Interfaces:**
- Consumes: completed audit evidence and final diff
- Produces: version-consistent release metadata and an indexed audit record

- [ ] Set every version-bearing file to `0.12.0` using npm/Cargo-compatible tooling and minimal manual edits.
- [ ] Add the dated 0.12.0 changelog section covering presentation, engineering, security, and unchanged runtime guarantees.
- [ ] Write the release audit with finding severity, fixes, residual risks, checks, and manual verification scope.
- [ ] Index the current release audit at the top of `docs/README.md`.
- [ ] Run governance and secret scans to prove release documentation is complete and safe.

### Task 6: Publish contribution work and repository metadata

**Files:**
- External: GitHub Issues, labels, repository description/homepage/topics, security settings
- Modify if issue numbers are added: `ROADMAP.md`

**Interfaces:**
- Consumes: roadmap priorities and repository metadata audit
- Produces: real contributor tasks and a reviewer-friendly GitHub About panel

- [ ] Ensure `good first issue`, `help wanted`, `documentation`, `testing`, and `security` labels exist.
- [ ] Create bounded Issues for English maintenance documentation, Windows clean-VM smoke testing, and release artifact verification/provenance.
- [ ] Link those Issues from the roadmap.
- [ ] Update the repository description/homepage/topics without making unsupported privacy or platform claims.
- [ ] Enable Dependabot security updates if supported; leave branch-protection changes as an explicit maintainer decision if current account/repository constraints make them disruptive.

### Task 7: Verify, commit, push, and release

**Files:**
- Inspect: complete staged diff and generated release artifacts
- External: Git commit, `main` push, tag `v0.12.0`, GitHub Release

**Interfaces:**
- Consumes: complete release diff and local credentials
- Produces: published, remotely verified `v0.12.0`

- [ ] Run `npm run ai:release-check` and capture the complete exit status.
- [ ] Run `npx tauri build` and confirm the NSIS setup executable exists and is non-empty.
- [ ] Run staged secret scanning, governance checks, `git diff --check`, and a final scope review.
- [ ] Commit with a concise Simplified Chinese message and push `main` without force.
- [ ] Wait for CI and CodeQL on the pushed commit; inspect and fix any actionable failure before tagging.
- [ ] Create and push annotated tag `v0.12.0`, then publish a non-draft GitHub Release with the NSIS installer and checksums if produced by the build.
- [ ] Verify the release URL, assets, latest-release metadata, issue links, repository About panel, and final clean working tree.
